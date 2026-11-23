pub trait Moving {
    fn position(&self) -> Vector3;
    fn velocity(&self) -> Vector3;
}

pub trait FieldSource: Moving {
    fn fields_at(&self, point: Vector3, model: &ModelConstants) -> Result<SourceFields, FieldError>;
}

pub trait Escentron: FieldSource {
    fn work_in(&self, fields: FieldTotals, model: &ModelConstants, law: WorkLaw)
        -> Result<ParticleWork, FieldError>;
}

impl Escentron for ParticleState {
    fn work_in(&self, fields: FieldTotals, model: &ModelConstants, law: WorkLaw)
        -> Result<ParticleWork, FieldError>
    {
        self.validate()?;
        model.validate()?;
        law.validate()?;

        let mass_energy = model.k_g * self.mass;
        let pulse_energy = model.k_l * self.pulse;
        let gravity = fields.gravitational.work(mass_energy, law)?;
        let levitation = fields.levitational.work(pulse_energy, law)?;
        let kinetic = gravity + levitation;
        let direction = kinetic.0.unit_or_zero()?;

        Ok(ParticleWork {
            gravity, levitation, kinetic,
            gravity_projection: gravity.0.dot(direction),
            levitation_projection: levitation.0.dot(direction),
        })
    }
}

impl GravitationalField {
    fn work(self, mass_energy: f64, law: WorkLaw) -> Result<GravityWork, FieldError> {
        law.corrected_work(self.0, mass_energy).map(GravityWork)
    }
}

impl LevitationalField {
    fn work(self, pulse_energy: f64, law: WorkLaw) -> Result<LevitationWork, FieldError> {
        law.corrected_work(self.0, pulse_energy).map(LevitationWork)
    }
}

impl WorkLaw {
    fn corrected_work(self, field: Vector3, energy: f64) -> Result<Vector3, FieldError> {
        let strength = field.length();
        if !energy.is_finite() || !strength.is_finite() { return Err(FieldError::NonFiniteResult); }
        let available = (energy - self.minimum_energy).max(0.0);
        let fraction = self.work_fraction(strength);
        Ok(field.unit_or_zero()? * (available * fraction))
    }
}

impl ParticleState {
    pub fn advance_in(&self, fields: FieldTotals, model: &ModelConstants, law: WorkLaw, dt: f64)
        -> Result<Self, WorkStepError>
    {
        let work = self.work_in(fields, model, law)?;
        validate_step(self, model, law, dt)?;
        let magnitude = work.kinetic.0.length();
        if magnitude == 0.0 {
            return self.drift(self.velocity, dt);
        }

        let initial_energy = crate::energy_of(self, model)?.kinetic;
        let initial = self.velocity.unit_or_zero()? * initial_energy;
        let next = initial + work.kinetic.0;
        let next_energy = next.length();
        let velocity = next.unit_or_zero()? * (next_energy.sqrt() / model.k_k.sqrt());

        // (|next| - |initial|) / |work|, rationalized for nearly cancelling work.
        let direction = work.kinetic.0.unit_or_zero()?;
        let scale = next_energy.max(initial_energy);
        let denominator = next_energy / scale + initial_energy / scale;
        let energy_per_work = (direction.dot(next / scale) + direction.dot(initial / scale))
            / denominator;
        let gravity_change = work.gravity_projection * energy_per_work;
        let levitation_change = work.levitation_projection * energy_per_work;
        let mass = (model.k_g * self.mass - gravity_change) / model.k_g;
        let pulse = (model.k_l * self.pulse - levitation_change) / model.k_l;

        let moved = self.drift(velocity, dt)?;
        let result = Self { mass, pulse, ..moved };
        validate_step(&result, model, law, dt)?;
        Ok(result)
    }
}
