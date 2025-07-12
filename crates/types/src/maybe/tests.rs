use crate::Bool32;

#[test]
fn partial_ord() {
    let m = Bool32::new(42);
    assert_eq!(m.partial_cmp(&42), 42.partial_cmp(&42));
    assert_eq!(m.partial_cmp(&41), 42.partial_cmp(&41));
    assert_eq!(m.partial_cmp(&43), 42.partial_cmp(&43));

    assert_eq!(42.partial_cmp(&m), 42.partial_cmp(&42));
    assert_eq!(41.partial_cmp(&m), 41.partial_cmp(&42));
    assert_eq!(43.partial_cmp(&m), 43.partial_cmp(&42));
}
