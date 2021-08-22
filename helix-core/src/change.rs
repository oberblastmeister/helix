use std::{borrow::Cow, convert::TryInto, iter::FromIterator};

use crate::{
    text_size::{TextRange, TextSize},
    Tendril,
};
use ropey::Rope;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub delete: TextRange,
    pub insert: Tendril,
}

impl Change {
    fn new(delete: TextRange, insert: Tendril) -> Change {
        Change { delete, insert }
    }

    fn apply(&self, text: &mut Rope) {
        text.remove(self.delete.try_into_usize_range().unwrap());
        text.insert(self.delete.start().try_into().unwrap(), &self.insert)
    }

    fn add_offset(self, offset: i64) -> Self {
        Change {
            delete: TextRange::new(
                (self.delete.start() as i64 + offset).try_into().unwrap(),
                (self.delete.end() as i64 + offset).try_into().unwrap(),
            ),
            insert: self.insert,
        }
    }

    fn offset(&self) -> i64 {
        self.insert.len() as i64 - i64::from(self.delete.len())
    }

    fn invert(&self, original_text: &Rope) -> Self {
        let insert = Tendril::from_slice(&Cow::from(
            original_text.slice(self.delete.try_into_usize_range().unwrap()),
        ));
        let delete = TextRange::new(self.delete.start(), self.insert.len().try_into().unwrap());
        Change { delete, insert }
    }
}

#[derive(Default, Debug)]
pub struct ChangeSetBuilder {
    changes: Vec<Change>,
}

impl Extend<Change> for ChangeSetBuilder {
    fn extend<T: IntoIterator<Item = Change>>(&mut self, iter: T) {
        self.changes.extend(iter)
    }
}

impl FromIterator<Change> for ChangeSetBuilder {
    fn from_iter<T: IntoIterator<Item = Change>>(iter: T) -> Self {
        ChangeSetBuilder {
            changes: iter.into_iter().collect(),
        }
    }
}
impl ChangeSetBuilder {
    pub fn new() -> ChangeSetBuilder {
        Self::default()
    }

    pub fn push(&mut self, change: Change) {
        self.changes.push(change)
    }

    pub fn build(mut self) -> ChangeSet {
        assert_disjoint(&mut self.changes);
        self.build_unchecked()
    }

    pub fn build_unstable(mut self) -> ChangeSet {
        assert_disjoint_unstable(&mut self.changes);
        self.build_unchecked()
    }

    pub fn build_unchecked(self) -> ChangeSet {
        ChangeSet {
            changes: self.changes,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ChangeSet {
    changes: Vec<Change>,
}

impl ChangeSet {
    pub fn apply(self, text: &mut Rope) {
        let mut offset = 0.into();
        for change in self.changes {
            let change_offset = change.offset();
            change.add_offset(offset).apply(text);
            offset += change_offset;
        }
    }
}

impl FromIterator<Change> for ChangeSet {
    fn from_iter<T: IntoIterator<Item = Change>>(iter: T) -> Self {
        iter.into_iter().collect::<ChangeSetBuilder>().build()
    }
}

fn assert_disjoint(changes: &mut [Change]) {
    assert!(check_disjoint(changes), "Changes were not disjoint");
}

fn assert_disjoint_unstable(changes: &mut [Change]) {
    assert!(
        check_disjoint_unstable(changes),
        "Changes were not disjoint"
    )
}

fn check_disjoint_unstable(changes: &mut [Change]) -> bool {
    check_disjoint_impl(changes, true)
}

fn check_disjoint(changes: &mut [Change]) -> bool {
    check_disjoint_impl(changes, false)
}

fn check_disjoint_impl(changes: &mut [Change], unstable: bool) -> bool {
    fn key(change: &Change) -> (TextSize, TextSize) {
        (change.delete.start(), change.delete.end())
    }
    if unstable {
        changes.sort_unstable_by_key(key);
    } else {
        changes.sort_by_key(key);
    }
    changes
        .iter()
        .zip(changes.iter().skip(1))
        .all(|(l, r)| l.delete.end() <= r.delete.start())
}

#[cfg(test)]
mod tests {
    use std::array;

    use super::*;

    fn check_apply<T: Into<Rope>, U: Into<Rope>, W: Into<Tendril>, const N: usize>(
        changes: [(u32, u32, W); N],
        before: T,
        after: U,
    ) {
        let change_set: ChangeSet = array::IntoIter::new(changes)
            .map(|(start, end, contents)| Change::new((start..end).into(), contents.into()))
            .collect();
        let mut before = before.into();
        let after = after.into();
        change_set.apply(&mut before);
        assert_eq!(before, after);
    }

    #[test]
    fn test_apply() {
        check_apply(
            [(5, 6, "   "), (0, 0, "prefix "), (0, 0, "another ")],
            "hello world!",
            "prefix another hello   world!",
        );
    }

    #[should_panic]
    #[test]
    fn apply_not_disjoint() {
        check_apply(
            [(5, 6, "asdfasdf"), (5, 6, "asdfasd;fkas")],
            "asdpfoiuapdsiofuadpoif",
            "adspfoiuadf",
        );
    }

    #[should_panic]
    #[test]
    fn not_long_enough() {
        check_apply([(3, 4, "")], "", "");
    }
}