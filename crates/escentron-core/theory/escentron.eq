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
