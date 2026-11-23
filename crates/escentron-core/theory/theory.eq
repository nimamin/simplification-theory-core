// Current Rust model. Scalar = f64, Vec3 = a three-dimensional vector.
// Vector primitives: dot(a,b), cross(a,b), length(a), vector +/- vector, vector * scalar.
equations! {
    neutral_radius(p: Scalar, m: Scalar) -> Scalar = p / m;
    gravitational_energy(k: Scalar, m: Scalar) -> Scalar = k * m;
    levitational_energy(k: Scalar, p: Scalar) -> Scalar = k * p;
    kinetic_energy(k: Scalar, v: Vec3) -> Scalar = k * dot(v, v);

    // Source fields at displacement d from the source, including the linear inner core.
    source_fields(d: Vec3, r: Scalar, core: Scalar, g: Scalar, m: Scalar, p: Scalar)
        -> (Scalar, Scalar, Vec3, Vec3) = {
        let inv = 1.0 / r.max(core);
        let direction = d * inv;
        let gravity = g * m * inv * inv;
        let levitation = g * p * inv * inv * inv;
        let fraction = if r < core { r / core } else { 1.0 };
        (gravity * fraction, levitation * fraction, direction * -gravity, direction * levitation)
    };

    response_fraction(energy: Scalar, cosmic: Scalar) -> Scalar = energy / cosmic;
    apply_response(field: Vec3, fraction: Scalar) -> Vec3 = field * fraction;
    resultant(gravity: Vec3, levitation: Vec3) -> Vec3 = gravity + levitation;

    // Signed projections of each acceleration on the resultant determine its share.
    gravity_share(g_proj: Scalar, l_proj: Scalar, total_squared: Scalar) -> Scalar =
        if g_proj <= 0.0 { 0.0 } else if l_proj <= 0.0 { 1.0 }
        else { (g_proj / total_squared).clamp(0.0, 1.0) };

    advance(value: Vec3, rate: Vec3, dt: Scalar) -> Vec3 = value + rate * dt;
    kinetic_change(next: Scalar, current: Scalar) -> Scalar = next - current;
    split_change(delta: Scalar, share: Scalar) -> (Scalar, Scalar) = {
        let gravity = delta * share;
        (gravity, delta - gravity)
    };
    transfer(source: Scalar, kinetic: Scalar, change: Scalar) -> (Scalar, Scalar) =
        (source - change, kinetic + change);
    from_energy(energy: Scalar, coefficient: Scalar) -> Scalar = energy / coefficient;
}
