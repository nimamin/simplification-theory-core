use escentron_core::{FieldTotals, GravitationalField, LevitationalField, ParticleState, Vector3};
use escentron_core::theory_v2::{advance, advance_system, energies, V2Constants};

fn constants() -> V2Constants {
    V2Constants { k_g: 1.0, k_l: 1.0, k_k: 1.0, c_g: 1.0, c_l: 1.0,
        energy_floor: 1e-6, reference_field: 1.0, k_w: 2.0 }
}
fn particle(x: f64) -> ParticleState {
    ParticleState { position: Vector3::new(x, 0.0, 0.0),
        velocity: Vector3::ZERO, mass: 1.0, pulse: 1.0 }
}
fn field(g: f64, l: f64) -> FieldTotals {
    FieldTotals { gravitational: GravitationalField(Vector3::new(g, 0.0, 0.0)),
        levitational: LevitationalField(Vector3::new(l, 0.0, 0.0)) }
}
fn check_energy(s: &ParticleState, c: &V2Constants, expected: f64) {
    let e = energies(s, c).unwrap();
    assert!((e.total() - expected).abs() < 1e-9);
    assert!(e.gravitational >= c.energy_floor - 1e-12);
    assert!(e.levitational >= c.energy_floor - 1e-12);
}

#[test]
fn v2_001_saturation_reversal() {
    let c = constants();
    let dt = 0.001;
    let mut s = particle(0.0);
    let mut steps = 0;
    while energies(&s, &c).unwrap().gravitational - c.energy_floor > 1e-4 && steps < 100_000 {
        s = advance(&s, field(1.0, 0.0), &c, dt).unwrap();
        check_energy(&s, &c, 2.0);
        steps += 1;
    }
    assert!(steps < 100_000, "no saturation");
    let mut braking = 0;
    while s.velocity.x > 0.0 && braking < 100_000 {
        s = advance(&s, field(-1.0, 0.0), &c, dt).unwrap();
        check_energy(&s, &c, 2.0);
        braking += 1;
    }
    assert!(braking < 100_000, "no reversal");
    assert!((braking as f64 - steps as f64).abs() * dt <= 0.02,
        "acceleration={}, braking={}", steps as f64 * dt, braking as f64 * dt);
    s = advance(&s, field(-1.0, 0.0), &c, dt).unwrap();
    assert!(s.velocity.x < 0.0);
}

#[test]
fn v2_001_exact_floor_reversal() {
    let c = constants();
    let dt = 0.001;
    let s = advance(&particle(0.0), field(1000.0, 0.0), &c, dt).unwrap();
    check_energy(&s, &c, 2.0);
    assert!((energies(&s, &c).unwrap().gravitational - c.energy_floor).abs() < 1e-12);
    let held = advance(&s, field(1000.0, 0.0), &c, dt).unwrap();
    assert!((held.velocity.x - s.velocity.x).abs() < 1e-12);
    let back = advance(&s, field(-1000.0, 0.0), &c, dt).unwrap();
    assert!(back.velocity.x < s.velocity.x, "exact saturation must permit braking");
    check_energy(&back, &c, 2.0);
    assert!(back.mass > s.mass);
}

#[test]
fn v2_002_symmetric_rebound() {
    let c = constants();
    for dt in [0.001, 0.0005, 0.00025] {
        let mut states = vec![particle(-5.0), particle(5.0)];
        let mut approached = false;
        let mut rebounded = false;
        let mut minimum = 10.0_f64;
        for step in 0..(100.0 / dt) as usize {
            states = advance_system(&states, &c, dt).unwrap();
            if step == 0 {
                assert!(states[0].velocity.x > 0.0 && states[1].velocity.x < 0.0, "initial attraction required");
            }
            let gap = states[1].position.x - states[0].position.x;
            minimum = minimum.min(gap);
            assert!(gap > 0.0, "particles crossed at dt={dt}");
            assert!((states[0].position.x + states[1].position.x).abs() < 1e-9);
            for s in &states { check_energy(s, &c, 2.0); }
            approached |= gap < 10.0 - 1e-6;
            if approached && states[0].velocity.x < 0.0 && states[1].velocity.x > 0.0 {
                rebounded = true;
                println!("dt={dt}, rebound_time={}, minimum_gap={minimum}", (step + 1) as f64 * dt);
                break;
            }
        }
        assert!(rebounded, "no rebound at dt={dt}; minimum gap={minimum}");
    }
}

#[test]
fn v2_003_free_motion() {
    let c = constants();
    let mut s = particle(0.0);
    s.velocity = Vector3::new(1.0, 0.0, 0.0);
    let n = advance(&s, FieldTotals::ZERO, &c, 0.5).unwrap();
    assert_eq!(n.velocity, s.velocity);
    assert_eq!(n.mass, s.mass);
    assert_eq!(n.pulse, s.pulse);
    assert_eq!(n.position.x, 0.5);
}

#[test]
fn v2_004_cancellation() {
    let s = particle(0.0);
    assert_eq!(advance(&s, field(1.0, -1.0), &constants(), 0.001).unwrap(), s);
}

#[test]
fn v2_005_energy_and_floors() {
    let c = constants();
    let mut s = particle(0.0);
    for i in 0..1000 {
        let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
        s = advance(&s, field(sign * 100.0, -sign * 50.0), &c, 0.001).unwrap();
        check_energy(&s, &c, 2.0);
    }
}

#[test]
fn v2_006_translation_rotation() {
    let c = constants();
    let a = vec![particle(-1.0), particle(1.0)];
    let b: Vec<_> = a.iter().map(|s| ParticleState {
        position: Vector3::new(3.0, s.position.x + 5.0, 7.0), ..*s
    }).collect();
    let x = advance_system(&a, &c, 0.001).unwrap();
    let y = advance_system(&b, &c, 0.001).unwrap();
    for (p, q) in x.iter().zip(y.iter()) {
        assert!((q.position.y - p.position.x - 5.0).abs() < 1e-12);
        assert!((q.position.x - 3.0).abs() < 1e-12);
        assert!((q.position.z - 7.0).abs() < 1e-12);
        assert!((q.velocity.y - p.velocity.x).abs() < 1e-12);
        assert!((q.mass - p.mass).abs() < 1e-12);
        assert!((q.pulse - p.pulse).abs() < 1e-12);
    }
}

#[test]
fn v2_007_time_refinement() {
    fn run(dt: f64) -> ParticleState {
        let c = constants();
        let mut s = particle(0.0);
        for _ in 0..(1.0 / dt).round() as usize {
            s = advance(&s, field(0.5, 0.0), &c, dt).unwrap();
        }
        s
    }
    let a = run(0.001);
    let b = run(0.0005);
    let d = run(0.00025);
    assert!((d.velocity.x - b.velocity.x).abs() < (b.velocity.x - a.velocity.x).abs());
    assert!((d.position.x - b.position.x).abs() < (b.position.x - a.position.x).abs());
}

#[test]
fn v2_008_opposing_work_cancels_before_allocation() {
    let c = constants();
    let initial = particle(0.0);
    let next = advance(&initial, field(6.0, -3.0), &c, 0.001).unwrap();
    let e = energies(&next, &c).unwrap();
    assert_eq!(next.pulse, initial.pulse);
    assert!((e.kinetic - 3.0 * c.k_w / 2.0_f64.sqrt() * 0.001).abs() < 1e-12);
    assert!((1.0 - e.gravitational - e.kinetic).abs() < 1e-12);
    check_energy(&next, &c, 2.0);
    let back = advance(&next, field(-6.0, 3.0), &c, 0.0001).unwrap();
    assert!(back.mass > next.mass);
    assert_eq!(back.pulse, next.pulse);
    check_energy(&back, &c, 2.0);
}

#[test]
fn v2_009_aligned_work_shares_and_reverse_dominance() {
    let c = constants();
    let initial = particle(0.0);
    let next = advance(&initial, field(6.0, 3.0), &c, 0.001).unwrap();
    let e = energies(&next, &c).unwrap();
    assert!((1.0 - e.gravitational - e.kinetic * 2.0 / 3.0).abs() < 1e-12);
    assert!((1.0 - e.levitational - e.kinetic / 3.0).abs() < 1e-12);
    let next = advance(&initial, field(3.0, -6.0), &c, 0.001).unwrap();
    assert_eq!(next.mass, initial.mass);
    assert!(next.pulse < initial.pulse && next.velocity.x < 0.0);
    check_energy(&next, &c, 2.0);
}
