use crate::{Rope, Selection, TextRange, MarkedRangeId};
use slotmap::HopSlotMap;

#[derive(Debug, Clone)]
pub struct State {
    pub doc: Rope,
    pub selection: Selection,
    pub marked_ranges: HopSlotMap<MarkedRangeId, TextRange>,
}

impl State {
    #[must_use]
    pub fn new(doc: Rope) -> Self {
        Self {
            doc,
            selection: Selection::point(0),
            marked_ranges: HopSlotMap::default(),
        }
    }
}
