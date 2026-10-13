//! Keyboard-driven cursors, in the spirit of the terminal frontend.

/// Selection over `len` rows that wraps at both ends.
///
/// The rows themselves live with the data that owns them; this only tracks
/// which one is selected and whether the keyboard moved it since the renderer
/// last looked (so the view can scroll to it once).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Selection {
    /// Number of rows.
    pub len: usize,
    /// Index of the selected row, if any.
    pub selected: Option<usize>,
    moved: bool,
}

impl Selection {
    fn set(&mut self, index: Option<usize>) {
        self.selected = index.filter(|i| *i < self.len);
        self.moved = true;
    }

    /// Selects a row directly (mouse click).
    pub fn select(&mut self, index: usize) {
        self.set(Some(index));
    }

    /// Clears the selection.
    pub fn clear(&mut self) {
        self.set(None);
    }

    /// Selects the first row.
    pub fn first(&mut self) {
        self.set((self.len > 0).then_some(0));
    }

    /// Selects the last row.
    pub fn last(&mut self) {
        self.set(self.len.checked_sub(1));
    }

    /// Moves `amount` rows down, wrapping to the top.
    pub fn next(&mut self, amount: usize) {
        self.set(match self.selected {
            Some(i) if i.saturating_add(amount) < self.len => Some(i + amount),
            _ => (self.len > 0).then_some(0),
        });
    }

    /// Moves `amount` rows up, wrapping to the bottom.
    pub fn previous(&mut self, amount: usize) {
        self.set(match self.selected {
            Some(0) | None => self.len.checked_sub(1),
            Some(i) => Some(i.saturating_sub(amount)),
        });
    }

    /// Whether the keyboard moved the selection since the last call.
    pub fn take_moved(&mut self) -> bool {
        std::mem::take(&mut self.moved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selection(len: usize) -> Selection {
        Selection {
            len,
            selected: (len > 0).then_some(0),
            moved: true,
        }
    }

    #[test]
    fn navigation_wraps() {
        let mut sel = selection(3);
        assert_eq!(sel.selected, Some(0));
        assert!(sel.take_moved());
        assert!(!sel.take_moved());
        sel.next(1);
        sel.next(1);
        assert_eq!(sel.selected, Some(2));
        sel.next(1);
        assert_eq!(sel.selected, Some(0));
        sel.previous(1);
        assert_eq!(sel.selected, Some(2));
        sel.previous(5);
        assert_eq!(sel.selected, Some(0));
        sel.last();
        assert_eq!(sel.selected, Some(2));
        sel.first();
        assert_eq!(sel.selected, Some(0));
        sel.clear();
        assert_eq!(sel.selected, None);
    }

    #[test]
    fn empty_selection() {
        let mut sel = Selection::default();
        sel.next(1);
        sel.previous(1);
        sel.first();
        sel.last();
        assert_eq!(sel.selected, None);
    }
}
