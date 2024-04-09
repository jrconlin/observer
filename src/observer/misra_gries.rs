use std::{error::Error, fmt};

use crate::observer::counter::Counter;
use backtrace::Backtrace;
use thiserror::Error;

// == Error stuff
#[derive(Debug)]
pub struct MGError {
    kind: MGErrorKind,
    backtrace: Backtrace,
}

#[derive(Clone, Eq, PartialEq, Debug, Error)]
pub enum MGErrorKind {
    #[error("General error {:?}", _0)]
    General(String),
}

impl MGError {
    pub fn kind(&self) -> &MGErrorKind {
        &self.kind
    }

    pub fn general(msg: &str) -> Self {
        MGErrorKind::General(msg.to_owned()).into()
    }
}

impl fmt::Display for MGError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}\nBacktrace:\n{:?}", self.kind, self.backtrace)?;

        // Go down the chain of errors
        let mut error: &dyn Error = &self.kind;
        while let Some(source) = error.source() {
            write!(f, "\n\nCaused by: {}", source)?;
            error = source;
        }

        Ok(())
    }
}

impl<T> From<T> for MGError
where
    MGErrorKind: From<T>,
{
    fn from(item: T) -> Self {
        MGError {
            kind: MGErrorKind::from(item),
            backtrace: Backtrace::new(),
        }
    }
}

impl Error for MGError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.kind.source()
    }
}

/// Return the approximate high frequency items for a stream.
/// This uses the Misra-Gries algorithm, with a limiter, meaning that
/// the algorithm will only track the top N items. As such, "high frequency"
/// is determined by the number of times the item is seen within that
/// limit. Some caution and tweaking may be required to ensure that the
/// limit is generous enough to capture high frequency items, yet small
/// enough not to consume all memory.
///
/// For instance, let's suppose we have a limit of 4 and a string like
/// "aaabbcde". The algorithm will only track the top 4 items. This means
/// that by the time that we read "e", we start running out of room and
/// start decrementing "old" values. Since "a" was near the start of the
/// set, it's decremented, along with b. Single items are removed as
/// infrequent. This means the final count will be [("a", 2), ("b", 1)],
/// even though those characters appeared more frequently in the total
/// stream.
///
/// This tool is really just about finding the most common keys within the
/// window of the limit within a given stream. The counts are more an
/// approximation and not a strict definition. (If you wish to have
/// a more accurate count, set the limit higher.)
#[derive(Clone, Debug)]
struct MisraGries {
    /// The collection of items we're tracking
    counter: Counter<String>,
    /// The limit of the size of the collection of items we're tracking.
    limit: usize,
}

impl MisraGries {
    /// Create a new Misra-Gries counter with a limit.
    pub fn new(limit: usize) -> Self {
        Self {
            counter: Counter::new(),
            limit,
        }
    }

    /// Return the `count` most frequently seen items.
    pub fn top(self, count: usize) -> Vec<(String, i64)> {
        self.counter
            .k_most_common_ordered(count)
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect()
    }

    /// Remove `count` least frequently seen items. (removes `count` items, not items seen less than `count` times)
    pub fn purge(mut self, count: usize) {
        let total = self.counter.len();
        let purge_amount = if count > total { total } else { count };
        let all = self
            .counter
            .most_common_ordered()
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect::<Vec<(String, i64)>>();
        for (cnt, item) in all.iter().rev().enumerate() {
            if cnt > purge_amount {
                break;
            }
            self.counter.remove(&item.0);
        }
    }
}

impl MisraGries {
    /// Consider a string for frequency counting.
    pub fn track(&mut self, item: &str) -> bool {
        let clone = self.clone();
        let keys = clone.counter.keys();
        let string = item.to_owned();
        if keys.contains(&&string) || keys.len() < self.limit {
            self.counter.add(string);
            true
        } else {
            for key in keys {
                let kk = key.to_string();
                self.counter.decrease(kk.clone());
                if let Some(cnt) = self.counter.get(&kk) {
                    if *cnt < 1 {
                        self.counter.remove(&kk);
                    }
                }
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::result::Result;

    use super::*;

    #[test]
    fn test_mg() -> Result<(), MGError> {
        let mut mg = MisraGries::new(10);

        // if our window is 10 items, we should ensure that the values appear
        // no more than 10 characters apart.
        let items = "acbacbdefagh".chars().collect::<Vec<char>>();
        for item in items {
            mg.track(&item.to_string());
        }
        assert_eq!(mg.counter.len(), 8);
        assert_eq!(mg.counter.get(&"a".to_owned()), Some(&3));
        assert_eq!(mg.counter.get(&"b".to_owned()), Some(&2));
        assert_eq!(mg.counter.get(&"c".to_owned()), Some(&2));

        Ok(())
    }
}
