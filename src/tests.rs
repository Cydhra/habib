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
fn test_try_inserts() {
    // Test that inserting a key that already exists will return an error

    let mut map = BiMap::default();

    let result = map.try_insert(1, 2);
    assert_eq!(result, Ok(()));
    assert_eq!(map.len(), 1);

    let result = map.try_insert(2, 3);
    assert_eq!(result, Ok(()));
    assert_eq!(map.len(), 2);

    let result = map.try_insert(2, 4);
    assert_eq!(result, Err((Some(&3), None)));
    assert_eq!(map.len(), 2);

    let result = map.try_insert(1, 4);
    assert_eq!(result, Err((Some(&2), None)));
    assert_eq!(map.len(), 2);

    let result = map.try_insert(0, 2);
    assert_eq!(result, Err((None, Some(&1))));
    assert_eq!(map.len(), 2);

    let result = map.try_insert(0, 0);
    assert_eq!(result, Ok(()));
    assert_eq!(map.len(), 3);

    let result = map.try_insert(0, 0);
    assert_eq!(result, Err((Some(&0), Some(&0))));
    assert_eq!(map.len(), 3);

    let result = map.try_insert(0, 3);
    assert_eq!(result, Err((Some(&0), Some(&2))));
    assert_eq!(map.len(), 3);
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

#[test]
fn test_deletion() {
    // Test that deleting a key removes the mapping

    let mut map = BiMap::default();

    map.insert(1, 2);
    map.insert(2, 3);

    assert_eq!(map.len(), 2);

    let right = map.remove_left(&1);
    assert_eq!(right, Some(2));
    assert_eq!(map.len(), 1);
    assert_eq!(map.get_right(&1), None);
    assert_eq!(map.get_left(&2), None);
    assert_eq!(map.get_right(&2), Some(&3));
    assert_eq!(map.get_left(&3), Some(&2));

    let left = map.remove_right(&3);
    assert_eq!(left, Some(2));
    assert_eq!(map.len(), 0);
    assert_eq!(map.get_right(&2), None);
    assert_eq!(map.get_left(&3), None);

    let right = map.remove_left(&1);
    assert_eq!(right, None);
    assert!(map.is_empty());

    let left = map.remove_right(&2);
    assert_eq!(left, None);
    assert!(map.is_empty());

    map.insert(1, 2);
    assert_eq!(map.remove_left(&0), None);
    assert_eq!(map.remove_right(&0), None);
    assert_eq!(map.get_right(&1), Some(&2));
    assert_eq!(map.get_left(&2), Some(&1));
}

#[test]
fn test_insert_after_delete() {
    // Test that inserting a key after deleting it works

    let mut map = BiMap::default();

    map.insert(1, 2);
    map.insert(2, 3);

    assert_eq!(map.len(), 2);
    assert!(map.try_insert(1, 2).is_err());

    assert_eq!(map.remove_left(&1), Some(2));
    assert_eq!(map.len(), 1);
    assert_eq!(map.get_right(&1), None);
    assert_eq!(map.get_left(&2), None);

    assert!(map.try_insert(1, 2).is_ok());
    assert_eq!(map.len(), 2);
    assert_eq!(map.get_right(&1), Some(&2));
    assert_eq!(map.get_left(&2), Some(&1));

    assert_eq!(map.remove_right(&3), Some(2));
    assert_eq!(map.get_right(&2), None);
    assert_eq!(map.get_left(&3), None);

    map.insert(1, 3);
    assert_eq!(map.len(), 1);
    assert_eq!(map.get_right(&1), Some(&3));
    assert_eq!(map.get_left(&3), Some(&1));
    assert_eq!(map.get_right(&2), None);
    assert_eq!(map.get_left(&2), None);
}