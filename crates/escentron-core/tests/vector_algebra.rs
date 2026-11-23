use escentron_core::Vector3;

#[test]
fn vector_operations_preserve_direction_and_length() {
    let x = Vector3::new(1.0, 0.0, 0.0);
    let y = Vector3::new(0.0, 1.0, 0.0);
    let z = Vector3::new(0.0, 0.0, 1.0);
    assert_eq!(x.cross(y), z);
    assert_eq!(y.cross(x), z * -1.0);
    assert_eq!(x.dot(y), 0.0);
    assert_eq!((x * 3.0 + y * 4.0).length(), 5.0);
    assert_eq!((x + y) - y, x);
}
