//! A module for working with items that can either occur once
//! or multiple times.
//!
//! Especially useful for working with configuration data where,
//! for convenience, the user is often allowed to either specify
//! a single configuration item directly or a list of items.
//!
//! ## Example .travis.yml
//! ```yaml
//! language: rust
//! rust: stable
//! ```
//! vs.
//!
//! ```yaml
//! language: rust
//! rust:
//!  - stable
//!  - beta
//!  - nightly
//! ```

/// An iterator over a `Multi`s values.
#[derive(Debug)]
pub struct IntoIter<T>(IntoIterInner<T>);

#[derive(Debug)]
enum IntoIterInner<T> {
    Single(::std::iter::Once<T>),
    Multiple(::std::vec::IntoIter<T>),
}

/// An iterator over references of a `Multi`s values.
#[derive(Debug)]
pub struct Iter<'a, T: 'a>(IterInner<'a, T>);

#[derive(Debug)]
enum IterInner<'a, T: 'a> {
    Single(::std::iter::Once<&'a T>),
    Multiple(::std::slice::Iter<'a, T>),
}

/// An iterator over mutable references of a `Multi`s values.
#[derive(Debug)]
pub struct IterMut<'a, T: 'a>(IterMutInner<'a, T>);

#[derive(Debug)]
enum IterMutInner<'a, T: 'a> {
    Single(::std::iter::Once<&'a mut T>),
    Multiple(::std::slice::IterMut<'a, T>),
}

/// A wrapper for a value that can either occur once or multiple times.
///
/// This is mainly used with serialization formats such as yaml, where
/// the user usually has the option of specifying one single option directly
/// (without wrapping it in a list) or specifying multiple items.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Multi<T> {
    Single(T),
    Multiple(Vec<T>),
}

impl<T> Multi<T> {
    pub fn is_multiple(&self) -> bool {
        match *self {
            Multi::Multiple(_) => true,
            Multi::Single(_) => false,
        }
    }

    pub fn is_single(&self) -> bool {
        !self.is_multiple()
    }

    pub fn unwrap_multiple(self) -> Vec<T> {
        match self {
            Multi::Multiple(items) => items,
            _ => panic!("unwrap_multiple called on Multi with single value"),
        }
    }

    pub fn unwrap_single(self) -> T {
        match self {
            Multi::Single(item) => item,
            _ => panic!("unwrap_single called on Multi with multiple values"),
        }
    }
}

impl<'a, T: 'a> Multi<T> {
    pub fn iter(&'a self) -> Iter<'a, T> {
        self.into_iter()
    }

    pub fn iter_mut(&'a mut self) -> IterMut<'a, T> {
        self.into_iter()
    }
}

impl<T> IntoIterator for Multi<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Multi::Multiple(items) => IntoIter(IntoIterInner::Multiple(items.into_iter())),
            Multi::Single(item) => IntoIter(IntoIterInner::Single(::std::iter::once(item))),
        }
    }
}

impl<'a, T: 'a> IntoIterator for &'a Multi<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        match *self {
            Multi::Multiple(ref items) => Iter(IterInner::Multiple(items.iter())),
            Multi::Single(ref item) => Iter(IterInner::Single(::std::iter::once(item))),
        }
    }
}

impl<'a, T: 'a> IntoIterator for &'a mut Multi<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        match *self {
            Multi::Multiple(ref mut items) => IterMut(IterMutInner::Multiple(items.iter_mut())),
            Multi::Single(ref mut item) => IterMut(IterMutInner::Single(::std::iter::once(item))),
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            IntoIterInner::Multiple(ref mut it) => it.next(),
            IntoIterInner::Single(ref mut it) => it.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.0 {
            IntoIterInner::Multiple(ref it) => it.size_hint(),
            IntoIterInner::Single(ref it) => it.size_hint(),
        }
    }
}

impl<T> ExactSizeIterator for IntoIter<T> { }

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            IterInner::Multiple(ref mut it) => it.next(),
            IterInner::Single(ref mut it) => it.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.0 {
            IterInner::Multiple(ref it) => it.size_hint(),
            IterInner::Single(ref it) => it.size_hint(),
        }
    }
}

impl<'a, T: 'a> ExactSizeIterator for Iter<'a, T> { }

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            IterMutInner::Multiple(ref mut it) => it.next(),
            IterMutInner::Single(ref mut it) => it.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.0 {
            IterMutInner::Multiple(ref it) => it.size_hint(),
            IterMutInner::Single(ref it) => it.size_hint(),
        }
    }
}

impl<'a, T: 'a> ExactSizeIterator for IterMut<'a, T> { }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterate_multiple_value() {
        let multi = Multi::Multiple(vec![1, 2, 3]);

        assert_eq!(multi.clone().into_iter().len(), 3);

        let mut items_seen = 0;
        for _ in multi {
            items_seen += 1;
        }
        assert_eq!(items_seen, 3);
    }

    #[test]
    fn iterate_one_value() {
        let multi = Multi::Single(1);

        assert_eq!(multi.clone().into_iter().len(), 1);

        let mut items_seen = 0;
        for _ in multi {
            items_seen += 1;
        }
        assert_eq!(items_seen, 1);
    }
}
