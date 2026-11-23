//! Escentron model parameters, fields, energy accounting, and laws of motion.

pub(crate) mod allocation {
    //! Allocate a proposed kinetic-energy change between acceleration channels.
    //!
    //! Fields are never combined here. The inputs are acceleration vectors
    //! obtained separately from the gravitational and levitational responses.
    //! This is a geometric allocation rule; it is not a timestep integrator.
    use crate::Vector3;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AllocationError {
        NonFiniteInput,
        NonFiniteResult,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct AccelerationShares {
        /// Acceleration after vector addition of the two response channels.
        pub total: Vector3,
        /// Fraction of a subsequent kinetic-energy change assigned to gravity.
        pub gravitational: f64,
        /// Fraction assigned to levitation. Shares sum to one for nonzero total.
        pub levitational: f64,
    }

    /// Compute the relative projections onto total acceleration without sqrt.
    ///
    /// Let A = a_g + a_l and d_g = a_g dot A, d_l = a_l dot A.
    /// Since d_g + d_l = |A|², the positive projection ratios follow directly:
    ///   w_g = d_g / (d_g + d_l), w_l = d_l / (d_g + d_l).
    ///
    /// If one projection opposes A (negative dot product), the opposing channel
    /// is assigned zero; the other channel receives all of the *net* effect.
    /// In perfect cancellation A=0, both shares are zero.
    ///
    /// This function does not move energy from one non-kinetic reservoir to the
    /// other. It only labels how a subsequent signed kinetic-energy delta is
    /// attributed to its source or destination reservoirs.
    pub fn acceleration_shares(
        gravitational: Vector3,
        levitational: Vector3,
    ) -> Result<AccelerationShares, AllocationError> {
        if !gravitational.is_finite() || !levitational.is_finite() {
            return Err(AllocationError::NonFiniteInput);
        }
        let total = Vector3::new(
            gravitational.x + levitational.x,
            gravitational.y + levitational.y,
            gravitational.z + levitational.z,
        );
        if !total.is_finite() {
            return Err(AllocationError::NonFiniteResult);
        }
        let dot = |a: Vector3, b: Vector3| a.x*b.x + a.y*b.y + a.z*b.z;
        let total_squared = dot(total, total);
        if !total_squared.is_finite() {
            return Err(AllocationError::NonFiniteResult);
        }
        if total_squared == 0.0 {
            return Ok(AccelerationShares {
                total,
                gravitational: 0.0,
                levitational: 0.0,
            });
        }

        let g = dot(gravitational, total);
        let l = dot(levitational, total);
        if !g.is_finite() || !l.is_finite() {
            return Err(AllocationError::NonFiniteResult);
        }

        let (g_share, l_share) = if g <= 0.0 {
            (0.0, 1.0)
        } else if l <= 0.0 {
            (1.0, 0.0)
        } else {
            // Both projections point along the resultant. No square root,
            // normalization of direction, angle or trigonometric function.
            let share = g / total_squared;
            if !share.is_finite() {
                return Err(AllocationError::NonFiniteResult);
            }
            let share = share.clamp(0.0, 1.0);
            (share, 1.0 - share)
        };
        Ok(AccelerationShares {
            total,
            gravitational: g_share,
            levitational: l_share,
        })
    }

    /// Divide a *separately computed* signed kinetic-energy change between
    /// the two reservoirs according to the acceleration shares.
    /// No reservoir is modified here; use transfer_energy for that step.
    pub fn allocate_kinetic_change(
        shares: AccelerationShares,
        delta_kinetic: f64,
    ) -> Result<(f64, f64), AllocationError> {
        if !delta_kinetic.is_finite()
            || !shares.gravitational.is_finite()
            || !shares.levitational.is_finite()
            || shares.gravitational < 0.0
            || shares.levitational < 0.0
            || shares.gravitational > 1.0
            || shares.levitational > 1.0
            || (shares.gravitational + shares.levitational - 1.0).abs() > 1e-12
                && (shares.gravitational != 0.0 || shares.levitational != 0.0)
        {
            return Err(AllocationError::NonFiniteInput);
        }
        let g = delta_kinetic * shares.gravitational;
        let l = delta_kinetic - g;
        if !g.is_finite() || !l.is_finite() {
            return Err(AllocationError::NonFiniteResult);
        }
        // If the total acceleration is zero, only zero kinetic change is valid.
        if shares.gravitational == 0.0 && shares.levitational == 0.0 {
            if delta_kinetic != 0.0 { return Err(AllocationError::NonFiniteInput); }
            return Ok((0.0, 0.0));
        }
        Ok((g, l))
    }

    #[cfg(test)]
    mod tests {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/allocation.rs"));
    }
}

pub(crate) mod analytic {
    //! Independent *continuous-limit* benchmarks for constant fields.
    //!
    //! These formulas do not call the discrete step implementation and provide
    //! analytically derived reference checks. They are model hypotheses, not
    //! experimental evidence. Only explicitly documented 1D cases are supported.

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum AnalyticError {
        InvalidInput,
        PrecisionLimit,
    }

    impl std::fmt::Display for AnalyticError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self:?}")
        }
    }
    impl std::error::Error for AnalyticError {}

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct SingleFieldExact {
        pub velocity: f64,
        pub remaining_field_energy: f64,
        pub kinetic_energy: f64,
    }

    /// Exact continuous-time response along a single fixed field axis.
    ///
    /// Assumes a single active reservoir E>0, k_k>0, E_c>0,
    /// dv/dt = field_strength * E/E_c, E + k_k*v^2 = constant.
    /// The field strength is a SIGNED 1D scalar. The opposite reservoir stays
    /// fixed and does not participate in energy exchange.
    ///
    /// Let C=E_0+k_k*v_0², U=sqrt(C/k_k),
    /// z=atanh(v_0/U)+(field_strength/E_c)*sqrt(C*k_k)*duration.
    /// Then v=U*tanh(z), E=C/cosh(z)². No timestepping and no clamping.
    /// Extremely large z can underflow to exactly zero in f64; this returns
    /// PrecisionLimit rather than inventing a positive epsilon.
    pub fn single_field_constant_exact(
        initial_velocity: f64,
        initial_field_energy: f64,
        field_strength: f64,
        duration: f64,
        kinetic_coefficient: f64,
        cosmic_energy: f64,
    ) -> Result<SingleFieldExact, AnalyticError> {
        if !initial_velocity.is_finite()
            || !initial_field_energy.is_finite()
            || !field_strength.is_finite()
            || !duration.is_finite()
            || !kinetic_coefficient.is_finite()
            || !cosmic_energy.is_finite()
            || initial_field_energy <= 0.0
            || duration < 0.0
            || kinetic_coefficient <= 0.0
            || cosmic_energy <= 0.0
        {
            return Err(AnalyticError::InvalidInput);
        }
        let c = initial_field_energy
            + kinetic_coefficient * initial_velocity * initial_velocity;
        let u = (c / kinetic_coefficient).sqrt();
        let ratio = initial_velocity / u;
        if !u.is_finite() || !ratio.is_finite() || ratio.abs() >= 1.0 {
            return Err(AnalyticError::PrecisionLimit);
        }
        let z = ratio.atanh()
            + (field_strength / cosmic_energy)
                * (c * kinetic_coefficient).sqrt() * duration;
        if !z.is_finite() {
            return Err(AnalyticError::PrecisionLimit);
        }
        let ch = z.cosh();
        if !ch.is_finite() {
            return Err(AnalyticError::PrecisionLimit);
        }
        let energy = (c / ch) / ch;
        let velocity = u * z.tanh();
        let kinetic_energy = kinetic_coefficient * velocity * velocity;
        if !energy.is_finite() || energy <= 0.0
            || !velocity.is_finite() || !kinetic_energy.is_finite()
        {
            return Err(AnalyticError::PrecisionLimit);
        }
        Ok(SingleFieldExact {
            velocity,
            remaining_field_energy: energy,
            kinetic_energy,
        })
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct AlignedFieldsExact {
        pub gravitational_energy: f64,
        pub levitational_energy: f64,
    }

    /// Analytical energy-versus-distance invariant, for two co-directed
    /// positive constant fields along +x, positive displacement x, and a
    /// continuous trajectory whose acceleration attribution stays co-directed.
    ///
    /// dE_g/dx = -2*k_k*(f_g/E_c)*E_g
    /// dE_l/dx = -2*k_k*(f_l/E_c)*E_l
    /// Hence E_g(x)=E_g(0)*exp(-2*k_k*f_g*x/E_c) and similarly for E_l.
    ///
    /// This is NOT a closed-form solution for position as a function of time.
    /// Reverse trajectories and opposing field directions require separate
    /// treatment; no general solution is claimed.
    pub fn aligned_fields_energy_at_distance(
        initial_gravitational_energy: f64,
        initial_levitational_energy: f64,
        gravitational_field: f64,
        levitational_field: f64,
        displacement: f64,
        kinetic_coefficient: f64,
        cosmic_energy: f64,
    ) -> Result<AlignedFieldsExact, AnalyticError> {
        let inputs = [
            initial_gravitational_energy, initial_levitational_energy,
            gravitational_field, levitational_field, displacement,
            kinetic_coefficient, cosmic_energy,
        ];
        if !inputs.iter().all(|v| v.is_finite())
            || initial_gravitational_energy <= 0.0
            || initial_levitational_energy <= 0.0
            || gravitational_field <= 0.0
            || levitational_field <= 0.0
            || displacement < 0.0
            || kinetic_coefficient <= 0.0
            || cosmic_energy <= 0.0
        {
            return Err(AnalyticError::InvalidInput);
        }
        let factor = -2.0 * kinetic_coefficient * displacement / cosmic_energy;
        let eg = initial_gravitational_energy
            * (factor * gravitational_field).exp();
        let el = initial_levitational_energy
            * (factor * levitational_field).exp();
        if !eg.is_finite() || !el.is_finite() || eg <= 0.0 || el <= 0.0 {
            return Err(AnalyticError::PrecisionLimit);
        }
        Ok(AlignedFieldsExact {
            gravitational_energy: eg,
            levitational_energy: el,
        })
    }

    #[cfg(test)]
    mod tests {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/analytic.rs"));
    }
}

pub(crate) mod budget {
    //! Accounting for energy exchange between one source reservoir and kinetic energy.
    //!
    //! These functions do not determine the amount of energy to exchange.
    //! Field response, acceleration, timestep and velocity update are separate.
    use crate::EnergyBreakdown;

    /// Which of the Escentron's own reservoirs exchanges energy with kinetic energy.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EnergyReservoir {
        Gravitational,
        Levitational,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TransferError {
        NonFinite,
        NegativeEnergy,
        NonPositiveFieldEnergy,
        InsufficientSourceEnergy,
        InsufficientKineticEnergy,
    }

    impl std::fmt::Display for TransferError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::NonFinite => write!(f, "energy transfer requires finite values"),
                Self::NegativeEnergy => write!(f, "energy reservoirs cannot be negative"),
                Self::NonPositiveFieldEnergy => write!(f, "gravitational and levitational reservoirs must remain positive"),
                Self::InsufficientSourceEnergy =>
                    write!(f, "selected reservoir cannot supply the requested kinetic increase"),
                Self::InsufficientKineticEnergy =>
                    write!(f, "kinetic reservoir cannot supply the requested reverse transfer"),
            }
        }
    }
    impl std::error::Error for TransferError {}

    /// Transfer a *specified* signed kinetic-energy change between one selected
    /// reservoir and the kinetic reservoir of the SAME Escentron.
    ///
    /// delta_kinetic > 0: selected reservoir -> kinetic energy
    /// delta_kinetic < 0: kinetic energy -> selected reservoir
    ///
    /// The other non-kinetic reservoir is unchanged. Energy is not transferred
    /// between Escentrons. This is bookkeeping, not an acceleration algorithm.
    /// The caller must determine delta_kinetic from the final dynamics law.
    pub fn transfer_energy(
        current: EnergyBreakdown,
        source: EnergyReservoir,
        delta_kinetic: f64,
    ) -> Result<EnergyBreakdown, TransferError> {
        let values = [
            current.gravitational,
            current.levitational,
            current.kinetic,
            delta_kinetic,
        ];
        if values.iter().any(|value| !value.is_finite()) {
            return Err(TransferError::NonFinite);
        }
        if current.gravitational < 0.0
            || current.levitational < 0.0
            || current.kinetic < 0.0
        {
            return Err(TransferError::NegativeEnergy);
        }

        if current.gravitational == 0.0 || current.levitational == 0.0 {
            return Err(TransferError::NonPositiveFieldEnergy);
        }

        let source_energy = match source {
            EnergyReservoir::Gravitational => current.gravitational,
            EnergyReservoir::Levitational => current.levitational,
        };

        if delta_kinetic > source_energy {
            return Err(TransferError::InsufficientSourceEnergy);
        }
        if delta_kinetic < -current.kinetic {
            return Err(TransferError::InsufficientKineticEnergy);
        }

        let updated_source = source_energy - delta_kinetic;
        let updated_kinetic = current.kinetic + delta_kinetic;
        if !updated_source.is_finite() || !updated_kinetic.is_finite() {
            return Err(TransferError::NonFinite);
        }

        if updated_source == 0.0 {
            return Err(TransferError::NonPositiveFieldEnergy);
        }

        let mut updated = current;
        match source {
            EnergyReservoir::Gravitational => updated.gravitational = updated_source,
            EnergyReservoir::Levitational => updated.levitational = updated_source,
        }
        updated.kinetic = updated_kinetic;
        Ok(updated)
    }

    #[cfg(test)]
    mod tests {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/budget.rs"));
    }
}

pub(crate) mod constants {
    //! Explicit parameters of a simulation; no arbitrary default values.
    use crate::validation::{finite_nonnegative, finite_positive, ValidationError};

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct ModelConstants {
        /// G: common field scale parameter.
        pub g: f64,
        /// E_g = k_g * m
        pub k_g: f64,
        /// E_l = k_l * p
        pub k_l: f64,
        /// E_k = k_k * |v|^2
        pub k_k: f64,
        /// Shared per-particle cosmic energy E_c.
        pub cosmic_energy: f64,
    }

    impl ModelConstants {
        pub fn validate(&self) -> Result<(), ValidationError> {
            finite_nonnegative(self.g, "G")?;
            finite_nonnegative(self.k_g, "k_g")?;
            finite_nonnegative(self.k_l, "k_l")?;
            finite_nonnegative(self.k_k, "k_k")?;
            finite_positive(self.cosmic_energy, "E_c")?;
            Ok(())
        }
    }
}

pub(crate) mod constraint {
    //! Geometric building block for energy-constrained acceleration.
    //!
    //! This module is NOT an energy-depletion solver. A future dynamics algorithm
    //! must determine which direction drains a reservoir and the permissible
    //! forward acceleration from the reservoir's remaining energy.
    use crate::Vector3;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DirectionalError {
        NonFiniteInput,
        ZeroDirection,
        NegativeLimit,
        NonFiniteResult,
    }

    /// Preserve acceleration perpendicular to and opposite to a specified
    /// depletion direction, while limiting its forward component.
    ///
    /// Given a direction d and acceleration a:
    ///   t = (a dot d) / (d dot d)
    ///   a_corrected = a - max(t - max_forward_factor, 0) * d
    ///
    /// The limit refers to the coefficient multiplying d, not to a speed or
    /// energy value. The direction need not be normalized; no sqrt is required.
    ///
    /// A caller can set max_forward_factor=0 once a reservoir reaches its
    /// numerical depletion threshold. The forbidden component then vanishes,
    /// while orthogonal and reverse components remain intact.
    ///
    /// IMPORTANT: choosing d and max_forward_factor requires a separate physical
    /// energy-transfer and event-detection rule. No such rule is inferred here.
    pub fn limit_forward_acceleration(
        acceleration: Vector3,
        depletion_direction: Vector3,
        max_forward_factor: f64,
    ) -> Result<Vector3, DirectionalError> {
        if !acceleration.is_finite()
            || !depletion_direction.is_finite()
            || !max_forward_factor.is_finite()
        {
            return Err(DirectionalError::NonFiniteInput);
        }
        if max_forward_factor < 0.0 {
            return Err(DirectionalError::NegativeLimit);
        }
        let d2 = depletion_direction.magnitude_squared();
        if !d2.is_finite() { return Err(DirectionalError::NonFiniteResult); }
        if d2 == 0.0 { return Err(DirectionalError::ZeroDirection); }

        let dot = acceleration.x * depletion_direction.x
            + acceleration.y * depletion_direction.y
            + acceleration.z * depletion_direction.z;
        if !dot.is_finite() { return Err(DirectionalError::NonFiniteResult); }
        let forward_factor = dot / d2;
        if !forward_factor.is_finite() {
            return Err(DirectionalError::NonFiniteResult);
        }
        if forward_factor <= max_forward_factor {
            return Ok(acceleration);
        }
        let excess = forward_factor - max_forward_factor;
        let corrected = Vector3::new(
            acceleration.x - excess * depletion_direction.x,
            acceleration.y - excess * depletion_direction.y,
            acceleration.z - excess * depletion_direction.z,
        );
        if !corrected.is_finite() { return Err(DirectionalError::NonFiniteResult); }
        Ok(corrected)
    }

    #[cfg(test)]
    mod tests {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/constraint.rs"));
    }
}

pub(crate) mod coupling {
    //! Explicit adapter: external source fields -> caller-defined raw acceleration
    //! proposals -> energy-fraction scaling -> reference simulation.
    //!
    //! Only the target evolves. All other source particles are frozen during the
    //! reference interval. This is NOT a self-consistent multi-body simulation.
    use crate::{
        acceleration_from_fields, fields_for_target_state,
        gravitational_response_coefficient, levitational_response_coefficient,
        AccelerationBreakdown,
        ModelConstants, ParticleState, ResponseError, ResponseLaw,
        Vector3, FieldTotals, DirectFieldResponse,
    };

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum CouplingError {
        Field(crate::FieldError),
        Response(ResponseError),
    }

    impl From<crate::FieldError> for CouplingError {
        fn from(error: crate::FieldError) -> Self { Self::Field(error) }
    }
    impl From<ResponseError> for CouplingError {
        fn from(error: ResponseError) -> Self { Self::Response(error) }
    }

    /// Full auditable breakdown at a single position. The two field types
    /// remain separate and are never numerically added.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct CouplingTrace {
        pub fields: FieldTotals,
        /// Raw acceleration proposals from a supplied, explicitly chosen law.
        pub proposed: AccelerationBreakdown,
        pub gravitational_energy_fraction: f64,
        pub levitational_energy_fraction: f64,
        /// The two response-scaled accelerations and their vector resultant.
        pub applied: AccelerationBreakdown,
    }

    fn scale_vector(vector: Vector3, factor: f64) -> Vector3 {
        Vector3::new(vector.x * factor, vector.y * factor, vector.z * factor)
    }

    fn checked_acceleration(
        gravitational: Vector3,
        levitational: Vector3,
    ) -> Result<AccelerationBreakdown, CouplingError> {
        if !gravitational.is_finite() {
            return Err(CouplingError::Response(ResponseError::InvalidAcceleration("gravitational")));
        }
        if !levitational.is_finite() {
            return Err(CouplingError::Response(ResponseError::InvalidAcceleration("levitational")));
        }
        let total = Vector3::new(
            gravitational.x + levitational.x,
            gravitational.y + levitational.y,
            gravitational.z + levitational.z,
        );
        if !total.is_finite() {
            return Err(CouplingError::Response(ResponseError::NonFiniteTotal));
        }
        Ok(AccelerationBreakdown { gravitational, levitational, total })
    }

    /// Trace the complete *known* part of field-to-acceleration processing:
    /// separate fields -> explicit raw law -> individual energy fractions
    /// -> separately scaled acceleration vectors.
    ///
    /// This function intentionally does not assume any missing physical
    /// relation for conversion from fields to raw acceleration proposals.
    pub fn coupling_trace(
        sources: &[ParticleState],
        target_index: usize,
        current_target: &ParticleState,
        constants: &ModelConstants,
        law: &impl ResponseLaw,
    ) -> Result<CouplingTrace, CouplingError> {
        let fields = fields_for_target_state(sources, target_index, current_target, constants)?;
        let proposed = acceleration_from_fields(current_target, &fields, law)?;
        let gravitational_energy_fraction =
            gravitational_response_coefficient(current_target, constants)?;
        let levitational_energy_fraction =
            levitational_response_coefficient(current_target, constants)?;
        let applied = checked_acceleration(
            scale_vector(proposed.gravitational, gravitational_energy_fraction),
            scale_vector(proposed.levitational, levitational_energy_fraction),
        )?;
        Ok(CouplingTrace {
            fields,
            proposed,
            gravitational_energy_fraction,
            levitational_energy_fraction,
            applied,
        })
    }

    /// Lightweight path for the reference solver. The independent field response
    /// and the two energy fractions are applied exactly once.
    pub fn coupled_acceleration(
        sources: &[ParticleState],
        target_index: usize,
        current_target: &ParticleState,
        constants: &ModelConstants,
        law: &impl ResponseLaw,
    ) -> Result<AccelerationBreakdown, CouplingError> {
        Ok(coupling_trace(sources, target_index, current_target, constants, law)?.applied)
    }

    /// The confirmed model acceleration for the target under other sources:
    /// a_g = (E_g/E_c) * f_g; a_l = (E_l/E_c) * f_l.
    /// The two acceleration vectors, unlike their source field types, can sum.
    pub fn model_acceleration(
        sources: &[ParticleState],
        target_index: usize,
        current_target: &ParticleState,
        constants: &ModelConstants,
    ) -> Result<AccelerationBreakdown, CouplingError> {
        coupled_acceleration(
            sources, target_index, current_target, constants, &DirectFieldResponse,
        )
    }

    #[cfg(test)]
    mod tests {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/coupling.rs"));
    }
}

pub(crate) mod energy {
    //! Algebraic energy bookkeeping. Does not evolve particle state.
    use crate::{ModelConstants, ParticleState, ValidationError};

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct EnergyBreakdown {
        pub gravitational: f64,
        pub levitational: f64,
        pub kinetic: f64,
    }

    impl EnergyBreakdown {
        pub fn total(self) -> f64 {
            self.gravitational + self.levitational + self.kinetic
        }

        /// Difference between computed energy and the model's shared E_c.
        /// This is a diagnostic, not an automatic correction.
        pub fn residual(self, cosmic_energy: f64) -> f64 {
            self.total() - cosmic_energy
        }
    }

    pub fn energy_of(
        state: &ParticleState,
        constants: &ModelConstants,
    ) -> Result<EnergyBreakdown, ValidationError> {
        state.validate()?;
        constants.validate()?;
        let energy = EnergyBreakdown {
            gravitational: constants.k_g * state.mass,
            levitational: constants.k_l * state.pulse,
            kinetic: constants.k_k * state.velocity.magnitude_squared(),
        };
        if !energy.gravitational.is_finite() { return Err(ValidationError::NotFinite("E_g")); }
        if !energy.levitational.is_finite() { return Err(ValidationError::NotFinite("E_l")); }
        if !energy.kinetic.is_finite() { return Err(ValidationError::NotFinite("E_k")); }
        if !energy.total().is_finite() { return Err(ValidationError::NotFinite("E_total")); }
        Ok(energy)
    }

    #[cfg(test)]
    mod tests {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/energy.rs"));
    }
}

pub(crate) mod field {
    //! Independent gravitational and levitational fields generated by Escentrons.
    //!
    //! These field types are heterogeneous and MUST NOT be added to one another.
    //! Source contributions are accumulated only within their own field type.
    //! Converting either field into an acceleration is intentionally unspecified.
    use crate::{ModelConstants, ParticleState, ValidationError, Vector3};

    /// Experimental ratio of the inner-core radius to the neutral radius.
    pub const INNER_RADIUS_RATIO: f64 = 0.001;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum FieldError {
        InvalidInput(ValidationError),
        UndefinedCoreRadius,
        InvalidTargetIndex,
        NonFiniteResult,
    }

    impl From<ValidationError> for FieldError {
        fn from(value: ValidationError) -> Self { Self::InvalidInput(value) }
    }

    impl std::fmt::Display for FieldError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::InvalidInput(err) => err.fmt(f),
                Self::UndefinedCoreRadius => write!(f, "finite inner radius cannot be defined from X=p/m"),
                Self::InvalidTargetIndex => write!(f, "target particle index is out of range"),
                Self::NonFiniteResult => write!(f, "field calculation exceeded finite numerical range"),
            }
        }
    }

    impl std::error::Error for FieldError {}

    /// A gravitational field vector (inward toward its source).
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct GravitationalField(pub Vector3);

    /// A levitational field vector (outward from its source).
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct LevitationalField(pub Vector3);

    /// Field values from a single Escentron, kept strictly separate.
    /// Magnitudes are nonnegative; vectors encode directions.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct SourceFields {
        pub distance: f64,
        pub gravitational_magnitude: f64,
        pub levitational_magnitude: f64,
        pub gravitational: GravitationalField,
        pub levitational: LevitationalField,
    }

    /// Independent superposition of same-kind fields from multiple sources.
    /// There is intentionally no heterogeneous "net field".
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct FieldTotals {
        pub gravitational: GravitationalField,
        pub levitational: LevitationalField,
    }

    impl FieldTotals {
        pub const ZERO: Self = Self {
            gravitational: GravitationalField(Vector3::ZERO),
            levitational: LevitationalField(Vector3::ZERO),
        };
    }

    /// A finite inner radius defined by R = 0.001 * X, X = p/m.
    /// The model requires strictly positive mass and pulse for every existing Escentron.
    pub fn inner_radius(source: &ParticleState) -> Result<f64, FieldError> {
        source.validate()?;
        if source.mass <= 0.0 || source.pulse <= 0.0 {
            return Err(FieldError::UndefinedCoreRadius);
        }
        let radius = (source.pulse / source.mass) * INNER_RADIUS_RATIO;
        if !radius.is_finite() || radius <= 0.0 {
            return Err(FieldError::UndefinedCoreRadius);
        }
        Ok(radius)
    }

    /// Evaluate distinct gravitational and levitational fields.
    ///
    /// r >= R: original inverse-square/inverse-cube magnitudes.
    /// 0 <= r < R: magnitudes decrease linearly from their value at R to zero
    /// at the center, with the same radial directions.
    ///
    /// The interior vector calculation uses displacement/R directly, avoiding
    /// an additional division by r. R = 0.001*(p/m) is provisional.
    pub fn field_at(
        source: &ParticleState,
        observation: Vector3,
        constants: &ModelConstants,
    ) -> Result<SourceFields, FieldError> {
        source.validate()?;
        constants.validate()?;
        if !observation.is_finite() {
            return Err(ValidationError::NotFinite("observation").into());
        }
        let dx = observation.x - source.position.x;
        let dy = observation.y - source.position.y;
        let dz = observation.z - source.position.z;
        if !dx.is_finite() || !dy.is_finite() || !dz.is_finite() {
            return Err(FieldError::NonFiniteResult);
        }
        let distance = dx.hypot(dy).hypot(dz);
        if !distance.is_finite() { return Err(FieldError::NonFiniteResult); }

        let radius = inner_radius(source)?;
        let boundary_inv = 1.0 / radius;
        let (gravitational_magnitude, levitational_magnitude, dir) =
            if distance < radius {
                // Boundary magnitudes times r/R; directions are displacement/r,
                // thus the vector components are boundary magnitude * displacement/R.
                let g_boundary = (constants.g * source.mass * boundary_inv) * boundary_inv;
                let l_boundary = ((constants.g * source.pulse * boundary_inv) * boundary_inv) * boundary_inv;
                let fraction = distance / radius;
                (
                    g_boundary * fraction,
                    l_boundary * fraction,
                    Vector3::new(dx * boundary_inv, dy * boundary_inv, dz * boundary_inv),
                )
            } else {
                let inv_r = 1.0 / distance;
                (
                    (constants.g * source.mass * inv_r) * inv_r,
                    ((constants.g * source.pulse * inv_r) * inv_r) * inv_r,
                    Vector3::new(dx * inv_r, dy * inv_r, dz * inv_r),
                )
            };
        if !gravitational_magnitude.is_finite() || !levitational_magnitude.is_finite() {
            return Err(FieldError::NonFiniteResult);
        }

        // Inside the core, dir already contains r/R. Use the boundary
        // magnitudes for vectors, rather than multiplying by r/R twice.
        let (g_vector_scale, l_vector_scale) = if distance < radius {
            let g_boundary = (constants.g * source.mass * boundary_inv) * boundary_inv;
            let l_boundary = ((constants.g * source.pulse * boundary_inv) * boundary_inv) * boundary_inv;
            (g_boundary, l_boundary)
        } else {
            (gravitational_magnitude, levitational_magnitude)
        };
        let gravitational = Vector3::new(
            -g_vector_scale * dir.x, -g_vector_scale * dir.y, -g_vector_scale * dir.z,
        );
        let levitational = Vector3::new(
            l_vector_scale * dir.x, l_vector_scale * dir.y, l_vector_scale * dir.z,
        );
        if !gravitational.is_finite() || !levitational.is_finite() {
            return Err(FieldError::NonFiniteResult);
        }
        Ok(SourceFields {
            distance,
            gravitational_magnitude,
            levitational_magnitude,
            gravitational: GravitationalField(gravitational),
            levitational: LevitationalField(levitational),
        })
    }

    /// Sum source fields at the position of one particle, skipping its own
    /// source contribution by index. Coincident *other* particles still exert
    /// their regularized zero-at-center fields; collision rules remain separate.
    pub fn fields_for_particle(
        sources: &[ParticleState],
        target_index: usize,
        constants: &ModelConstants,
    ) -> Result<FieldTotals, FieldError> {
        let target = sources.get(target_index).ok_or(FieldError::InvalidTargetIndex)?;
        target.validate()?;
        constants.validate()?;
        let mut result = FieldTotals::ZERO;
        for (index, source) in sources.iter().enumerate() {
            if index == target_index { continue; }
            let contribution = field_at(source, target.position, constants)?;
            result.gravitational = GravitationalField(sum_vectors(
                result.gravitational.0, contribution.gravitational.0,
            )?);
            result.levitational = LevitationalField(sum_vectors(
                result.levitational.0, contribution.levitational.0,
            )?);
        }
        Ok(result)
    }

    /// Query fields at a MOVING target's current position while excluding
    /// its own source by stable index. Other source particles are treated as
    /// fixed during this one-target reference integration.
    pub fn fields_for_target_state(
        sources: &[ParticleState],
        target_index: usize,
        target: &ParticleState,
        constants: &ModelConstants,
    ) -> Result<FieldTotals, FieldError> {
        if target_index >= sources.len() {
            return Err(FieldError::InvalidTargetIndex);
        }
        target.validate()?;
        constants.validate()?;
        let mut result = FieldTotals::ZERO;
        for (index, source) in sources.iter().enumerate() {
            if index == target_index {
                continue;
            }
            let f = field_at(source, target.position, constants)?;
            result.gravitational = GravitationalField(sum_vectors(
                result.gravitational.0, f.gravitational.0,
            )?);
            result.levitational = LevitationalField(sum_vectors(
                result.levitational.0, f.levitational.0,
            )?);
        }
        Ok(result)
    }

    fn sum_vectors(a: Vector3, b: Vector3) -> Result<Vector3, FieldError> {
        let sum = Vector3::new(a.x + b.x, a.y + b.y, a.z + b.z);
        if !sum.is_finite() { return Err(FieldError::NonFiniteResult); }
        Ok(sum)
    }

    /// Sum gravitational contributions with gravitational contributions, and
    /// levitational contributions with levitational contributions, at one point.
    /// No response of any target Escentron is calculated here.
    pub fn fields_at(
        sources: &[ParticleState],
        observation: Vector3,
        constants: &ModelConstants,
    ) -> Result<FieldTotals, FieldError> {
        constants.validate()?;
        if !observation.is_finite() {
            return Err(ValidationError::NotFinite("observation").into());
        }
        let mut result = FieldTotals::ZERO;
        for source in sources {
            let contribution = field_at(source, observation, constants)?;
            result.gravitational = GravitationalField(sum_vectors(
                result.gravitational.0, contribution.gravitational.0,
            )?);
            result.levitational = LevitationalField(sum_vectors(
                result.levitational.0, contribution.levitational.0,
            )?);
        }
        Ok(result)
    }

    #[cfg(test)]
    mod tests {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/field.rs"));
    }
}

pub(crate) mod particle {
    //! State variables of one Escentron.
    use crate::{validation::{finite_positive, ValidationError}, Vector3};

    /// Instantaneous state. No acceleration or integration step is implied.
    /// 'mass' (m) and 'pulse' (p) are the model's named physical parameters.
    /// Their physical units and calibration remain to be established.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct ParticleState {
        pub position: Vector3,
        pub velocity: Vector3,
        pub mass: f64,
        pub pulse: f64,
    }

    impl ParticleState {
        pub fn validate(&self) -> Result<(), ValidationError> {
            if !self.position.is_finite() { return Err(ValidationError::NotFinite("position")); }
            if !self.velocity.is_finite() { return Err(ValidationError::NotFinite("velocity")); }
            finite_positive(self.mass, "mass")?;
            finite_positive(self.pulse, "pulse")?;
            Ok(())
        }

        /// X = p/m for a valid Escentron. Invalid states return None.
        pub fn neutral_radius(&self) -> Option<f64> {
            if self.validate().is_err() { return None; }
            let radius = self.pulse / self.mass;
            if radius.is_finite() && radius > 0.0 { Some(radius) } else { None }
        }
    }

    #[cfg(test)]
    mod tests {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/particle.rs"));
    }
}

pub(crate) mod response {
    //! Target-response boundary between independent field types and acceleration.
    //!
    //! The physical induction laws are not specified in this module. A caller must
    //! provide an explicit implementation of each response law. The resulting
    //! accelerations, unlike the heterogeneous fields, may be vector-summed.
    use crate::{energy_of, FieldTotals, ModelConstants, ParticleState, ValidationError, Vector3};

    /// Errors in acceleration contributions or the response calculation.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ResponseError {
        InvalidTarget(ValidationError),
        InvalidField(&'static str),
        InvalidAcceleration(&'static str),
        NonFiniteTotal,
        EnergyExceedsCosmic,
    }

    impl std::fmt::Display for ResponseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::InvalidTarget(e) => write!(f, "invalid target: {e}"),
                Self::InvalidField(name) => write!(f, "non-finite {name} field"),
                Self::InvalidAcceleration(name) => write!(f, "non-finite {name} acceleration"),
                Self::NonFiniteTotal => write!(f, "non-finite total acceleration"),
                Self::EnergyExceedsCosmic => write!(f, "field-specific energy exceeds cosmic energy"),
            }
        }
    }
    impl std::error::Error for ResponseError {}


    /// Fraction of a target Escentron's cosmic energy stored as gravitational energy.
    ///
    /// alpha_g = E_g / E_c = (k_g * m) / E_c.
    ///
    /// This is one *partial* response coefficient, not a complete acceleration
    /// law. The input state must respect the energy budget: E_g <= E_c.
    /// No clamping or normalization is performed.
    pub fn gravitational_response_coefficient(
        target: &ParticleState,
        constants: &ModelConstants,
    ) -> Result<f64, ResponseError> {
        let energy = energy_of(target, constants).map_err(ResponseError::InvalidTarget)?;
        if energy.gravitational > constants.cosmic_energy {
            return Err(ResponseError::EnergyExceedsCosmic);
        }
        let coefficient = energy.gravitational / constants.cosmic_energy;
        if !coefficient.is_finite() {
            return Err(ResponseError::InvalidAcceleration("gravitational coefficient"));
        }
        Ok(coefficient)
    }

    /// Scale a separately determined *proposed* gravitational acceleration
    /// by the target's gravitational energy fraction.
    ///
    /// This is NOT a conversion from gravitational field to acceleration.
    /// It does not implement the direction-dependent energy transfer, velocity
    /// constraint, timestep, or remaining response rules.
    pub fn scale_gravitational_acceleration(
        target: &ParticleState,
        constants: &ModelConstants,
        proposed: Vector3,
    ) -> Result<Vector3, ResponseError> {
        if !proposed.is_finite() {
            return Err(ResponseError::InvalidAcceleration("proposed gravitational"));
        }
        let alpha = gravitational_response_coefficient(target, constants)?;
        let scaled = Vector3::new(
            proposed.x * alpha,
            proposed.y * alpha,
            proposed.z * alpha,
        );
        if !scaled.is_finite() {
            return Err(ResponseError::InvalidAcceleration("scaled gravitational"));
        }
        Ok(scaled)
    }


    /// Fraction of a target Escentron's cosmic energy stored as levitational energy.
    ///
    /// alpha_l = E_l / E_c = (k_l * p) / E_c.
    /// This is a partial response coefficient, not a complete acceleration law.
    /// No clamping or energy redistribution is performed.
    pub fn levitational_response_coefficient(
        target: &ParticleState,
        constants: &ModelConstants,
    ) -> Result<f64, ResponseError> {
        let energy = energy_of(target, constants).map_err(ResponseError::InvalidTarget)?;
        if energy.levitational > constants.cosmic_energy {
            return Err(ResponseError::EnergyExceedsCosmic);
        }
        let coefficient = energy.levitational / constants.cosmic_energy;
        if !coefficient.is_finite() {
            return Err(ResponseError::InvalidAcceleration("levitational coefficient"));
        }
        Ok(coefficient)
    }

    /// Apply the target's levitational energy fraction to an independently
    /// determined *proposed* levitational acceleration.
    /// This does not convert a field to acceleration or impose a speed limit.
    pub fn scale_levitational_acceleration(
        target: &ParticleState,
        constants: &ModelConstants,
        proposed: Vector3,
    ) -> Result<Vector3, ResponseError> {
        if !proposed.is_finite() {
            return Err(ResponseError::InvalidAcceleration("proposed levitational"));
        }
        let alpha = levitational_response_coefficient(target, constants)?;
        let scaled = Vector3::new(
            proposed.x * alpha,
            proposed.y * alpha,
            proposed.z * alpha,
        );
        if !scaled.is_finite() {
            return Err(ResponseError::InvalidAcceleration("scaled levitational"));
        }
        Ok(scaled)
    }

    /// Placeholder contract for target-specific physical induction rules.
    /// Implementations will be supplied only after their equations are agreed.
    /// Receiving two distinct field types prevents implicit field addition.
    pub trait ResponseLaw {
        fn from_gravitational(
            &self,
            target: &ParticleState,
            field: crate::GravitationalField,
        ) -> Vector3;

        fn from_levitational(
            &self,
            target: &ParticleState,
            field: crate::LevitationalField,
        ) -> Vector3;
    }

    /// Confirmed Simplification Theory response: raw field vectors are the
    /// unscaled acceleration proposals. The coupling layer subsequently applies
    /// E_g/E_c to gravity and E_l/E_c to levitation, separately and once.
    ///
    /// This is an explicit model assumption, not an experimentally validated
    /// relation. No additional field-to-acceleration coefficient is introduced.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct DirectFieldResponse;

    impl ResponseLaw for DirectFieldResponse {
        fn from_gravitational(
            &self,
            _target: &ParticleState,
            field: crate::GravitationalField,
        ) -> Vector3 {
            field.0
        }

        fn from_levitational(
            &self,
            _target: &ParticleState,
            field: crate::LevitationalField,
        ) -> Vector3 {
            field.0
        }
    }

    /// Separate acceleration contributions, plus their vector sum.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct AccelerationBreakdown {
        pub gravitational: Vector3,
        pub levitational: Vector3,
        pub total: Vector3,
    }

    /// Apply a specified target-response law to previously computed field totals.
    ///
    /// The law is deliberately external. This function does not assume a coupling
    /// coefficient, particle-to-particle transfer, timestep, or force formula.
    pub fn acceleration_from_fields(
        target: &ParticleState,
        fields: &FieldTotals,
        law: &impl ResponseLaw,
    ) -> Result<AccelerationBreakdown, ResponseError> {
        target.validate().map_err(ResponseError::InvalidTarget)?;
        if !fields.gravitational.0.is_finite() {
            return Err(ResponseError::InvalidField("gravitational"));
        }
        if !fields.levitational.0.is_finite() {
            return Err(ResponseError::InvalidField("levitational"));
        }
        let gravitational = law.from_gravitational(target, fields.gravitational);
        let levitational = law.from_levitational(target, fields.levitational);
        if !gravitational.is_finite() {
            return Err(ResponseError::InvalidAcceleration("gravitational"));
        }
        if !levitational.is_finite() {
            return Err(ResponseError::InvalidAcceleration("levitational"));
        }
        let total = Vector3::new(
            gravitational.x + levitational.x,
            gravitational.y + levitational.y,
            gravitational.z + levitational.z,
        );
        if !total.is_finite() { return Err(ResponseError::NonFiniteTotal); }
        Ok(AccelerationBreakdown { gravitational, levitational, total })
    }

    #[cfg(test)]
    mod tests {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/response.rs"));
    }
}

pub(crate) mod step {
    //! Evaluate a finite velocity step without committing any state mutation.
    //!
    //! This is a diagnostic/proposal stage, not the final Escentron integrator.
    //! Field coupling, response coefficients and acceleration limits are upstream.
    //! No energy transfers between distinct particles or directly between
    //! gravitational and levitational reservoirs.
    use crate::{
        acceleration_shares, allocate_kinetic_change, energy_of, transfer_energy,
        AllocationError, EnergyBreakdown, EnergyReservoir, ModelConstants,
        ParticleState, TransferError, ValidationError, Vector3,
    };

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum StepError {
        InvalidInput(ValidationError),
        InvalidTimeStep,
        NonFiniteResult,
        EnergyNotConservedInitially,
        Allocation(AllocationError),
        Transfer(TransferError),
        EnergyMismatch,
    }

    impl From<ValidationError> for StepError {
        fn from(e: ValidationError) -> Self { Self::InvalidInput(e) }
    }
    impl From<AllocationError> for StepError {
        fn from(e: AllocationError) -> Self { Self::Allocation(e) }
    }
    impl From<TransferError> for StepError {
        fn from(e: TransferError) -> Self { Self::Transfer(e) }
    }
    impl std::fmt::Display for StepError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self:?}")
        }
    }
    impl std::error::Error for StepError {}

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct StepProposal {
        pub proposed_velocity: Vector3,
        pub total_acceleration: Vector3,
        pub kinetic_change: f64,
        pub gravitational_kinetic_change: f64,
        pub levitational_kinetic_change: f64,
        pub proposed_energies: EnergyBreakdown,
    }

    /// Suggest a velocity and energy change over dt > 0.
    ///
    /// Computes v' = v + (a_g + a_l)*dt, then the exact finite-step difference
    /// delta_E_k = k_k (|v'|² - |v|²), with no sqrt or normalization.
    /// Allocation uses dot products with total acceleration and zeroes canceled
    /// contributions. The two assigned energy changes are individually exchanged
    /// with kinetic energy only. A step demanding unavailable energy is rejected.
    ///
    /// IMPORTANT: This is an unconstrained candidate, NOT the final time evolution
    /// or a guarantee that this acceleration is physically realizable. The full
    /// theory may require additional corrections before accepting a candidate.
    pub fn propose_step(
        target: &ParticleState,
        constants: &ModelConstants,
        gravitational_acceleration: Vector3,
        levitational_acceleration: Vector3,
        dt: f64,
    ) -> Result<StepProposal, StepError> {
        if !dt.is_finite() || dt <= 0.0 { return Err(StepError::InvalidTimeStep); }
        let initial = energy_of(target, constants)?;
        if constants.k_k <= 0.0 {
            return Err(StepError::InvalidInput(ValidationError::NonPositive("k_k")));
        }
        let tolerance = 1e-10 * constants.cosmic_energy.max(1.0);
        if (initial.total() - constants.cosmic_energy).abs() > tolerance {
            return Err(StepError::EnergyNotConservedInitially);
        }
        let shares = acceleration_shares(gravitational_acceleration, levitational_acceleration)?;
        let a = shares.total;
        let velocity = Vector3::new(
            target.velocity.x + a.x * dt,
            target.velocity.y + a.y * dt,
            target.velocity.z + a.z * dt,
        );
        if !velocity.is_finite() { return Err(StepError::NonFiniteResult); }
        let next_kinetic = constants.k_k * velocity.magnitude_squared();
        if !next_kinetic.is_finite() { return Err(StepError::NonFiniteResult); }
        let delta = next_kinetic - initial.kinetic;
        if !delta.is_finite() { return Err(StepError::NonFiniteResult); }
        let (g_delta, l_delta) = allocate_kinetic_change(shares, delta)?;
        let after_g = transfer_energy(initial, EnergyReservoir::Gravitational, g_delta)?;
        let after_l = transfer_energy(after_g, EnergyReservoir::Levitational, l_delta)?;
        // Compensate only numerical roundoff; actual energy validity is checked
        // rather than silently altering the proposed velocity or source shares.
        if (after_l.kinetic - next_kinetic).abs() > tolerance
            || (after_l.total() - initial.total()).abs() > tolerance
        {
            return Err(StepError::EnergyMismatch);
        }
        Ok(StepProposal {
            proposed_velocity: velocity,
            total_acceleration: a,
            kinetic_change: delta,
            gravitational_kinetic_change: g_delta,
            levitational_kinetic_change: l_delta,
            proposed_energies: after_l,
        })
    }

    #[cfg(test)]
    mod tests {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/step.rs"));
    }
}

pub(crate) mod work_path {
    //! Alternative RESEARCH hypothesis: each acceleration channel accounts for its
    //! own kinetic work. Not used by the current projection-based step integrator.
    //!
    //! With constant 3D field vectors, dE_i/dt=-2*k_k*v·a_i and
    //! a_i=(E_i/E_c)*f_i imply E_i(x)=E_i0 exp[-2*k_k/E_c*f_i·(x-x0)].
    //! This is a path-independent result only for CONSTANT field vectors.
    use crate::Vector3;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum WorkPathError {
        InvalidInput,
        PrecisionLimit,
        UnreachableKineticEnergy,
    }
    impl std::fmt::Display for WorkPathError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self:?}")
        }
    }
    impl std::error::Error for WorkPathError {}

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct WorkPathEnergies {
        pub gravitational: f64,
        pub levitational: f64,
        pub kinetic: f64,
    }

    /// Hypothetical channel-specific work accounting at a GIVEN displacement.
    ///
    /// Field vectors are fixed throughout the displacement. Does NOT solve
    /// position or velocity direction as a function of time. Does NOT alter the
    /// canonical energy allocation rule. Never inserts artificial minimum energy.
    pub fn constant_field_work_path(
        initial_gravitational: f64,
        initial_levitational: f64,
        initial_kinetic: f64,
        gravitational_field: Vector3,
        levitational_field: Vector3,
        displacement: Vector3,
        kinetic_coefficient: f64,
        cosmic_energy: f64,
    ) -> Result<WorkPathEnergies, WorkPathError> {
        let values = [
            initial_gravitational, initial_levitational, initial_kinetic,
            kinetic_coefficient, cosmic_energy,
        ];
        if !values.iter().all(|v| v.is_finite())
            || !gravitational_field.is_finite()
            || !levitational_field.is_finite()
            || !displacement.is_finite()
            || initial_gravitational <= 0.0
            || initial_levitational <= 0.0
            || initial_kinetic < 0.0
            || kinetic_coefficient <= 0.0
            || cosmic_energy <= 0.0
        {
            return Err(WorkPathError::InvalidInput);
        }
        let total = initial_gravitational + initial_levitational + initial_kinetic;
        if !total.is_finite() || (total - cosmic_energy).abs() >
            1e-10 * cosmic_energy.max(1.0) {
            return Err(WorkPathError::InvalidInput);
        }
        let dot = |a: Vector3, b: Vector3| a.x * b.x + a.y * b.y + a.z * b.z;
        let coefficient = -2.0 * kinetic_coefficient / cosmic_energy;
        let eg = initial_gravitational
            * (coefficient * dot(gravitational_field, displacement)).exp();
        let el = initial_levitational
            * (coefficient * dot(levitational_field, displacement)).exp();
        if !eg.is_finite() || !el.is_finite() || eg <= 0.0 || el <= 0.0 {
            return Err(WorkPathError::PrecisionLimit);
        }
        let ek = cosmic_energy - eg - el;
        if !ek.is_finite() {
            return Err(WorkPathError::PrecisionLimit);
        }
        if ek < 0.0 {
            return Err(WorkPathError::UnreachableKineticEnergy);
        }
        Ok(WorkPathEnergies {
            gravitational: eg, levitational: el, kinetic: ek,
        })
    }

    #[cfg(test)]
    mod tests {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/work_path.rs"));
    }
}
