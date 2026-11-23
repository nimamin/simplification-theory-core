use escentron_core::*;

#[test]
fn opposite_fields_keep_signed_work_projections_and_respect_the_floor() {
    let model = ModelConstants { g: 1.0, k_g: 1.0, k_l: 1.0, k_k: 1.0, cosmic_energy: 8.0 };
    let mut particle = ParticleState { mass: 5.0, pulse: 3.0, position: Vector3::ZERO, velocity: Vector3::ZERO };
    let x = Vector3::new(1.0, 0.0, 0.0);
    let fields = FieldTotals { gravitational: GravitationalField(x), levitational: LevitationalField(x * -1.0) };
    let law = WorkLaw { minimum_energy: 1.0, half_strength: 1.0 };
    let work = particle.work_in(fields, &model, law).unwrap();
    assert_eq!(work.kinetic.0, x);
    assert_eq!((work.gravity_projection, work.levitation_projection), (2.0, -1.0));
    particle.mass = particle.pulse;
    let balanced = particle.work_in(fields, &model, law).unwrap();
    assert_eq!((balanced.gravity_projection, balanced.levitation_projection), (0.0, 0.0));
    particle.mass = law.minimum_energy;
    assert_eq!(particle.work_in(fields, &model, law).unwrap().gravity.0, Vector3::ZERO);
    assert_eq!(work_at_particle(&[particle], 0, &model, law).unwrap().kinetic.0, Vector3::ZERO);
}

#[test]
fn accelerating_braking_and_cancellation_balance_each_reservoir() {
    let x = Vector3::new(1.0, 0.0, 0.0);
    let fields = FieldTotals { gravitational: GravitationalField(x), levitational: LevitationalField(x * -1.0) };
    let law = WorkLaw { minimum_energy: 1.0, half_strength: 1.0 };
    for (speed, mass, pulse, next_mass, next_pulse, next_kinetic) in [
        (0.0, 5.0, 3.0, 3.0, 4.0, 1.0),
        (-2.0, 5.0, 3.0, 7.0, 2.0, 3.0),
        (-1.0, 5.0, 3.0, 7.0, 2.0, 0.0),
        (1.0, 3.0, 3.0, 3.0, 3.0, 1.0),
    ] {
        let model = ModelConstants { g: 1.0, k_g: 1.0, k_l: 1.0, k_k: 1.0, cosmic_energy: mass + pulse + speed * speed };
        let state = ParticleState { mass, pulse, position: Vector3::ZERO, velocity: x * speed };
        let next = state.advance_in(fields, &model, law, 0.25).unwrap();
        assert_eq!((next.mass, next.pulse), (next_mass, next_pulse));
        assert!((next.velocity.dot(next.velocity) - next_kinetic).abs() < 1e-12);
        assert_eq!(next.position, next.velocity * 0.25);
        let isolated = advance_escentrons(&[state], &model, law, 0.25).unwrap();
        assert_eq!(isolated[0].velocity, state.velocity);
    }
}
