use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use slotmap::HopSlotMap;

use crate::TextRange;

slotmap::new_key_type! { pub struct MarkedRangeId; }

#[derive(Debug, Default)]
pub struct MarkedRanges {
    pub(crate) slotmap: Rc<RefCell<HopSlotMap<MarkedRangeId, TextRange>>>,
    pub(crate) sorted: Vec<MarkedRangeId>,
    should_sort: bool,
    should_reset: bool,
}

impl MarkedRanges {
    pub fn insert(&mut self, range: TextRange) -> MarkedRangeId {
        let id = self.slotmap.as_ref().borrow_mut().insert(range);
        self.sorted.push(id);
        self.should_sort = true;
        id
    }

    pub fn remove(&mut self, id: MarkedRangeId) -> Option<TextRange> {
        let range = self.slotmap.as_ref().borrow_mut().remove(id)?;
        self.should_reset = true;
        self.should_sort = true;
        Some(range)
    }

    pub(crate) fn invariants(&mut self) {
        let slotmap = self.slotmap.as_ref().borrow();

        if self.should_reset {
            self.sorted.clear();
            self.sorted.extend(slotmap.keys())
        }

        if self.should_sort {
            self.sorted
                .sort_unstable_by_key(|id| slotmap.get(*id).unwrap())
        }

        self.should_reset = false;
        self.should_sort = false;
    }

    pub(crate) fn iter(&mut self) -> impl Iterator<Item = (MarkedRangeId, TextRange)> + '_ {
        self.invariants();
        self.sorted.iter().copied().flat_map({
            let slotmap = Rc::clone(&self.slotmap);
            move |id| {
                slotmap
                    .as_ref()
                    .borrow()
                    .get(id)
                    .copied()
                    .map(|it| (id, it))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let mut marked_ranges = MarkedRanges::default();
        let i = marked_ranges.iter();
        let c: Vec<_> = i.collect();
        // marked_ranges.insert((0, 1).into());
    }
}
