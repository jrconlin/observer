use std::collections::btree_map::BTreeMap;
use std::fmt::Debug;

/// A counter that keeps track of the number of times a key has been seen.
/// Note, there is a `counter` crate, however it does not appear to perform
/// correctly. This code was taken from the BTreeMap example.
///

#[derive(Clone, Debug)]
pub(crate) struct Counter<K> {
    map: BTreeMap<K, i64>,
}

impl<K> Counter<K>
where
    K: Ord + Clone + Debug,
{
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    /// Return the number of known keys in the map.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Add or update a key in the counter map. Returns the current
    /// count of the key.
    pub fn add(&mut self, key: K) -> i64 {
        debug!("🧮 Adding {:?}", key);
        let count = self.map.get(&key).unwrap_or(&0) + 1;
        self.map.insert(key, count);
        count
    }

    /// Decrease the count of a key by one. This preseves the creation time so that
    /// old values can decay out of the map. Remove any key that goes below 1.
    /// Returns the count of the key if it still exists.
    pub fn decrease(&mut self, key: K) -> Option<i64> {
        let mut count = self.map.get(&key).unwrap_or(&0).to_owned();
        count -= 1;
        if count < 1 {
            self.map.remove(&key);
            None
        } else {
            self.map.insert(key, count);
            Some(count)
        }
    }

    #[allow(dead_code)]
    /// Get the count of a given key if it exists.
    pub fn get(&self, key: &K) -> Option<&i64> {
        self.map.get(key)
    }

    /// Remove a given key from the map.
    pub fn remove(&mut self, key: &K) {
        debug!("🧮 Dropping {:?}", key);
        self.map.remove(key);
    }

    /// Return just the list of known keys.
    pub fn keys(&self) -> Vec<&K> {
        self.map.keys().collect()
    }

    /// Return the most common keys in the map, sorted by occurrence.
    pub fn most_common_ordered(&self) -> Vec<(&K, i64)> {
        let mut vec = self.map.iter().collect::<Vec<(&K, &i64)>>();
        vec.sort_by(|a, b| b.1.cmp(a.1));
        vec.iter()
            .map(|(k, v)| (k.to_owned(), *v.to_owned()))
            .collect::<Vec<(&K, i64)>>()
    }

    /// Return the `k` most common keys in the map, sorted by occurrence.
    pub fn k_most_common_ordered(&self, k: usize) -> Vec<(&K, i64)> {
        self.most_common_ordered().into_iter().take(k).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let mut counter = Counter::new();
        for c in "aaabbc".chars() {
            counter.add(c.to_string());
        }

        assert_eq!(counter.get(&'a'.to_string()), Some(&3));
        assert_eq!(counter.get(&'b'.to_string()), Some(&2));
        assert_eq!(counter.get(&'c'.to_string()), Some(&1));
        assert_eq!(counter.get(&'d'.to_string()), None);

        counter.decrease('a'.to_string());
        assert_eq!(counter.get(&'a'.to_string()), Some(&2));

        counter.remove(&'a'.to_string());
        assert_eq!(counter.get(&'a'.to_string()), None);
    }
}
