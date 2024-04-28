use std::hash::{BuildHasher, Hash, Hasher, RandomState};
use std::mem;

const DEFAULT_CAPACITY: usize = 32;

const EMPTY_SLOT: usize = usize::MAX;

// TODO instead of linear searching, use smart search from https://ieeexplore.ieee.org/stamp/stamp.jsp?tp=&arnumber=4568152
/// A bi-directional map.
#[derive(Clone, PartialEq, Eq)]
pub struct BiMap<T, U, H = RandomState, RH = RandomState>
    where T: Hash + Eq, U: Hash + Eq
{
    // TODO can we profit of Result-optimization if the 0th element is a sentinel?
    data: Vec<Bucket<T, U>>,
    left_index: Box<[usize]>,
    right_index: Box<[usize]>,
    hasher: H,
    reverse_hasher: RH,
}

#[derive(Clone, PartialEq, Eq)]
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
        let capacity = capacity.next_power_of_two();
        let left_index = vec![EMPTY_SLOT; capacity].into_boxed_slice();
        let right_index = vec![EMPTY_SLOT; capacity].into_boxed_slice();
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
    /// Get the ideal index (i.e. without collisions) for a left value under the current
    /// container size.
    #[inline(always)]
    fn get_ideal_index_left(&self, left: &T) -> usize {
        let mut hasher = self.hasher.build_hasher();
        left.hash(&mut hasher);
        hasher.finish() as usize % self.current_capacity()
    }

    /// Get the ideal index (i.e. without collisions) for a right value under the current
    /// container size.
    #[inline(always)]
    fn get_ideal_index_right(&self, right: &U) -> usize {
        let mut hasher = self.reverse_hasher.build_hasher();
        right.hash(&mut hasher);
        hasher.finish() as usize % self.current_capacity()
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
    fn lookup_index<E>(&self, element: &E, hash_index: &[usize], ideal_index_func: fn(&Self, &E) -> usize, lookup: fn(&Bucket<T, U>) -> &E) -> Result<usize, usize>
        where E: Hash + Eq
    {
        let ideal_index = ideal_index_func(&self, element);
        let mut index = ideal_index;
        let mut dist = 0;
        while hash_index[index] < EMPTY_SLOT {
            let bucket = &self.data[hash_index[index]];
            if lookup(bucket) == element {
                return Ok(index);
            } else {
                let target_probe_dist = index.wrapping_sub(ideal_index_func(&self, lookup(bucket))).rem_euclid(self.current_capacity());
                if dist > target_probe_dist {
                    return Err(index);
                }
            }

            index = (index + 1) % self.current_capacity();
            dist += 1;
        }
        Err(index)
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
        self.lookup_index(left, &self.left_index, Self::get_ideal_index_left, |bucket: &Bucket<T, U>| &bucket.left)
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
        self.lookup_index(right, &self.right_index, Self::get_ideal_index_right, |bucket: &Bucket<T, U>| &bucket.right)
    }

    /// Push a new bucket to the tail of the data array. This method is used when both left and right
    /// values are new.
    /// No changes to the indices are made.
    fn push_new_bucket(&mut self, bucket: Bucket<T, U>) {
        self.data.push(bucket);
    }

    /// Delete a bucket at the given index. It will update one entry in each index, since
    /// the last bucket is moved to the deleted bucket's position.
    fn delete_bucket(&mut self, bucket_index: usize) -> Bucket<T, U> {
        assert!(bucket_index < self.len(), "index out of bounds");

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
    /// * `mapping_index` - The index in the left index to insert at. It must be the index returnedd
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

    /// Update the index where the mapping is pointing to in the right index.
    fn update_mapping_right(&mut self, mapping_index: usize, bucket_index: usize) {
        self.right_index[mapping_index] = bucket_index;
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

    /// Get the current capacity for both indices.
    fn current_capacity(&self) -> usize {
        self.left_index.len()
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

    pub fn insert(&mut self, left: T, right: U) -> (Option<U>, Option<T>) {
        // TODO check if the map is near full and resize if necessary

        let left_index = self.lookup_index_left(&left);
        let right_index = self.lookup_index_right(&right);

        let mut old_right = None;
        let mut old_left = None;

        if let Ok(left_meta_index) = left_index {
            let left_bucket = self.left_index[left_meta_index];

            // delete the right bucket if it exists and store the old left value
            // unless the right bucket is the same as the left bucket
            if let Ok(right_meta_index) = right_index {
                let right_bucket = self.right_index[right_meta_index];
                if left_bucket != right_bucket {
                    // delete the mapping for the left element of this bucket
                    self.delete_mapping_left(self.lookup_index_left(&self.data[right_bucket].left).unwrap());

                    // delete the right bucket
                    let bucket = self.delete_bucket(right_bucket);

                    // update the right index mapping to the left bucket that will be replaced
                    self.update_mapping_right(right_meta_index, left_bucket);
                    old_left = Some(bucket.left);
                } else {
                    // old mapping is equal to the new mapping, do nothing
                    return (Some(right), Some(left));
                }
            } else {
                // insert a new mapping for the right value
                self.insert_mapping_right(right_index.unwrap_err(), left_bucket);
            }

            // replace left bucket with new bucket, no update to left index necessary
            let bucket = self.replace_bucket(left_bucket, Bucket { left, right });
            old_right = Some(bucket.right);
        } else if let Ok(right_meta_index) = right_index {
            let right_bucket = self.right_index[right_meta_index];
            let bucket = self.replace_bucket(right_bucket, Bucket { left, right });
            old_left = Some(bucket.left);

            // insert mapping to the left index, no update to right index necessary
            self.insert_mapping_left(left_index.unwrap_err(), right_bucket);
        } else {
            self.push_new_bucket(Bucket { left, right });

            self.insert_mapping_left(left_index.unwrap_err(), self.len() - 1);
            self.insert_mapping_right(right_index.unwrap_err(), self.len() - 1);
        }

        (old_right, old_left)
    }

    /// Returns the number of bijections stored in the map, meaning it is half the number of elements.
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests;