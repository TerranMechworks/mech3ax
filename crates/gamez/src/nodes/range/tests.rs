use super::RangeI32;

#[test]
fn positive_range() {
    let range = RangeI32::new(0, 10, 1);
    let vec = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    assert_eq!(range.len(), vec.len() as i32);
    assert_eq!(range.into_iter().collect::<Vec<_>>(), vec);

    let range = RangeI32::new(0, 100, 10);
    let vec = vec![0, 10, 20, 30, 40, 50, 60, 70, 80, 90];
    assert_eq!(range.len(), vec.len() as i32);
    assert_eq!(range.into_iter().collect::<Vec<_>>(), vec);
}

#[test]
fn negative_range() {
    let range = RangeI32::new(10, 0, -1);
    let vec = vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
    assert_eq!(range.len(), vec.len() as i32);
    assert_eq!(range.into_iter().collect::<Vec<_>>(), vec);
}
