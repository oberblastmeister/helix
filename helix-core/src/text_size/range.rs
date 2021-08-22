use core::fmt;
use std::{
    cmp::{self, Ordering},
    convert::TryFrom,
    ops::{self, Add, Deref, DerefMut, Sub},
};

use super::size::TextSize;

/// A range in text, represented as a pair of [`TextSize`][struct@TextSize].
///
/// It is a logic error for `start` to be greater than `end`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TextRange {
    // Invariant: start <= end
    start: TextSize,
    end: TextSize,
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
    /// # use text_size::*;
    /// let start = TextSize::from(5);
    /// let end = TextSize::from(10);
    /// let range = TextRange::new(start, end);
    ///
    /// assert_eq!(range.start(), start);
    /// assert_eq!(range.end(), end);
    /// assert_eq!(range.len(), end - start);
    /// ```
    #[inline]
    pub fn new<T: Into<TextSize>, U: Into<TextSize>>(start: T, end: U) -> TextRange {
        let start = start.into();
        let end = end.into();
        assert!(start <= end);
        TextRange { start, end }
    }

    /// Create a new `TextRange` with the given `offset` and `len` (`offset..offset + len`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use text_size::*;
    /// let text = "0123456789";
    ///
    /// let offset = TextSize::from(2);
    /// let length = TextSize::from(5);
    /// let range = TextRange::at(offset, length);
    ///
    /// assert_eq!(range, TextRange::new(offset, offset + length));
    /// assert_eq!(&text[range], "23456")
    /// ```
    #[inline]
    pub fn at<T: Into<TextSize>, U: Into<TextSize>>(offset: T, len: U) -> TextRange {
        let offset = offset.into();
        let len = len.into();
        TextRange::new(offset, offset + len)
    }

    /// Create a zero-length range at the specified offset (`offset..offset`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use text_size::*;
    /// let point: TextSize;
    /// # point = TextSize::from(3);
    /// let range = TextRange::empty(point);
    /// assert!(range.is_empty());
    /// assert_eq!(range, TextRange::new(point, point));
    /// ```
    #[inline]
    pub fn empty<T: Into<TextSize>>(offset: T) -> TextRange {
        let offset = offset.into();
        TextRange::new(offset, offset)
    }

    /// Create a range up to the given end (`..end`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use text_size::*;
    /// let point: TextSize;
    /// # point = TextSize::from(12);
    /// let range = TextRange::up_to(point);
    ///
    /// assert_eq!(range.len(), point);
    /// assert_eq!(range, TextRange::new(0.into(), point));
    /// assert_eq!(range, TextRange::at(0.into(), point));
    /// ```
    #[inline]
    pub fn up_to<T: Into<TextSize>>(end: T) -> TextRange {
        TextRange::new(0, end)
    }
}

/// Identity methods.
impl TextRange {
    /// The start point of this range.
    #[inline]
    pub const fn start(self) -> TextSize {
        self.start
    }

    /// The end point of this range.
    #[inline]
    pub const fn end(self) -> TextSize {
        self.end
    }

    /// The size of this range.
    #[inline]
    pub const fn len(self) -> TextSize {
        // HACK for const fn: math on primitives only
        TextSize {
            raw: self.end().raw - self.start().raw,
        }
    }

    /// Check if this range is empty.
    #[inline]
    pub const fn is_empty(self) -> bool {
        // HACK for const fn: math on primitives only
        self.start().raw == self.end().raw
    }
}

/// Manipulation methods.
impl TextRange {
    /// Check if this range contains an offset.
    ///
    /// The end index is considered excluded.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use text_size::*;
    /// let (start, end): (TextSize, TextSize);
    /// # start = 10.into(); end = 20.into();
    /// let range = TextRange::new(start, end);
    /// assert!(range.contains(start));
    /// assert!(!range.contains(end));
    /// ```
    #[inline]
    pub fn contains<T: Into<TextSize>>(self, offset: T) -> bool {
        let offset = offset.into();
        self.start() <= offset && offset < self.end()
    }

    /// Check if this range contains an offset.
    ///
    /// The end index is considered included.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use text_size::*;
    /// let (start, end): (TextSize, TextSize);
    /// # start = 10.into(); end = 20.into();
    /// let range = TextRange::new(start, end);
    /// assert!(range.contains_inclusive(start));
    /// assert!(range.contains_inclusive(end));
    /// ```
    #[inline]
    pub fn contains_inclusive<T: Into<TextSize>>(self, offset: T) -> bool {
        let offset = offset.into();
        self.start() <= offset && offset <= self.end()
    }

    /// Check if this range completely contains another range.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use text_size::*;
    /// let larger = TextRange::new(0.into(), 20.into());
    /// let smaller = TextRange::new(5.into(), 15.into());
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
    /// # use text_size::*;
    /// assert_eq!(
    ///     TextRange::intersect(
    ///         TextRange::new(0.into(), 10.into()),
    ///         TextRange::new(5.into(), 15.into()),
    ///     ),
    ///     Some(TextRange::new(5.into(), 10.into())),
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
    /// # use text_size::*;
    /// assert_eq!(
    ///     TextRange::cover(
    ///         TextRange::new(0.into(), 5.into()),
    ///         TextRange::new(15.into(), 20.into()),
    ///     ),
    ///     TextRange::new(0.into(), 20.into()),
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
    /// # use text_size::*;
    /// assert_eq!(
    ///     TextRange::empty(0.into()).cover_offset(20.into()),
    ///     TextRange::new(0.into(), 20.into()),
    /// )
    /// ```
    #[inline]
    pub fn cover_offset<T: Into<TextSize>>(self, offset: T) -> TextRange {
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
    pub fn checked_add<T: Into<TextSize>>(self, offset: T) -> Option<TextRange> {
        let offset = offset.into();
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
    pub fn checked_sub<T: Into<TextSize>>(self, offset: T) -> Option<TextRange> {
        let offset = offset.into();
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
    /// # use text_size::*;
    /// # use std::cmp::Ordering;
    ///
    /// let a = TextRange::new(0.into(), 3.into());
    /// let b = TextRange::new(4.into(), 5.into());
    /// assert_eq!(a.ordering(b), Ordering::Less);
    ///
    /// let a = TextRange::new(0.into(), 3.into());
    /// let b = TextRange::new(3.into(), 5.into());
    /// assert_eq!(a.ordering(b), Ordering::Less);
    ///
    /// let a = TextRange::new(0.into(), 3.into());
    /// let b = TextRange::new(2.into(), 5.into());
    /// assert_eq!(a.ordering(b), Ordering::Equal);
    ///
    /// let a = TextRange::new(0.into(), 3.into());
    /// let b = TextRange::new(2.into(), 2.into());
    /// assert_eq!(a.ordering(b), Ordering::Equal);
    ///
    /// let a = TextRange::new(2.into(), 3.into());
    /// let b = TextRange::new(2.into(), 2.into());
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

impl<T> From<TextRange> for ops::Range<T>
where
    T: From<TextSize>,
{
    #[inline]
    fn from(r: TextRange) -> Self {
        r.start.into()..r.end.into()
    }
}

impl<T> From<ops::Range<T>> for TextRange
where
    T: Into<TextSize>,
{
    fn from(r: ops::Range<T>) -> Self {
        TextRange {
            start: r.start.into(),
            end: r.end.into(),
        }
    }
}

impl From<&TextRange> for TextRange {
    fn from(value: &TextRange) -> Self {
        *value
    }
}

impl<T> Add<T> for TextRange
where
    TextSize: Add<T, Output = TextSize>,
    T: Copy,
{
    type Output = TextRange;

    fn add(self, rhs: T) -> Self::Output {
        TextRange {
            start: self.start + rhs,
            end: self.end + rhs,
        }
    }
}

impl<T> Sub<T> for TextRange
where
    TextSize: Sub<T, Output = TextSize>,
    T: Copy,
{
    type Output = TextRange;

    fn sub(self, rhs: T) -> Self::Output {
        TextRange {
            start: self.start - rhs,
            end: self.end - rhs,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TextRange1(TextRange);

impl Deref for TextRange1 {
    type Target = TextRange;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TextRange1 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TextRange1 {
    pub fn new<T: Into<TextSize>, U: Into<TextSize>>(start: T, end: U) -> TextRange1 {
        let inner = TextRange::new(start, end);
        Self::assert(inner)
    }

    /// lifted into
    pub fn into1<T: From<TextRange>>(self) -> T {
        TextRange::from(self).into()
    }

    pub fn assert(inner: TextRange) -> TextRange1 {
        Self::try_from(inner).unwrap_or_else(|_| panic!("NonEmptyTextRange cannot be empty"))
    }
}

impl TryFrom<TextRange> for TextRange1 {
    type Error = ();

    fn try_from(inner: TextRange) -> Result<Self, Self::Error> {
        if inner.is_empty() {
            Err(())
        } else {
            Ok(TextRange1(inner))
        }
    }
}

impl From<TextRange1> for TextRange {
    fn from(value: TextRange1) -> Self {
        value.0
    }
}

impl From<&TextRange1> for TextRange {
    fn from(value: &TextRange1) -> Self {
        value.0
    }
}

impl<T> Add<T> for TextRange1
where
    TextSize: Add<T, Output = TextSize>,
    T: Copy,
{
    type Output = TextRange1;

    fn add(self, rhs: T) -> Self::Output {
        TextRange1(self.0 + rhs)
    }
}
