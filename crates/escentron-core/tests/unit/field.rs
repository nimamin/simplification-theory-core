use super::*;

fn source_at(x: f64) -> ParticleState {
    ParticleState {
        position: Vector3::new(x, 0.0, 0.0),
        velocity: Vector3::ZERO,
        mass: 2.0,
        pulse: 6.0,
    }
}
fn constants() -> ModelConstants {
    ModelConstants {
        g: 1.0, k_g: 1.0, k_l: 1.0, k_k: 1.0, cosmic_energy: 100.0,
    }
}
fn approx(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-12, "{a} is not close to {b}");
}

#[test]
fn neutral_radius_has_equal_magnitudes_but_separate_fields() {
    let f = field_at(&source_at(0.0), Vector3::new(3.0, 0.0, 0.0), &constants()).unwrap();
    approx(f.gravitational_magnitude, 2.0 / 9.0);
    approx(f.levitational_magnitude, 2.0 / 9.0);
    approx(f.gravitational.0.x, -2.0 / 9.0);
    approx(f.levitational.0.x, 2.0 / 9.0);
}

#[test]
fn near_and_far_do_not_combine_field_types() {
    let near = field_at(&source_at(0.0), Vector3::new(1.0, 0.0, 0.0), &constants()).unwrap();
    approx(near.gravitational_magnitude, 2.0);
    approx(near.levitational_magnitude, 6.0);
    let far = field_at(&source_at(0.0), Vector3::new(4.0, 0.0, 0.0), &constants()).unwrap();
    approx(far.gravitational_magnitude, 2.0 / 16.0);
    approx(far.levitational_magnitude, 6.0 / 64.0);
}

#[test]
fn same_kind_fields_superpose_vectorially() {
    let sources = [source_at(-1.0), source_at(-2.0)];
    let totals = fields_at(&sources, Vector3::ZERO, &constants()).unwrap();
    approx(totals.gravitational.0.x, -2.5);
    approx(totals.levitational.0.x, 6.75);
}

#[test]
fn opposite_sources_cancel_each_field_independently() {
    let sources = [source_at(-1.0), source_at(1.0)];
    let totals = fields_at(&sources, Vector3::ZERO, &constants()).unwrap();
    approx(totals.gravitational.0.x, 0.0);
    approx(totals.levitational.0.x, 0.0);
}

#[test]
fn field_directions_are_three_dimensional() {
    let f = field_at(&source_at(0.0), Vector3::new(0.0, -4.0, 0.0), &constants()).unwrap();
    assert!(f.gravitational.0.y > 0.0);
    assert!(f.levitational.0.y < 0.0);
    approx(f.gravitational.0.x, 0.0);
}

#[test]
fn center_fields_are_zero() {
    assert_eq!(
        field_at(&source_at(0.0), Vector3::ZERO, &constants()),
        Ok(SourceFields {
            distance: 0.0,
            gravitational_magnitude: 0.0,
            levitational_magnitude: 0.0,
            gravitational: GravitationalField(Vector3::ZERO),
            levitational: LevitationalField(Vector3::ZERO),
        })
    );
}

#[test]
fn non_finite_observation_is_rejected() {
    assert_eq!(
        fields_at(&[], Vector3::new(f64::NAN, 0.0, 0.0), &constants()),
        Err(FieldError::InvalidInput(ValidationError::NotFinite("observation")))
    );
}

#[test]
fn no_sources_produce_two_zero_fields() {
    assert_eq!(
        fields_at(&[], Vector3::ZERO, &constants()).unwrap(),
        FieldTotals::ZERO
    );
}

#[test]
fn zero_g_produces_two_zero_fields() {
    let mut c = constants();
    c.g = 0.0;
    assert_eq!(
        fields_at(&[source_at(1.0)], Vector3::ZERO, &c).unwrap(),
        FieldTotals::ZERO
    );
}

#[test]
fn inner_radius_is_one_thousandth_of_neutral_radius() {
    approx(inner_radius(&source_at(0.0)).unwrap(), 0.003);
}

#[test]
fn interior_magnitudes_decrease_linearly() {
    let r = inner_radius(&source_at(0.0)).unwrap();
    let boundary = field_at(&source_at(0.0), Vector3::new(r, 0.0, 0.0), &constants()).unwrap();
    let midpoint = field_at(&source_at(0.0), Vector3::new(r/2.0, 0.0, 0.0), &constants()).unwrap();
    approx(midpoint.gravitational_magnitude / boundary.gravitational_magnitude, 0.5);
    approx(midpoint.levitational_magnitude / boundary.levitational_magnitude, 0.5);
    approx(midpoint.gravitational.0.x / boundary.gravitational.0.x, 0.5);
    approx(midpoint.levitational.0.x / boundary.levitational.0.x, 0.5);
}

#[test]
fn undefined_neutral_radius_is_an_explicit_error() {
    let mut source = source_at(0.0);
    source.mass = 0.0;
    assert_eq!(field_at(&source, Vector3::ZERO, &constants()), Err(FieldError::InvalidInput(ValidationError::NonPositive("mass"))));
}

#[test]
fn zero_pulse_source_is_invalid() {
    let mut source = source_at(0.0);
    source.pulse = 0.0;
    assert_eq!(
        field_at(&source, Vector3::ZERO, &constants()),
        Err(FieldError::InvalidInput(ValidationError::NonPositive("pulse")))
    );
}

#[test]
fn particle_does_not_include_its_own_fields() {
    let sources = [source_at(0.0), source_at(1.0)];
    let totals = fields_for_particle(&sources, 0, &constants()).unwrap();
    let expected = field_at(&sources[1], sources[0].position, &constants()).unwrap();
    assert_eq!(totals.gravitational, expected.gravitational);
    assert_eq!(totals.levitational, expected.levitational);
}

#[test]
fn invalid_particle_index_is_rejected() {
    assert_eq!(
        fields_for_particle(&[source_at(0.0)], 1, &constants()),
        Err(FieldError::InvalidTargetIndex)
    );
}

#[test]
fn moving_target_excludes_own_source_even_after_displacement() {
    let sources = [source_at(0.0), source_at(5.0)];
    let mut moved = sources[0];
    moved.position = Vector3::new(1.0, 0.0, 0.0);
    let actual = fields_for_target_state(&sources, 0, &moved, &constants()).unwrap();
    let expected = field_at(&sources[1], moved.position, &constants()).unwrap();
    assert_eq!(actual.gravitational, expected.gravitational);
    assert_eq!(actual.levitational, expected.levitational);
}
