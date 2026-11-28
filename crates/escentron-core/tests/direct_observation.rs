use escentron_core::{FieldTotals, GravitationalField, LevitationalField, ParticleState, Vector3};
use escentron_core::theory_v2::{V2Constants, direct::{DirectModel, observed_energy}};

#[test]
fn observe_energy_without_enforcing_conservation() {
    let c = V2Constants { k_g: 1.0, k_l: 1.0, k_k: 1.0, c_g: 1.0, c_l: 1.0,
        energy_floor: 1e-6, reference_field: 1.0, k_w: 2.0 };
    let initial = ParticleState { position: Vector3::ZERO, velocity: Vector3::ZERO,
        mass: 1.0, pulse: 1.0 };
    let model = DirectModel::new(&initial, c).unwrap();
    println!("dt,time,mass,pulse,vx,Eg,El,Ek,total,delta_total");
    for dt in [0.001_f64, 0.0005, 0.00025] {
        let mut state = initial;
        let steps = (2.0 / dt).round() as usize;
        let interval = (0.25 / dt).round() as usize;
        for i in 0..=steps {
            if i % interval == 0 {
                let e = observed_energy(&state, &c);
                println!("{dt},{},{},{},{},{},{},{},{},{}", i as f64 * dt,
                    state.mass, state.pulse, state.velocity.x, e.gravitational,
                    e.levitational, e.kinetic, e.total(), e.total() - model.initial_cosmic_energy);
            }
            if i == steps { break; }
            let direction = if i < steps / 2 { 1.0 } else { -1.0 };
            let fields = FieldTotals {
                gravitational: GravitationalField(Vector3::new(direction, 0.0, 0.0)),
                levitational: LevitationalField(Vector3::ZERO),
            };
            state = model.advance(&state, fields, dt).unwrap();
        }
    }
}

#[test]
fn direct_floor_allows_replenishment() {
    let c = V2Constants { k_g: 1.0, k_l: 1.0, k_k: 1.0, c_g: 1.0, c_l: 1.0,
        energy_floor: 1e-6, reference_field: 1.0, k_w: 2.0 };
    let initial = ParticleState { position: Vector3::ZERO, velocity: Vector3::ZERO,
        mass: 1.0, pulse: 1.0 };
    let model = DirectModel::new(&initial, c).unwrap();
    let field = |x| FieldTotals { gravitational: GravitationalField(Vector3::new(x, 0.0, 0.0)),
        levitational: LevitationalField(Vector3::ZERO) };
    let saturated = model.advance(&initial, field(1000.0), 0.001).unwrap();
    assert!((saturated.mass - model.mass_min).abs() < 1e-12);
    let held = model.advance(&saturated, field(1000.0), 0.001).unwrap();
    assert!((held.velocity.x - saturated.velocity.x).abs() < 1e-12);
    assert!((held.velocity.y - saturated.velocity.y).abs() < 1e-12);
    assert!((held.velocity.z - saturated.velocity.z).abs() < 1e-12);
    let braking = model.advance(&held, field(-1000.0), 0.001).unwrap();
    assert!(braking.mass > held.mass);
    assert!(braking.velocity.x < held.velocity.x);
}
