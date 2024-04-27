use super::*;

#[test]
fn test_replacing_inserts() {
    let mut map = BiMap::default();

    let (old_right, old_left) = map.insert(1, 2);
    assert_eq!(map.len(), 1);
    assert_eq!(old_right, None);
    assert_eq!(old_left, None);

    let (old_right, old_left) = map.insert(2, 3);
    assert_eq!(map.len(), 2);
    assert_eq!(old_right, None);
    assert_eq!(old_left, None);

    let (old_right, old_left) = map.insert(2, 4);
    assert_eq!(map.len(), 2);
    assert_eq!(old_right, Some(3));
    assert_eq!(old_left, None);

    let (old_right, old_left) = map.insert(1, 4);
    assert_eq!(map.len(), 1);
    assert_eq!(old_right, Some(2));
    assert_eq!(old_left, Some(2));
}