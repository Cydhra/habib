use std::hash::{BuildHasher, Hash, Hasher, RandomState};
use std::mem;

const DEFAULT_CAPACITY: usize = 32;

const GROWTH_FACTOR: f64 = 2.0;

const MAX_LOAD_FACTOR: f64 = 0.9;

const EMPTY_SLOT: usize = usize::MAX;

// TODO instead of linear searching, use smart search from https://ieeexplore.ieee.org/stamp/stamp.jsp?tp=&arnumber=4568152
/// A bi-directional map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BiMap<T, U, H = RandomState, RH = RandomState>
    where T: Hash + Eq, U: Hash + Eq
{
    data: Vec<Bucket<T, U>>,
    left_index: Box<[usize]>,
    right_index: Box<[usize]>,
    hasher: H,
    reverse_hasher: RH,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Bucket<T, U> {
    left: T,
    right: U,
}

impl<T, U> Default for BiMap<T, U>
    where T: Hash + Eq, U: Hash + Eq
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, U> BiMap<T, U>
    where T: Hash + Eq, U: Hash + Eq {
    /// Create a new empty BiMap with the default capacity.
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// Create a new empty BiMap with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity_with_load = Self::apply_load_factor(capacity);
        let left_index = vec![EMPTY_SLOT; capacity_with_load].into_boxed_slice();
        let right_index = vec![EMPTY_SLOT; capacity_with_load].into_boxed_slice();
        BiMap {
            data: Vec::with_capacity(capacity),
            left_index,
            right_index,
            hasher: RandomState::default(),
            reverse_hasher: RandomState::default(),
        }
    }
}

impl<T, U, H, RH> BiMap<T, U, H, RH>
    where T: Hash + Eq, U: Hash + Eq, H: BuildHasher, RH: BuildHasher
{
    /// Create a new empty BiMap with the given capacity and hashers.
    pub fn with_hashers(capacity: usize, hasher: H, reverse_hasher: RH) -> Self {
        let left_index = vec![EMPTY_SLOT; capacity].into_boxed_slice();
        let right_index = vec![EMPTY_SLOT; capacity].into_boxed_slice();
        BiMap {
            data: Vec::with_capacity(capacity),
            left_index,
            right_index,
            hasher,
            reverse_hasher,
        }
    }

    /// Increase a capacity to make sure no reallocation is required while filling the capacity even
    /// when the maximum load factor is reached.
    fn apply_load_factor(capacity: usize) -> usize {
        (capacity as f64 / MAX_LOAD_FACTOR) as usize + 1
    }

    /// Convert an element into an index by hashing it and mapping the hash to the given capacity
    fn hash_to_index<E, G>(hasher: &G, element: &E, capacity: usize) -> usize
        where E: Hash, G: BuildHasher
    {
        let mut hasher = hasher.build_hasher();
        element.hash(&mut hasher);
        hasher.finish() as usize % capacity
    }

    /// Get the ideal index (i.e. without collisions) for a left value under the current
    /// container size.
    #[inline(always)]
    fn get_ideal_index_left(&self, left: &T) -> usize {
        Self::hash_to_index(&self.hasher, left, self.current_capacity())
    }

    /// Get the ideal index (i.e. without collisions) for a right value under the current
    /// container size.
    #[inline(always)]
    fn get_ideal_index_right(&self, right: &U) -> usize {
        Self::hash_to_index(&self.reverse_hasher, right, self.current_capacity())
    }

    /// Perform the probing algorithm on the given hash index to find the index of the element.
    /// This method is used for both left and right values, and requires the ideal index function
    /// and the lookup function to be passed in. It is not intended to be called directly, but rather
    /// through the lookup_index_left and lookup_index_right methods, or during rehashing.
    ///
    /// # Parameters
    /// * `element` - The element for which to find the index.
    /// * `hash_index` - The hash index to probe.
    /// * `hasher` - The hasher to use.
    /// * `lookup` - A function that returns elements of the element type from a bucket.
    /// * `buckets` - The buckets that contain the elements.
    /// * `capacity` - The capacity of the hash index.
    #[inline(always)]
    fn probe_index<E, G>(element: &E, hash_index: &[usize], hasher: &G, lookup: fn(&Bucket<T, U>) -> &E, buckets: &[Bucket<T, U>], capacity: usize) -> Result<usize, usize>
        where E: Hash + Eq, G: BuildHasher
    {
        let ideal_index = Self::hash_to_index(hasher, element, capacity);
        let mut index = ideal_index;
        let mut dist = 0;
        while hash_index[index] < EMPTY_SLOT {
            let bucket = &buckets[hash_index[index]];
            if lookup(bucket) == element {
                return Ok(index);
            } else {
                let target_probe_dist = index.wrapping_sub(Self::hash_to_index(hasher, lookup(bucket), capacity)).rem_euclid(capacity);
                if dist > target_probe_dist {
                    return Err(index);
                }
            }

            index = (index + 1) % capacity;
            dist += 1;
        }
        Err(index)
    }

    /// Look up the index of an element in the map. This method is used for both left and right
    /// values, and requires the ideal index function and the lookup function to be passed in.
    /// It is not intended to be called directly, but rather through the lookup_index_left and
    /// lookup_index_right methods.
    ///
    /// # Arguments
    /// * `element` - The element to look up.
    /// * `ideal_index_func` - A function that returns the ideal index for the element type.
    /// * `lookup` - A function that returns elements of the element type from a bucket.
    ///
    /// # Returns
    /// The index where the element is stored or would be stored. If the element is not in the map,
    /// the index of the first empty bucket is returned.
    ///
    /// # Panics
    /// This method panics if the map is full.
    #[inline(always)]
    fn lookup_index<E, G>(&self, element: &E, hash_index: &[usize], hasher: &G, lookup: fn(&Bucket<T, U>) -> &E) -> Result<usize, usize>
        where E: Hash + Eq, G: BuildHasher
    {
        Self::probe_index(element, hash_index, hasher, lookup, &self.data, self.current_capacity())
    }

    /// Find the index that the left value is stored at or would be stored at. If the left value
    /// is not in the map, the returned index is either empty or contains a bucket with a lower
    /// probe distance.
    ///
    /// # Arguments
    /// * `left` - The left value to look up.
    ///
    /// # Returns
    /// The index where the left value is stored or would be stored. If the left value is not in the
    /// map, the index of the first empty bucket is returned.
    ///
    /// # Panics
    /// This method panics if the map is full.
    fn lookup_index_left(&self, left: &T) -> Result<usize, usize> {
        self.lookup_index(left, &self.left_index, &self.hasher, |bucket: &Bucket<T, U>| &bucket.left)
    }

    /// Find the index that the right value is stored at or would be stored at. If the right value
    /// is not in the map, the returned index is either empty or contains a bucket with a lower
    /// probe distance.
    ///
    /// # Arguments
    /// * `right` - The right value to look up.
    ///
    /// # Returns
    /// The index where the right value is stored or would be stored. If the right value is not in the
    /// map, the index of the first empty bucket is returned.
    ///
    /// # Panics
    /// This method panics if the map is full.
    fn lookup_index_right(&self, right: &U) -> Result<usize, usize> {
        self.lookup_index(right, &self.right_index, &self.reverse_hasher, |bucket: &Bucket<T, U>| &bucket.right)
    }

    /// Push a new bucket to the tail of the data array. This method is used when both left and right
    /// values are new.
    ///
    /// The method will insert the mappings into the left and right indices at the given indices.
    ///
    /// # Parameters
    /// * `bucket` - The bucket to push.
    /// * `left_index` - The index in the left index where to insert the mapping.
    /// * `right_index` - The index in the right index where to insert the mapping.
    fn push_new_bucket(&mut self, bucket: Bucket<T, U>, left_index: usize, right_index: usize) {
        self.data.push(bucket);
        self.insert_mapping_left(left_index, self.len() - 1);
        self.insert_mapping_right(right_index, self.len() - 1);
    }

    /// Delete a bucket at the given index. It will update one entry in each index, since
    /// the last bucket is moved to the deleted bucket's position.
    /// It will also delete the mappings for the bucket in both left and right indices.
    ///
    /// Note: If the bucket to delete is NOT the last bucket, the last bucket will move, meaning
    /// that the indices of the moved bucket are no longer valid. This method ensures the mappings
    /// are updated correctly, but any temporary variables that hold the indices of the moved bucket
    /// will be invalid.
    ///
    /// # Parameters
    /// * `bucket_index` - The index of the bucket to delete.
    /// * `left_meta_index` - The entry in the left index that points to the bucket to delete.
    /// If none, this method will search for the index
    /// * `right_meta_index` - The entry in the right index that points to the bucket to delete.
    /// If none, this method will search for the index
    fn delete_bucket(&mut self, bucket_index: usize, left_meta_index: Option<usize>, right_meta_index: Option<usize>) -> Bucket<T, U> {
        assert!(bucket_index < self.len(), "index out of bounds");

        if let Some(left_meta_index) = left_meta_index {
            self.delete_mapping_left(left_meta_index);
        } else {
            self.delete_mapping_left(self.lookup_index_left(&self.data[bucket_index].left).unwrap());
        }

        if let Some(right_meta_index) = right_meta_index {
            self.delete_mapping_right(right_meta_index);
        } else {
            self.delete_mapping_right(self.lookup_index_right(&self.data[bucket_index].right).unwrap());
        }

        // trivial case: delete and return the last bucket
        if bucket_index == self.len() - 1 {
            return self.data.pop().unwrap();
        }

        // find metadata of the bucket to move
        let bucket_to_move = &self.data[self.len() - 1];
        let left_index = self.lookup_index_left(&bucket_to_move.left);
        let right_index = self.lookup_index_right(&bucket_to_move.right);
        debug_assert!(left_index.is_ok());
        debug_assert!(right_index.is_ok());

        let tail = self.len() - 1;
        let (lower, upper) = self.data.split_at_mut(tail);

        // swap with last element
        mem::swap(&mut lower[bucket_index], &mut upper[0]);

        // update metadata of moved bucket
        self.left_index[left_index.unwrap()] = bucket_index;
        self.right_index[right_index.unwrap()] = bucket_index;

        // return the deleted bucket
        self.data.pop().unwrap()
    }

    /// Replace a bucket at the given index with a new bucket. The old bucket is returned.
    /// No changes to the indices are made.
    fn replace_bucket(&mut self, bucket_index: usize, bucket: Bucket<T, U>) -> Bucket<T, U> {
        assert!(bucket_index < self.len(), "index out of bounds");
        let mut old_bucket = bucket;
        mem::swap(&mut self.data[bucket_index], &mut old_bucket);
        old_bucket
    }

    /// Insert metadata into the given index for the given element and bucket index.
    /// The method will move all elements to the right until an empty slot is found.
    /// The method is used for both left and right indices.
    ///
    /// # Parameters
    /// * `meta_index` - The meta index to insert into.
    /// * `mapping_index` - The index in the meta index to insert at. It must be the index returned
    /// by the `lookup_index_left` or `lookup_index_right` method. The method will move all elements
    /// to the right until an empty slot is found, so it should be the index that already exceeds
    /// the probe distance.
    #[inline(always)]
    fn insert_mapping(meta_index: &mut [usize], mut mapping_index: usize, bucket_index: usize) {
        let mut current_content = bucket_index;
        while meta_index[mapping_index] < EMPTY_SLOT {
            mem::swap(&mut meta_index[mapping_index], &mut current_content);
            mapping_index = (mapping_index + 1) % meta_index.len();
        }
        meta_index[mapping_index] = current_content;
    }

    /// Insert metadata into the left index for the given element and bucket index.
    ///
    /// # Parameters
    /// * `mapping_index` - The index in the left index to insert at. It must be the index returned
    /// by the `lookup_index_left` method. The method will move all elements to the right until an
    /// empty slot is found.
    /// * `bucket_index` - The index of the bucket to insert.
    fn insert_mapping_left(&mut self, mapping_index: usize, bucket_index: usize) {
        Self::insert_mapping(&mut self.left_index, mapping_index, bucket_index)
    }

    /// Insert metadata into the right index for the given element and bucket index.
    ///
    /// # Parameters
    /// * `mapping_index` - The index in the right index to insert at. It must be the index returned
    /// by the `lookup_index_right` method. The method will move all elements to the right until an
    /// empty slot is found.
    /// * `bucket_index` - The index of the bucket to insert.
    fn insert_mapping_right(&mut self, mapping_index: usize, bucket_index: usize) {
        Self::insert_mapping(&mut self.right_index, mapping_index, bucket_index)
    }

    /// Delete a mapping in the left index and move following elements to the right if necessary.
    fn delete_mapping_left(&mut self, mapping_index: usize) {
        self.left_index[mapping_index] = EMPTY_SLOT;
        let mut current_mapping_index = (mapping_index + 1) % self.current_capacity();

        // move elements over until we find a free spot or an element that is already in the right spot
        let mut current_neighbor = self.left_index[current_mapping_index];

        while current_neighbor != EMPTY_SLOT && self.get_ideal_index_left(&self.data[current_neighbor].left).wrapping_sub(current_mapping_index) != 0 {
            if current_mapping_index == 0 {
                let (lower, upper) = self.left_index.split_at_mut(self.current_capacity() - 1);
                mem::swap(&mut lower[0], &mut upper[0]);
            } else {
                let (lower, upper) = self.left_index.split_at_mut(current_mapping_index);
                mem::swap(&mut lower[current_mapping_index - 1], &mut upper[0]);
            }
            current_mapping_index = (current_mapping_index + 1) % self.current_capacity();
            current_neighbor = self.left_index[current_mapping_index];
        }
    }

    /// Delete a mapping in the right index and move following elements to the right if necessary.
    fn delete_mapping_right(&mut self, mapping_index: usize) {
        self.right_index[mapping_index] = EMPTY_SLOT;
        let mut current_mapping_index = (mapping_index + 1) % self.current_capacity();

        // move elements over until we find a free spot or an element that is already in the right spot
        let mut current_neighbor = self.right_index[current_mapping_index];

        while current_neighbor != EMPTY_SLOT && self.get_ideal_index_right(&self.data[current_neighbor].right).wrapping_sub(current_mapping_index) != 0 {
            if current_mapping_index == 0 {
                let (lower, upper) = self.right_index.split_at_mut(self.current_capacity() - 1);
                mem::swap(&mut lower[0], &mut upper[0]);
            } else {
                let (lower, upper) = self.right_index.split_at_mut(current_mapping_index);
                mem::swap(&mut lower[current_mapping_index - 1], &mut upper[0]);
            }
            current_mapping_index = (current_mapping_index + 1) % self.current_capacity();
            current_neighbor = self.right_index[current_mapping_index];
        }
    }

    /// Get the current capacity for both indices.
    fn current_capacity(&self) -> usize {
        self.left_index.len()
    }

    /// Returns whether the map can fit additional `num` elements without exceeding the maximum load.
    fn can_fit(&self, num: usize) -> bool {
        ((self.len() + num) as f64) < (self.current_capacity() as f64 * MAX_LOAD_FACTOR)
    }

    /// Grow the map to the given capacity.
    fn resize(&mut self, new_capacity: usize) {
        assert!(new_capacity >= self.len(), "new capacity must be at least the current length");

        let mut new_left_index = vec![EMPTY_SLOT; new_capacity].into_boxed_slice();
        let mut new_right_index = vec![EMPTY_SLOT; new_capacity].into_boxed_slice();

        for (bucket_index, bucket) in self.data.iter().enumerate() {
            let left_element_index = Self::probe_index(&bucket.left, &new_left_index, &self.hasher, |bucket: &Bucket<T, U>| &bucket.left, &self.data[..bucket_index], new_left_index.len()).unwrap_err();
            let right_element_index = Self::probe_index(&bucket.right, &new_right_index, &self.reverse_hasher, |bucket: &Bucket<T, U>| &bucket.right, &self.data[..bucket_index], new_right_index.len()).unwrap_err();

            Self::insert_mapping(&mut new_left_index, left_element_index, bucket_index);
            Self::insert_mapping(&mut new_right_index, right_element_index, bucket_index);
        }

        self.left_index = new_left_index;
        self.right_index = new_right_index;
    }

    /// Grow the map according to the growth factor.
    fn grow(&mut self) {
        self.resize((self.current_capacity() as f64 * GROWTH_FACTOR).ceil() as usize)
    }

    /// Get the right value for the given left value. If the left value is not in the map, None is
    /// returned.
    #[must_use]
    pub fn get_right(&self, left: &T) -> Option<&U> {
        self.lookup_index_left(left)
            .ok()
            .map(|index| &self.data[self.left_index[index]].right)
    }

    /// Get the left value for the given right value. If the right value is not in the map, None is
    /// returned.
    #[must_use]
    pub fn get_left(&self, right: &U) -> Option<&T> {
        self.lookup_index_right(right)
            .ok()
            .map(|index| &self.data[self.right_index[index]].left)
    }

    /// Check if the map contains a mapping for the given left value.
    #[must_use]
    pub fn contains_left(&self, left: &T) -> bool {
        self.lookup_index_left(left).is_ok()
    }

    /// Check if the map contains a mapping for the given right value.
    #[must_use]
    pub fn contains_right(&self, right: &U) -> bool {
        self.lookup_index_right(right).is_ok()
    }

    /// Inserts a value pair into the map, creating a bijection between the two values.
    /// If the map did have one key present, its value is updated and the old value is returned.
    /// If a key did not exist, None is returned instead.
    /// The first return value is the old right value assigned to the left key, and vice versa.
    /// The map assumes that keys never return true for `==` if they are not identical.
    /// It is a logical error to insert a key into the map that is equal (`==`) to a key that is
    /// already in the map, but not identical.
    ///
    /// If the map is near full, it will resize itself.
    /// The map will never shrink itself.
    ///
    /// If both the left and right values already exist in the map, but are not mapped to each other,
    /// both mappings will be updated, which will reduce the number of mappings by one (see [`len`]).
    ///
    /// [`len`]: #method.len
    pub fn insert(&mut self, left: T, right: U) -> (Option<U>, Option<T>) {
        if !self.can_fit(1) {
            self.grow();
        }

        let left_index = self.lookup_index_left(&left);
        let right_index = self.lookup_index_right(&right);

        let mut old_right = None;
        let mut old_left = None;

        if let Ok(left_meta_index) = left_index {
            // the bucket where the left element is currently stored, henceforth "the left bucket".
            let mut left_bucket = self.left_index[left_meta_index];

            // delete the right bucket if it exists and store the old left value
            // unless the right bucket is the same as the left bucket
            if let Ok(right_meta_index) = right_index {
                // the bucket where the right index is currently stored, henceforth "the right bucket".
                let right_bucket = self.right_index[right_meta_index];

                if left_bucket != right_bucket {
                    // delete the right bucket
                    let bucket = self.delete_bucket(right_bucket, None, right_index.ok());

                    // if the left bucket was moved to the right bucket's position, update the left index
                    if left_bucket == self.len() {
                        // the left bucket was moved to the right bucket's position, update the left index
                        left_bucket = right_bucket;
                    }

                    old_left = Some(bucket.left);
                } else {
                    // old mapping is equal to the new mapping, do nothing
                    return (Some(right), Some(left));
                }
            }

            // delete the right mapping for the left bucket, since we will insert a new right value,
            // and insert that value
            self.delete_mapping_right(self.lookup_index_right(&self.data[left_bucket].right).unwrap());
            self.insert_mapping_right(self.lookup_index_right(&right).unwrap_err(), left_bucket);

            // replace left bucket with new bucket, no update to left index necessary, since it
            // already points to this bucket.
            let bucket = self.replace_bucket(left_bucket, Bucket { left, right });
            old_right = Some(bucket.right);
        } else if let Ok(right_meta_index) = right_index {
            let right_bucket = self.right_index[right_meta_index];

            // replace the right bucket with the new bucket, and delete the left mapping to it,
            // since we insert a new left mapping for the new value
            self.delete_mapping_left(self.lookup_index_left(&self.data[right_bucket].left).unwrap());

            // insert mapping to the left index, no update to right index necessary.
            self.insert_mapping_left(left_index.unwrap_err(), right_bucket);
            let bucket = self.replace_bucket(right_bucket, Bucket { left, right });
            old_left = Some(bucket.left);
        } else {
            self.push_new_bucket(Bucket { left, right }, left_index.unwrap_err(), right_index.unwrap_err());
        }

        (old_right, old_left)
    }

    /// Tries to insert a value pair into the map, creating a bijection between the two values.
    /// If the map already had one of the values present, nothing is updated, and an error containing
    /// the present values is returned. The first value in the tuple is the present right value for the
    /// left key, and the second value is the present left value for the right key.
    /// If the map did not have any of the values present, the values are inserted and Ok is returned.
    ///
    /// If the map is near full, it will resize itself.
    // TODO adjust this method to mirror HashMap::try_insert (when it gets stabilized)
    //  this includes changing the Err to an occupied error type,
    //  and changing the name if Rust decides that try_ should be reserved to allocation errors
    pub fn try_insert(&mut self, left: T, right: U) -> Result<(), (Option<&U>, Option<&T>)> {
        let left_index = self.lookup_index_left(&left);
        let right_index = self.lookup_index_right(&right);

        if left_index.is_err() && right_index.is_err() {
            if !self.can_fit(1) {
                self.grow();
            }

            self.push_new_bucket(Bucket { left, right }, left_index.unwrap_err(), right_index.unwrap_err());
            Ok(())
        } else {
            Err((left_index.ok().map(|index| &self.data[self.left_index[index]].right), right_index.ok().map(|index| &self.data[self.right_index[index]].left)))
        }
    }

    /// Deletes the mappings for the given left value and returns the right value that was mapped to it.
    /// If the left value is not in the map, None is returned.
    pub fn remove_left(&mut self, left: &T) -> Option<U> {
        let left_index = self.lookup_index_left(left);
        if let Ok(left_meta_index) = left_index {
            let bucket = self.left_index[left_meta_index];

            // delete the bucket
            let bucket = self.delete_bucket(bucket, left_index.ok(), None);
            Some(bucket.right)
        } else {
            None
        }
    }

    /// Deletes the mappings for the given right value and returns the left value that was mapped to it.
    /// If the right value is not in the map, None is returned.
    pub fn remove_right(&mut self, right: &U) -> Option<T> {
        let right_index = self.lookup_index_right(right);
        if let Ok(right_meta_index) = right_index {
            let bucket = self.right_index[right_meta_index];

            // delete the bucket
            let bucket = self.delete_bucket(bucket, None, right_index.ok());
            Some(bucket.left)
        } else {
            None
        }
    }

    /// Reserves capacity for at least `additional` more elements to be inserted in the map.
    /// The collection may reserve more space to speculatively avoid frequent reallocations.
    /// After calling reserve, capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    ///
    /// # Panics
    /// Panics, if the new capacity overflows usize.
    /// Panics, if the allocation fails.
    pub fn reserve(&mut self, additional: usize) {
        if !self.can_fit(additional) {
            if usize::MAX - self.current_capacity() < additional {
                panic!("capacity overflow");
            }

            let new_capacity = Self::apply_load_factor(self.len() + additional);
            self.resize(new_capacity);
        }
    }

    /// Shrinks the capacity of the map as much as possible.
    /// It will drop down as much as possible while maintaining the internal rules and possibly
    /// leaving some space in accordance with the resize policy.
    pub fn shrink_to_fit(&mut self) {
        // we want to leave some space to avoid too many collisions
        let new_capacity = Self::apply_load_factor(self.len());
        self.resize(new_capacity);
        self.data.shrink_to_fit();
    }

    /// Clears the map, removing all mappings. Keeps the allocated memory for reuse.
    pub fn clear(&mut self) {
        self.data.clear();
        self.left_index.fill(EMPTY_SLOT);
        self.right_index.fill(EMPTY_SLOT);
    }

    /// Returns an iterator over the mappings in the map in arbitrary order.
    pub fn iter(&self) -> impl Iterator<Item=(&T, &U)> {
        self.data.iter().map(|bucket| (&bucket.left, &bucket.right))
    }

    /// Returns an iterator over the left values in the map in arbitrary order.
    pub fn left_values(&self) -> impl Iterator<Item=&T> {
        self.data.iter().map(|bucket| &bucket.left)
    }

    /// Returns an iterator over the right values in the map in arbitrary order.
    pub fn right_values(&self) -> impl Iterator<Item=&U> {
        self.data.iter().map(|bucket| &bucket.right)
    }

    /// Returns the number of bijections stored in the map, meaning it is half the number of values.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns a reference to the map’s `BuildHasher` for left values.
    pub fn hasher_left(&self) -> &H {
        &self.hasher
    }

    /// Returns a reference to the map’s `BuildHasher` for right values.
    pub fn hasher_right(&self) -> &RH {
        &self.reverse_hasher
    }
}

#[cfg(test)]
mod tests;