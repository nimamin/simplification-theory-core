use super::*;
#[test]
fn squared_speed_uses_all_components() {
    assert_eq!(Vector3::new(2.0, 3.0, 6.0).magnitude_squared(), 49.0);
}
