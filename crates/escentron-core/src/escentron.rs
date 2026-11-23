use crate::{field_at, fields_for_particle, FieldError, FieldTotals, GravitationalField,
    LevitationalField, ModelConstants, ParticleState, SourceFields, Vector3};

#[derive(Clone, Copy, Debug)]
pub struct WorkLaw {
    pub minimum_energy: f64,
    pub half_strength: f64,
}

impl WorkLaw {
    fn validate(self) -> Result<(), FieldError> {
        crate::validation::finite_positive(self.minimum_energy, "minimum_energy")?;
        crate::validation::finite_positive(self.half_strength, "half_strength")?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GravityWork(pub Vector3);
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LevitationWork(pub Vector3);
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KineticWork(pub Vector3);

impl std::ops::Add<LevitationWork> for GravityWork {
    type Output = KineticWork;
    fn add(self, rhs: LevitationWork) -> KineticWork { KineticWork(self.0 + rhs.0) }
}

#[derive(Clone, Copy, Debug)]
pub struct ParticleWork {
    pub gravity: GravityWork,
    pub levitation: LevitationWork,
    pub kinetic: KineticWork,
    pub gravity_projection: f64,
    pub levitation_projection: f64,
}

impl Vector3 {
    fn unit_or_zero(self) -> Result<Self, FieldError> {
        let norm = self.length();
        if !norm.is_finite() { return Err(FieldError::NonFiniteResult); }
        if norm == 0.0 { return Ok(Self::ZERO); }
        Ok(Self::new(self.x / norm, self.y / norm, self.z / norm))
    }
}

include!("../theory/escentron.eq");

impl Moving for ParticleState {
    fn position(&self) -> Vector3 { self.position }
    fn velocity(&self) -> Vector3 { self.velocity }
}

impl FieldSource for ParticleState {
    fn fields_at(&self, point: Vector3, model: &ModelConstants) -> Result<SourceFields, FieldError> {
        field_at(self, point, model)
    }
}

pub fn work_at_particle(particles: &[ParticleState], target: usize, model: &ModelConstants, law: WorkLaw)
    -> Result<ParticleWork, FieldError>
{
    let fields = fields_for_particle(particles, target, model)?;
    particles[target].work_in(fields, model, law)
}

impl WorkLaw {
    // strength / (strength + half_strength), evaluated without overflow.
    fn work_fraction(self, strength: f64) -> f64 {
        if strength <= self.half_strength {
            let ratio = strength / self.half_strength;
            ratio / (1.0 + ratio)
        } else {
            1.0 / (1.0 + self.half_strength / strength)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkStepError {
    Field(FieldError),
    InvalidInput(crate::ValidationError),
    BelowEnergyFloor,
    CosmicEnergyMismatch,
}

impl From<FieldError> for WorkStepError {
    fn from(error: FieldError) -> Self { Self::Field(error) }
}

impl From<crate::ValidationError> for WorkStepError {
    fn from(error: crate::ValidationError) -> Self { Self::InvalidInput(error) }
}

fn validate_step(state: &ParticleState, model: &ModelConstants, law: WorkLaw, dt: f64)
    -> Result<(), WorkStepError>
{
    use crate::validation::finite_positive;
    finite_positive(dt, "dt")?;
    finite_positive(model.k_k, "k_k")?;
    finite_positive(model.k_g, "k_g")?;
    finite_positive(model.k_l, "k_l")?;
    let energy = crate::energy_of(state, model)?;
    if energy.gravitational < law.minimum_energy || energy.levitational < law.minimum_energy {
        return Err(WorkStepError::BelowEnergyFloor);
    }
    if (energy.total() / model.cosmic_energy - 1.0).abs() > 1e-10 {
        return Err(WorkStepError::CosmicEnergyMismatch);
    }
    Ok(())
}

impl ParticleState {
    fn drift(&self, velocity: Vector3, dt: f64) -> Result<Self, WorkStepError> {
        let position = self.position + velocity * dt;
        let moved = Self { position, velocity, ..*self };
        moved.validate()?;
        Ok(moved)
    }
}

/// One discrete work update per particle. All fields use the same initial snapshot.
pub fn advance_escentrons(particles: &[ParticleState], model: &ModelConstants, law: WorkLaw, dt: f64)
    -> Result<Vec<ParticleState>, WorkStepError>
{
    particles.iter().enumerate().map(|(index, particle)| {
        let fields = fields_for_particle(particles, index, model)?;
        particle.advance_in(fields, model, law, dt)
    }).collect()
}
