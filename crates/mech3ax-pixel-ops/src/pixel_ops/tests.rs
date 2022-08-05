use super::LERP888;

#[test]
fn lerp888_smoke_test() {
    // black
    assert_eq!(LERP888[0b0000000000000000], 0x000000);
    // white
    assert_eq!(LERP888[0b1111111111111111], 0xFFFFFF);
    // red
    assert_eq!(LERP888[0b1111100000000000], 0xFF0000);
    // green
    assert_eq!(LERP888[0b0000011111100000], 0x00FF00);
    // blue
    assert_eq!(LERP888[0b0000000000011111], 0x0000FF);
    // red + green
    assert_eq!(LERP888[0b0000011111111111], 0x00FFFF);
    // red + blue
    assert_eq!(LERP888[0b1111100000011111], 0xFF00FF);
    // green + blue
    assert_eq!(LERP888[0b1111111111100000], 0xFFFF00);
}
