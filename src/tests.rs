use super::*;

#[test]
fn test_replacing_inserts() {
    // Test that inserting a key that already exists will replace the old value

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
    assert_eq!(map.get_right(&1), Some(&4));
    assert_eq!(map.get_left(&4), Some(&1));
    assert_eq!(map.get_right(&2), None);
    assert_eq!(map.get_left(&2), None);
}

#[test]
fn test_get() {
    // Test that we get correct values from the map

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
    // Test that reinserting a mapping that already exists does not change the map

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

#[test]
fn test_contains() {
    // Test that contains returns the correct value

    let mut map = BiMap::default();

    map.insert(1, 2);
    map.insert(2, 3);

    assert_eq!(map.contains_left(&1), true);
    assert_eq!(map.contains_left(&2), true);
    assert_eq!(map.contains_left(&3), false);
    assert_eq!(map.contains_left(&0), false);
    assert_eq!(map.contains_left(&usize::MAX), false);

    assert_eq!(map.contains_right(&1), false);
    assert_eq!(map.contains_right(&2), true);
    assert_eq!(map.contains_right(&3), true);
    assert_eq!(map.contains_right(&0), false);
    assert_eq!(map.contains_right(&usize::MAX), false);
}