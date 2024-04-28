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

#[test]
fn test_get() {
    let mut map = BiMap::default();

    map.insert(1, 2);
    map.insert(2, 3);

    assert_eq!(map.get_right(&0), None);
    assert_eq!(map.get_left(&0), None);

    assert_eq!(map.get_right(&1), Some(&2));
    assert_eq!(map.get_right(&2), Some(&3));
    assert_eq!(map.get_left(&2), Some(&1));
    assert_eq!(map.get_left(&3), Some(&2));

    map.insert(1, 3);

    assert_eq!(map.get_right(&1), Some(&3));
    assert_eq!(map.get_left(&3), Some(&1));

    assert_eq!(map.get_right(&2), None);
    assert_eq!(map.get_left(&2), None);
}

#[test]
fn test_reinsertion() {
    let mut map = BiMap::default();

    map.insert(1, 2);
    map.insert(2, 3);

    assert_eq!(map.len(), 2);
    assert_eq!(map.get_right(&1), Some(&2));
    assert_eq!(map.get_left(&2), Some(&1));
    assert_eq!(map.get_right(&2), Some(&3));
    assert_eq!(map.get_left(&3), Some(&2));

    let (old_right, old_left) = map.insert(1, 2);
    assert_eq!(old_left, Some(1));
    assert_eq!(old_right, Some(2));

    assert_eq!(map.len(), 2);
    assert_eq!(map.get_right(&1), Some(&2));
    assert_eq!(map.get_left(&2), Some(&1));
    assert_eq!(map.get_right(&2), Some(&3));
    assert_eq!(map.get_left(&3), Some(&2));
}