use super::*;

/// A hasher that simply returns the first byte of the input as the hash, for testing purposes
struct IdentityHasher {
    modulus: u8,
    state: u8,
}

impl Hasher for IdentityHasher {
    fn finish(&self) -> u64 {
        self.state as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        self.state = bytes.iter().find(|&&b| b != 0).or(Some(&0)).map(|&b| b % self.modulus).unwrap();
    }
}

impl Default for IdentityHasher {
    fn default() -> Self {
        IdentityHasher { modulus: DEFAULT_CAPACITY as u8, state: 0 }
    }
}

impl BuildHasher for IdentityHasher {
    type Hasher = IdentityHasher;

    fn build_hasher(&self) -> Self::Hasher {
        IdentityHasher::default()
    }
}

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

#[test]
fn test_collisions() {
    // Test that the map works correctly when two values are inserted with the same hash

    let mut map = BiMap::with_hashers(DEFAULT_CAPACITY, IdentityHasher::default(), IdentityHasher::default());

    // verify the test is working as expected
    assert_eq!(map.get_ideal_index_left(&1), map.get_ideal_index_left(&(DEFAULT_CAPACITY + 1)));

    // insert colliding values
    map.insert(1, 2);
    map.insert(DEFAULT_CAPACITY + 1, 3);
    map.insert(2 * DEFAULT_CAPACITY + 1, 4);

    assert_eq!(map.get_right(&1), Some(&2));
    assert_eq!(map.get_left(&2), Some(&1));
    assert_eq!(map.get_right(&(DEFAULT_CAPACITY + 1)), Some(&3));
    assert_eq!(map.get_left(&3), Some(&(DEFAULT_CAPACITY + 1)));
    assert_eq!(map.get_right(&(2 * DEFAULT_CAPACITY + 1)), Some(&4));
    assert_eq!(map.get_left(&4), Some(&(2 * DEFAULT_CAPACITY + 1)));

    // remove last collision
    map.remove_left(&(2 * DEFAULT_CAPACITY + 1));

    // verify other values are still present
    assert_eq!(map.get_right(&1), Some(&2));
    assert_eq!(map.get_left(&2), Some(&1));
    assert_eq!(map.get_right(&(DEFAULT_CAPACITY + 1)), Some(&3));
    assert_eq!(map.get_left(&3), Some(&(DEFAULT_CAPACITY + 1)));

    // verify deleted values are gone
    assert_eq!(map.get_right(&(2 * DEFAULT_CAPACITY + 1)), None);
    assert_eq!(map.get_left(&4), None);

    // reinsert last collision
    map.insert(2 * DEFAULT_CAPACITY + 1, 4);

    // remove second collision
    map.remove_left(&(DEFAULT_CAPACITY + 1));

    // verify other values are still present
    assert_eq!(map.get_right(&1), Some(&2));
    assert_eq!(map.get_left(&2), Some(&1));
    assert_eq!(map.get_right(&(2 * DEFAULT_CAPACITY + 1)), Some(&4));
    assert_eq!(map.get_left(&4), Some(&(2 * DEFAULT_CAPACITY + 1)));

    // verify deleted values are gone
    assert_eq!(map.get_right(&(DEFAULT_CAPACITY + 1)), None);
    assert_eq!(map.get_left(&3), None);
}

#[test]
fn test_collisions_wrapping() {
    // test that the map works correctly when two values are inserted with the same hash,
    // and the index for linear probing wraps around the end of the array

    let mut map = BiMap::with_hashers(DEFAULT_CAPACITY, IdentityHasher::default(), IdentityHasher::default());

    // verify the test is working as expected
    assert_eq!(map.get_ideal_index_left(&31), 31);
    assert_eq!(map.get_ideal_index_left(&(DEFAULT_CAPACITY + 31)), 31);

    map.insert(31, 2);

    // verify this hasn't wrapped around
    assert_eq!(map.left_index[0], EMPTY_SLOT);

    // insert colliding values, one of which should end up at index 0 of the mapping
    map.insert(DEFAULT_CAPACITY + 31, 3);

    // verify wrap-around
    assert_ne!(map.left_index[0], EMPTY_SLOT);
    assert_eq!(map.left_index[1], EMPTY_SLOT);

    // insert second colliding value
    map.insert(2 * DEFAULT_CAPACITY + 31, 4);

    // verify wrap-around
    assert_ne!(map.left_index[1], EMPTY_SLOT);

    // verify the values are recovered correctly
    assert_eq!(map.get_right(&31), Some(&2));
    assert_eq!(map.get_left(&2), Some(&31));
    assert_eq!(map.get_right(&(DEFAULT_CAPACITY + 31)), Some(&3));
    assert_eq!(map.get_left(&3), Some(&(DEFAULT_CAPACITY + 31)));
    assert_eq!(map.get_right(&(2 * DEFAULT_CAPACITY + 31)), Some(&4));
    assert_eq!(map.get_left(&4), Some(&(2 * DEFAULT_CAPACITY + 31)));

    // remove last collision
    map.remove_left(&(2 * DEFAULT_CAPACITY + 31));

    // verify other values are still present
    assert_eq!(map.get_right(&31), Some(&2));
    assert_eq!(map.get_left(&2), Some(&31));
    assert_eq!(map.get_right(&(DEFAULT_CAPACITY + 31)), Some(&3));
    assert_eq!(map.get_left(&3), Some(&(DEFAULT_CAPACITY + 31)));

    // verify deleted values are gone
    assert_eq!(map.get_right(&(2 * DEFAULT_CAPACITY + 31)), None);
    assert_eq!(map.get_left(&4), None);

    // reinsert last collision
    map.insert(2 * DEFAULT_CAPACITY + 31, 4);

    assert_ne!(map.left_index[0], EMPTY_SLOT);
    assert_ne!(map.left_index[1], EMPTY_SLOT);

    // remove second collision
    map.remove_left(&(DEFAULT_CAPACITY + 31));

    // verify other values are still present
    assert_eq!(map.get_right(&31), Some(&2));
    assert_eq!(map.get_left(&2), Some(&31));
    assert_eq!(map.get_right(&(2 * DEFAULT_CAPACITY + 31)), Some(&4));
    assert_eq!(map.get_left(&4), Some(&(2 * DEFAULT_CAPACITY + 31)));

    // verify deleted values are gone
    assert_eq!(map.get_right(&(DEFAULT_CAPACITY + 31)), None);
    assert_eq!(map.get_left(&3), None);
}

#[test]
fn test_collisions_replacement() {
    // test that the map works correctly when two values are inserted with the same hash,
    // and then some of them are replaced by a new insertion
    let mut map = BiMap::with_hashers(DEFAULT_CAPACITY, IdentityHasher::default(), IdentityHasher::default());

    map.insert(1, 2);
    map.insert(DEFAULT_CAPACITY + 1, 3);
    map.insert(1, 3);

    assert_eq!(map.get_right(&1), Some(&3));
    assert_eq!(map.get_left(&3), Some(&1));
    assert_eq!(map.get_right(&31), None);
    assert_eq!(map.get_left(&2), None);

    let mut map = BiMap::with_hashers(DEFAULT_CAPACITY, IdentityHasher::default(), IdentityHasher::default());

    map.insert(1, 2);
    map.insert(DEFAULT_CAPACITY + 1, 3);
    map.insert(1, 4);

    assert_eq!(map.get_right(&1), Some(&4));
    assert_eq!(map.get_left(&4), Some(&1));
    assert_eq!(map.get_right(&(DEFAULT_CAPACITY + 1)), Some(&3));
    assert_eq!(map.get_left(&3), Some(&(DEFAULT_CAPACITY + 1)));
    assert_eq!(map.get_right(&2), None);

    map.insert(DEFAULT_CAPACITY + 1, 5);

    assert_eq!(map.get_right(&1), Some(&4));
    assert_eq!(map.get_left(&4), Some(&1));
    assert_eq!(map.get_right(&(DEFAULT_CAPACITY + 1)), Some(&5));
    assert_eq!(map.get_left(&5), Some(&(DEFAULT_CAPACITY + 1)));
}

#[test]
fn test_multi_collision() {
    // test whether a lot of collisions are resolved correctly
    let mut map = BiMap::with_hashers(DEFAULT_CAPACITY, IdentityHasher::default(), IdentityHasher::default());

    for i in 0..10 {
        map.insert(i * DEFAULT_CAPACITY + 1, i + 1);

        for j in 0..=i {
            // verify the overflow slots are actually used (otherwise the test is broken)
            assert_ne!(map.left_index[1 + j], EMPTY_SLOT);
        }

        // verify the next slot after all overflow slots is empty
        assert_eq!(map.left_index[1 + i + 1], EMPTY_SLOT);

        for j in 0..=i {
            assert_eq!(map.get_right(&(j * DEFAULT_CAPACITY + 1)), Some(&(j + 1)));
            assert_eq!(map.get_left(&(j + 1)), Some(&(j * DEFAULT_CAPACITY + 1)));
        }
    }

    // test whether a lot of collisions are resolved correctly,
    // some of which wrap around the end of the array
    let mut map = BiMap::with_hashers(DEFAULT_CAPACITY, IdentityHasher::default(), IdentityHasher::default());

    for i in 0..10 {
        map.insert(i * DEFAULT_CAPACITY + (DEFAULT_CAPACITY - 2), i + 1);

        for j in 0..=i {
            assert_eq!(map.get_right(&(j * DEFAULT_CAPACITY + (DEFAULT_CAPACITY - 2))), Some(&(j + 1)));
            assert_eq!(map.get_left(&(j + 1)), Some(&(j * DEFAULT_CAPACITY + (DEFAULT_CAPACITY - 2))));
        }
    }

    // test whether a lot of collisions are resolved correctly,
    // some of which wrap around the end of the array, with some deletions

    map.remove_left(&(DEFAULT_CAPACITY - 2));

    assert_eq!(map.get_right(&(DEFAULT_CAPACITY - 2)), None);
    assert_eq!(map.get_left(&1), None);

    for j in 1..10 {
        assert_eq!(map.get_right(&(j * DEFAULT_CAPACITY + (DEFAULT_CAPACITY - 2))), Some(&(j + 1)));
        assert_eq!(map.get_left(&(j + 1)), Some(&(j * DEFAULT_CAPACITY + (DEFAULT_CAPACITY - 2))));
    }

    map.remove_right(&4);

    for j in 1..10 {
        if j != 3 {
            assert_eq!(map.get_right(&(j * DEFAULT_CAPACITY + (DEFAULT_CAPACITY - 2))), Some(&(j + 1)));
            assert_eq!(map.get_left(&(j + 1)), Some(&(j * DEFAULT_CAPACITY + (DEFAULT_CAPACITY - 2))));
        } else {
            assert_eq!(map.get_right(&(j * DEFAULT_CAPACITY + (DEFAULT_CAPACITY - 2))), None);
            assert_eq!(map.get_left(&(j + 1)), None);
        }
    }
}

#[test]
fn test_right_collision() {
    // test whether replacing works correctly when the right value has a collision
    let mut map = BiMap::with_hashers(DEFAULT_CAPACITY, IdentityHasher::default(), IdentityHasher::default());

    map.insert(1, 4);
    map.insert(2, DEFAULT_CAPACITY + 4);
    map.insert(1, DEFAULT_CAPACITY + 4);

    assert_eq!(map.get_right(&1), Some(&(DEFAULT_CAPACITY + 4)));
    assert_eq!(map.get_left(&(DEFAULT_CAPACITY + 4)), Some(&1));

    let mut map = BiMap::with_hashers(DEFAULT_CAPACITY, IdentityHasher::default(), IdentityHasher::default());

    map.insert(1, 4);
    map.insert(2, DEFAULT_CAPACITY + 4);
    map.insert(2, 4);

    assert_eq!(map.get_right(&2), Some(&4));
    assert_eq!(map.get_left(&4), Some(&2));
}

#[test]
fn test_move_bucket_during_replacement() {
    // test whether buckets are replaced correctly, if they move during the operation.
    // this can happen if an insert overwrites two mappings, and the right mapping is not the last
    // mapping in the mapping vector, but the left mapping is. In that case the left mapping will
    // be moved to the place of the right mapping, which can confuse the insert operation.

    let mut map = BiMap::default();

    map.insert(1, 2);
    map.insert(3, 4);

    // verify the data structure looks like the test expects, otherwise the test case is broken
    assert_eq!(map.data[0], Bucket { left: 1, right: 2 });
    assert_eq!(map.data[1], Bucket { left: 3, right: 4 });

    map.insert(3, 2);

    // verify the data structure looks like the test expects, otherwise the test case is broken
    assert_eq!(map.data[0], Bucket { left: 3, right: 2 });

    assert_eq!(map.get_right(&1), None);
    assert_eq!(map.get_right(&3), Some(&2));
    assert_eq!(map.get_left(&2), Some(&3));
    assert_eq!(map.get_left(&4), None);
}

#[test]
fn test_clear() {
    // test whether the map is cleared correctly
    let mut map = BiMap::default();

    map.insert(1, 2);
    map.insert(3, 4);

    map.clear();

    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map.get_right(&1), None);
    assert_eq!(map.get_left(&2), None);
    assert_eq!(map.get_right(&3), None);
    assert_eq!(map.get_left(&4), None);
}

#[test]
fn test_growing() {
    // test whether the map grows correctly
    let mut map = BiMap::with_capacity(10);

    let initial_capacity = map.current_capacity();

    // insert more than capacity elements
    for i in 0..40 {
        map.insert(i, i + 1);
    }

    for i in 0..40 {
        assert_eq!(map.get_right(&i), Some(&(i + 1)));
        assert_eq!(map.get_left(&(i + 1)), Some(&i));
    }

    assert!(map.current_capacity() > initial_capacity);
}

#[test]
fn test_capacity() {
    // test that the load factor is not exceeded when map is allocated with capacity

    for i in 0..1000 {
        let map = BiMap::<u8, u8>::with_capacity(i);
        assert!(map.can_fit(i), "Failed for capacity {}", i);
    }
}

#[test]
fn test_reserve() {
    // test that the map grows correctly when reserving space
    let mut map = BiMap::<u8, u8>::with_capacity(100);
    map.reserve(1000);
    assert!(map.can_fit(1000));

    let mut map = BiMap::<u8, u8>::with_capacity(200);
    map.reserve(5000);
    assert!(map.can_fit(5000));

    let mut map = BiMap::<u8, u8>::with_capacity(10);
    map.reserve(11);
    assert!(map.can_fit(11));

    let mut map = BiMap::<u8, u8>::with_capacity(10);
    map.reserve(0);
    assert!(map.can_fit(10));

    let mut map = BiMap::<u8, u8>::with_capacity(1000);
    for i in 0..200 {
        map.insert(i, i);
    }
    map.reserve(2000);
    assert!(map.can_fit(2000));
    for i in 0..200 {
        assert_eq!(map.get_right(&i), Some(&i));
        assert_eq!(map.get_left(&i), Some(&i));
    }
}

#[test]
fn test_shrink_to_fit() {
    let mut map = BiMap::with_capacity(1000);
    assert!(map.current_capacity() >= 1000);

    for i in 0..100 {
        map.insert(i, i);
    }

    map.shrink_to_fit();

    for i in 0..100 {
        assert_eq!(map.get_right(&i), Some(&i));
        assert_eq!(map.get_left(&i), Some(&i));
    }

    assert!(map.current_capacity() < 1000);
}

#[test]
fn test_shrink_to() {
    let mut map = BiMap::with_capacity(2000);
    assert!(map.current_capacity() >= 2000);

    for i in 0..100 {
        map.insert(i, i);
    }

    map.shrink_to(500);

    for i in 0..100 {
        assert_eq!(map.get_right(&i), Some(&i));
        assert_eq!(map.get_left(&i), Some(&i));
    }

    assert!(map.current_capacity() >= 500 && map.current_capacity() < 2000);

    map.shrink_to(10);

    for i in 0..100 {
        assert_eq!(map.get_right(&i), Some(&i));
        assert_eq!(map.get_left(&i), Some(&i));
    }

    assert!(map.current_capacity() < 500);
}

#[test]
fn test_iter() {
    // test that the iterator returns all elements
    let mut map = BiMap::default();

    map.insert(1, 2);
    map.insert(3, 4);
    map.insert(5, 6);
    map.insert(7, 8);
    map.insert(1, 8);

    let from_iter = map.iter().collect::<Vec<_>>();

    assert_eq!(from_iter.len(), 3);
    assert!(from_iter.contains(&(&1, &8)));
    assert!(from_iter.contains(&(&3, &4)));
    assert!(from_iter.contains(&(&5, &6)));

    let from_iter = map.left_values().collect::<Vec<_>>();

    assert_eq!(from_iter.len(), 3);
    assert!(from_iter.contains(&&1));
    assert!(from_iter.contains(&&3));
    assert!(from_iter.contains(&&5));

    let from_iter = map.right_values().collect::<Vec<_>>();

    assert_eq!(from_iter.len(), 3);
    assert!(from_iter.contains(&&4));
    assert!(from_iter.contains(&&6));
    assert!(from_iter.contains(&&8));
}

#[test]
pub fn test_drain() {
    // test that the drain iterator returns all elements
    let mut map = BiMap::default();

    map.insert(1, 2);
    map.insert(3, 4);
    map.insert(5, 6);
    map.insert(7, 8);
    map.insert(1, 8);

    let from_iter = map.drain().collect::<Vec<_>>();

    assert_eq!(from_iter.len(), 3);
    assert!(from_iter.contains(&(1, 8)));
    assert!(from_iter.contains(&(3, 4)));
    assert!(from_iter.contains(&(5, 6)));

    assert!(map.is_empty());

    assert_eq!(map.get_left(&1), None);
    assert_eq!(map.get_right(&8), None);
    assert_eq!(map.get_left(&3), None);
    assert_eq!(map.get_right(&4), None);
    assert_eq!(map.get_left(&5), None);
    assert_eq!(map.get_right(&6), None);
}