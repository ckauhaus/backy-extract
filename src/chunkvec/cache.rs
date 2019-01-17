use crate::{CHUNKSIZE, ZERO_CHUNK};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// chunk should be cached, but no data currently available
    Unknown,
    /// chunk has been cached and here is a copy
    Known(Vec<u8>),
    /// chunk has been cached and we know that this is a zero page
    KnownZero,
    /// chunk should not be cached
    Ignored,
}

use self::Entry::*;

const MAX_KEYS: usize = 16;

/// Cache those chunks which IDs are mentioned more than once in the revision spec. All other
/// IDs are not worth the effort. The cache size is further constrained by `MAX_KEYS` to limit
/// memory usage.
#[derive(Debug, Clone, Default)]
pub struct Cache<'a> {
    map: HashMap<&'a str, Entry>,
}

impl<'a> Cache<'a> {
    /// Initializes cache from `mapping` found in the revision spec.
    pub fn new(mapping: &HashMap<&'a str, &'a str>) -> Self {
        let mut histogram: HashMap<&str, u32> =
            mapping.values().fold(HashMap::new(), |mut h, id| {
                *(h.entry(*id).or_insert(0)) += 1;
                h
            });
        Self {
            map: histogram
                .drain()
                .filter(|(_, count)| *count > 1)
                .take(MAX_KEYS)
                .map(|(id, _)| (id, Unknown))
                .collect(),
        }
    }

    /// Checks if an ID is in the cache and returns a copy of the chunk if it is.
    pub fn query(&self, id: &'a str) -> Entry {
        self.map.get(&id).cloned().unwrap_or(Ignored)
    }

    /// Inserts a chunk into the cache. Panics if the `id` is not supposed to be cached.
    pub fn memoize(&mut self, id: &'a str, data: &[u8]) {
        let prev = if data == &ZERO_CHUNK[0..CHUNKSIZE as usize] {
            self.map.insert(id, KnownZero)
        } else {
            self.map.insert(id, Known(data.to_vec()))
        };
        if prev.is_none() {
            panic!("Trying to insert ignored key '{}' into cache", id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::assert_that;
    use galvanic_assert::matchers::collection::*;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref mapping: HashMap<&'static str, &'static str> = [
            ("0", "091b34a64e50a123da8547a1b700c7e8"),
            ("1", "df834550b74a5878179484f622ef7461"),
            ("2", "a06db62ff5c8e32a5c8132535f74b13d"),
            ("3", "830438f8c407cbf6712b9bd07731e6e2"),  // 2 times
            ("4", "c72b4ba82d1f51b71c8a18195ad33fc8"),  // 14 times
            ("5", "c72b4ba82d1f51b71c8a18195ad33fc8"),
            ("6", "c72b4ba82d1f51b71c8a18195ad33fc8"),
            ("7", "c72b4ba82d1f51b71c8a18195ad33fc8"),
            ("8", "c72b4ba82d1f51b71c8a18195ad33fc8"),
            ("9", "c72b4ba82d1f51b71c8a18195ad33fc8"),
            ("10", "c72b4ba82d1f51b71c8a18195ad33fc8"),
            ("11", "c72b4ba82d1f51b71c8a18195ad33fc8"),
            ("12", "c72b4ba82d1f51b71c8a18195ad33fc8"),
            ("13", "c72b4ba82d1f51b71c8a18195ad33fc8"),
            ("14", "c72b4ba82d1f51b71c8a18195ad33fc8"),
            ("15", "c72b4ba82d1f51b71c8a18195ad33fc8"),
            ("16", "c72b4ba82d1f51b71c8a18195ad33fc8"),
            ("17", "c72b4ba82d1f51b71c8a18195ad33fc8"),
            ("18", "cc4cb29100f1ce22515f3d44051f0c54"),
            ("19", "3ea3bc4448ff4b3f24dc4b904e9166f4"),
            ("20", "830438f8c407cbf6712b9bd07731e6e2"),
            ("21", "4c1088bf55ed131cf2ebf54b6283e0a2"),
            ("22", "eaf125854dcf1c4e071a5cddcaf37c37"), // 3 times
            ("23", "eaf125854dcf1c4e071a5cddcaf37c37"),
            ("24", "eaf125854dcf1c4e071a5cddcaf37c37"),
        ].iter().cloned().collect();
    }

    #[test]
    fn consider_only_interesting_ids() {
        let c = Cache::new(&mapping);
        assert_that!(
            &c.map.keys().collect::<Vec<_>>(),
            contains_in_any_order(&[
                "c72b4ba82d1f51b71c8a18195ad33fc8",
                "830438f8c407cbf6712b9bd07731e6e2",
                "eaf125854dcf1c4e071a5cddcaf37c37"
            ])
        );
    }

    #[test]
    fn unknown_and_ignored_entries() {
        let c = Cache::new(&mapping);
        // ID is present only once
        assert_eq!(c.query("091b34a64e50a123da8547a1b700c7e8"), Ignored);
        // ID is present multiple times
        assert_eq!(c.query("830438f8c407cbf6712b9bd07731e6e2"), Unknown);
    }

    #[test]
    fn memoize_data() {
        let mut c = Cache::new(&mapping);
        assert_eq!(c.query("eaf125854dcf1c4e071a5cddcaf37c37"), Unknown);
        c.memoize("eaf125854dcf1c4e071a5cddcaf37c37", &b"data"[..]);
        assert_eq!(
            c.query("eaf125854dcf1c4e071a5cddcaf37c37"),
            Known(b"data".to_vec())
        );
    }

    #[test]
    #[should_panic]
    fn memoize_ignored_chunk() {
        let mut c = Cache::new(&mapping);
        c.memoize("091b34a64e50a123da8547a1b700c7e8", &b"data"[..]);
    }

    #[test]
    fn memoize_zero_chunk() {
        let mut c = Cache::new(&mapping);
        c.memoize("c72b4ba82d1f51b71c8a18195ad33fc8", &ZERO_CHUNK);
        assert_eq!(c.query("c72b4ba82d1f51b71c8a18195ad33fc8"), KnownZero);
    }

}
