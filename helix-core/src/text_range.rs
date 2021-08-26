use core::fmt;
use std::{
    cmp::{self, Ordering},
    convert::{TryFrom, TryInto},
    ops::{self, Add, Index, Sub},
};

use ropey::{Rope, RopeSlice};

/// A range in text, represented as a pair of [`TextSize`][struct@TextSize].
///
/// It is a logic error for `start` to be greater than `end`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TextRange {
    // Invariant: start <= end
    start: u32,
    end: u32,
}

impl fmt::Debug for TextRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}..{:?}", self.start, self.end)
    }
}

/// Constructor methods
impl TextRange {
    /// Creates a new `TextRange` with the given `start` and `end` (`start..end`).
    ///
    /// # Panics
    ///
    /// Panics if `end < start`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use helix_core::text_size::*;
    /// let start = 5;
    /// let end = 10;
    /// let range = TextRange::new(start, end);
    ///
    /// assert_eq!(range.start(), start);
    /// assert_eq!(range.end(), end);
    /// assert_eq!(range.len(), end - start);
    /// ```
    #[inline]
    pub fn new(start: usize, end: usize) -> TextRange {
        let start = start.try_into().unwrap();
        let end = end.try_into().unwrap();
        assert!(start <= end);
        TextRange { start, end }
    }

    /// Create a new `TextRange` with the given `offset` and `len` (`offset..offset + len`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use helix_core::text_size::*;
    /// let text = "0123456789";
    ///
    /// let offset = 2;
    /// let length = 5;
    /// let range = TextRange::at(offset, length);
    ///
    /// assert_eq!(range, TextRange::new(offset, offset + length));
    /// assert_eq!(&text[range], "23456")
    /// ```
    #[inline]
    pub fn at(offset: usize, len: usize) -> TextRange {
        TextRange::new(offset, offset + len)
    }

    /// Create a zero-length range at the specified offset (`offset..offset`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use helix_core::text_size::*;
    /// let point: TextSize;
    /// point = 3;
    /// let range = TextRange::empty(point);
    /// assert!(range.is_empty());
    /// assert_eq!(range, TextRange::new(point, point));
    /// ```
    #[inline]
    pub fn empty(offset: usize) -> TextRange {
        TextRange::new(offset, offset)
    }

    /// Create a range up to the given end (`..end`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use helix_core::text_size::*;
    /// let point: TextSize;
    /// point = 12;
    /// let range = TextRange::up_to(point);
    ///
    /// assert_eq!(range.len(), point);
    /// assert_eq!(range, TextRange::new(0, point));
    /// assert_eq!(range, TextRange::at(0, point));
    /// ```
    #[inline]
    pub fn up_to(end: usize) -> TextRange {
        TextRange::new(0, end)
    }
}

/// Identity methods.
impl TextRange {
    /// The start point of this range.
    #[inline]
    pub const fn start(self) -> usize {
        self.start as usize
    }

    /// The end point of this range.
    #[inline]
    pub const fn end(self) -> usize {
        self.end as usize
    }

    /// The size of this range.
    #[inline]
    pub const fn len(self) -> usize {
        // HACK for const fn: math on primitives only
        self.end() - self.start()
    }

    /// Check if this range is empty.
    #[inline]
    pub const fn is_empty(self) -> bool {
        // HACK for const fn: math on primitives only
        self.start() == self.end()
    }
}

/// Manipulation methods.
impl TextRange {
    #[inline]
    pub fn with_start(self, start: usize) -> Self {
        TextRange::new(start, self.end())
    }

    #[inline]
    pub fn with_end(self, end: usize) -> Self {
        TextRange::new(self.start(), end)
    }

    #[inline]
    pub fn set_start(&mut self, start: usize) {
        self.start = start.try_into().unwrap();
    }

    #[inline]
    pub fn set_end(&mut self, end: usize) {
        self.end = end.try_into().unwrap();
    }

    /// Check if this range contains an offset.
    ///
    /// The end index is considered excluded.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use helix_core::text_size::*;
    /// let (start, end): (TextSize, TextSize);
    /// start = 10; end = 20;
    /// let range = TextRange::new(start, end);
    /// assert!(range.contains(start));
    /// assert!(!range.contains(end));
    /// ```
    #[inline]
    pub fn contains(self, offset: usize) -> bool {
        self.start() <= offset && offset < self.end()
    }

    /// Check if this range contains an offset.
    ///
    /// The end index is considered included.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use helix_core::text_size::*;
    /// let (start, end): (TextSize, TextSize);
    /// # start = 10; end = 20;
    /// let range = TextRange::new(start, end);
    /// assert!(range.contains_inclusive(start));
    /// assert!(range.contains_inclusive(end));
    /// ```
    #[inline]
    pub fn contains_inclusive(self, offset: usize) -> bool {
        self.start() <= offset && offset <= self.end()
    }

    /// Check if this range completely contains another range.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use helix_core::text_size::*;
    /// let larger = TextRange::new(0, 20);
    /// let smaller = TextRange::new(5, 15);
    /// assert!(larger.contains_range(smaller));
    /// assert!(!smaller.contains_range(larger));
    ///
    /// // a range always contains itself
    /// assert!(larger.contains_range(larger));
    /// assert!(smaller.contains_range(smaller));
    /// ```
    #[inline]
    pub fn contains_range(self, other: TextRange) -> bool {
        self.start() <= other.start() && other.end() <= self.end()
    }

    /// The range covered by both ranges, if it exists.
    /// If the ranges touch but do not overlap, the output range is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use helix_core::text_size::*;
    /// assert_eq!(
    ///     TextRange::intersect(
    ///         TextRange::new(0, 10),
    ///         TextRange::new(5, 15),
    ///     ),
    ///     Some(TextRange::new(5, 10)),
    /// );
    ///
    /// assert_eq!(
    ///     TextRange::intersect(
    ///         TextRange::new(5, 15),
    ///         TextRange::new(0, 10),
    ///     ),
    ///     Some(TextRange::new(5, 10)),
    /// );
    ///
    /// // They touch but they do not overlap
    /// assert_eq!(
    ///     TextRange::intersect(
    ///         TextRange::new(5, 10),
    ///         TextRange::new(10, 15),
    ///     ),
    ///     Some(TextRange::new(10, 10)),
    /// );
    /// ```
    #[inline]
    pub fn intersect(self, other: TextRange) -> Option<TextRange> {
        let start = cmp::max(self.start(), other.start());
        let end = cmp::min(self.end(), other.end());
        if end < start {
            return None;
        }
        Some(TextRange::new(start, end))
    }

    /// Extends the range to cover `other` as well.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use helix_core::text_size::*;
    /// assert_eq!(
    ///     TextRange::cover(
    ///         TextRange::new(0, 5),
    ///         TextRange::new(15, 20),
    ///     ),
    ///     TextRange::new(0, 20),
    /// );
    /// ```
    #[inline]
    pub fn cover(self, other: TextRange) -> TextRange {
        let start = cmp::min(self.start(), other.start());
        let end = cmp::max(self.end(), other.end());
        TextRange::new(start, end)
    }

    /// Extends the range to cover `other` offsets as well.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use helix_core::text_size::*;
    /// assert_eq!(
    ///     TextRange::empty(0).cover_offset(20),
    ///     TextRange::new(0, 20),
    /// )
    /// ```
    #[inline]
    pub fn cover_offset(self, offset: usize) -> TextRange {
        self.cover(TextRange::empty(offset))
    }

    /// Add an offset to this range.
    ///
    /// Note that this is not appropriate for changing where a `TextRange` is
    /// within some string; rather, it is for changing the reference anchor
    /// that the `TextRange` is measured against.
    ///
    /// The unchecked version (`Add::add`) will _always_ panic on overflow,
    /// in contrast to primitive integers, which check in debug mode only.
    #[inline]
    pub fn checked_add(self, offset: usize) -> Option<TextRange> {
        let offset = offset.try_into().unwrap();
        Some(TextRange {
            start: self.start.checked_add(offset)?,
            end: self.end.checked_add(offset)?,
        })
    }

    /// Subtract an offset from this range.
    ///
    /// Note that this is not appropriate for changing where a `TextRange` is
    /// within some string; rather, it is for changing the reference anchor
    /// that the `TextRange` is measured against.
    ///
    /// The unchecked version (`Sub::sub`) will _always_ panic on overflow,
    /// in contrast to primitive integers, which check in debug mode only.
    #[inline]
    pub fn checked_sub(self, offset: usize) -> Option<TextRange> {
        let offset = offset.try_into().unwrap();
        Some(TextRange {
            start: self.start.checked_sub(offset)?,
            end: self.end.checked_sub(offset)?,
        })
    }

    /// Relative order of the two ranges (overlapping ranges are considered
    /// equal).
    ///
    ///
    /// This is useful when, for example, binary searching an array of disjoint
    /// ranges.
    ///
    /// # Examples
    ///
    /// ```
    /// use helix_core::text_size::*;
    /// # use std::cmp::Ordering;
    ///
    /// let a = TextRange::new(0, 3);
    /// let b = TextRange::new(4, 5);
    /// assert_eq!(a.ordering(b), Ordering::Less);
    ///
    /// let a = TextRange::new(0, 3);
    /// let b = TextRange::new(3, 5);
    /// assert_eq!(a.ordering(b), Ordering::Less);
    ///
    /// let a = TextRange::new(0, 3);
    /// let b = TextRange::new(2, 5);
    /// assert_eq!(a.ordering(b), Ordering::Equal);
    ///
    /// let a = TextRange::new(0, 3);
    /// let b = TextRange::new(2, 2);
    /// assert_eq!(a.ordering(b), Ordering::Equal);
    ///
    /// let a = TextRange::new(2, 3);
    /// let b = TextRange::new(2, 2);
    /// assert_eq!(a.ordering(b), Ordering::Greater);
    /// ```
    #[inline]
    pub fn ordering(self, other: TextRange) -> Ordering {
        if self.end() <= other.start() {
            Ordering::Less
        } else if other.end() <= self.start() {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl TextRange {
    pub fn into_usize_range(self) -> ops::Range<usize> {
        let start = self.start() as usize;
        let end = self.end() as usize;
        start..end
    }

    pub fn slice(self, rope: &Rope) -> RopeSlice<'_> {
        rope.slice(self.into_usize_range())
    }

    pub fn start_empty(self) -> TextRange {
        TextRange::empty(self.start())
    }

    pub fn end_point(self) -> TextRange {
        TextRange::empty(self.end())
    }
}

impl From<TextRange> for ops::Range<usize> {
    #[inline]
    fn from(r: TextRange) -> Self {
        r.start()..r.end()
    }
}

impl From<TextRange> for (usize, usize) {
    #[inline]
    fn from(r: TextRange) -> (usize, usize) {
        (r.start(), r.end())
    }
}

impl From<(usize, usize)> for TextRange {
    #[inline]
    fn from(r: (usize, usize)) -> Self {
        TextRange::new(r.0, r.1)
    }
}

impl From<ops::Range<usize>> for TextRange {
    #[inline]
    fn from(r: ops::Range<usize>) -> Self {
        TextRange::new(r.start, r.end)
    }
}

impl Add<usize> for TextRange {
    type Output = TextRange;

    fn add(self, rhs: usize) -> Self::Output {
        let rhs = u32::try_from(rhs).unwrap();
        TextRange {
            start: self.start + rhs,
            end: self.end + rhs,
        }
    }
}

impl Sub<usize> for TextRange {
    type Output = TextRange;

    fn sub(self, rhs: usize) -> Self::Output {
        let rhs = u32::try_from(rhs).unwrap();
        TextRange {
            start: self.start - rhs,
            end: self.end - rhs,
        }
    }
}

impl Index<TextRange> for str {
    type Output = str;

    fn index(&self, index: TextRange) -> &Self::Output {
        &self[ops::Range::<usize>::from(index)]
    }
}
