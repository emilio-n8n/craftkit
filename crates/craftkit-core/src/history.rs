//! Bounded undo/redo over whole-document snapshots.
//!
//! Snapshots remove an entire class of inverse-command bugs. The history is
//! bounded so a long session cannot grow without limit. Applications apply a
//! mutation through [`History::apply`] and get one undo step per gesture.

/// Default maximum number of undo steps retained.
pub const DEFAULT_LIMIT: usize = 100;

/// Document editor state with undo/redo and save-point tracking.
#[derive(Debug, Clone)]
pub struct History<T> {
    current: T,
    saved: T,
    undo: Vec<T>,
    redo: Vec<T>,
    limit: usize,
    gesture: Option<T>,
}

impl<T: Clone> History<T> {
    /// Wrap a document with the default history depth.
    #[must_use]
    pub fn new(current: T) -> Self {
        Self::with_limit(current, DEFAULT_LIMIT)
    }

    /// Wrap a document with a custom history depth (at least one step).
    #[must_use]
    pub fn with_limit(current: T, limit: usize) -> Self {
        Self {
            saved: current.clone(),
            current,
            undo: Vec::new(),
            redo: Vec::new(),
            limit: limit.max(1),
            gesture: None,
        }
    }

    /// Read the current document.
    #[must_use]
    pub fn current(&self) -> &T {
        &self.current
    }

    /// Mutable access for a continuous gesture.
    ///
    /// Prefer [`History::apply`] for one-shot actions. For typing or dragging,
    /// call [`History::begin_gesture`] before the first mutation, mutate through
    /// this accessor, then call [`History::end_gesture`] once at the end. The
    /// whole gesture becomes a single undo step.
    pub fn current_mut(&mut self) -> &mut T {
        &mut self.current
    }

    /// Start a continuous gesture, snapshotting the current state once.
    ///
    /// Calling this while a gesture is active is a no-op.
    pub fn begin_gesture(&mut self) {
        if self.gesture.is_none() {
            self.gesture = Some(self.current.clone());
        }
    }

    /// Finish a continuous gesture and record one undo step if it changed.
    pub fn end_gesture(&mut self)
    where
        T: PartialEq,
    {
        let Some(before) = self.gesture.take() else {
            return;
        };
        if self.current == before {
            return;
        }
        self.undo.push(before);
        if self.undo.len() > self.limit {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Apply a mutation, recording one undo step only when the value changes.
    ///
    /// Returns `true` if the document changed.
    pub fn apply(&mut self, mutate: impl FnOnce(&mut T)) -> bool
    where
        T: PartialEq,
    {
        self.end_gesture();
        let before = self.current.clone();
        mutate(&mut self.current);
        if self.current == before {
            return false;
        }
        self.undo.push(before);
        if self.undo.len() > self.limit {
            self.undo.remove(0);
        }
        self.redo.clear();
        true
    }

    /// Replace the document and reset history (New / Open / sample).
    pub fn replace(&mut self, value: T) {
        self.saved = value.clone();
        self.current = value;
        self.undo.clear();
        self.redo.clear();
        self.gesture = None;
    }

    /// Undo one step. Returns `true` if anything changed.
    pub fn undo(&mut self) -> bool {
        self.gesture = None;
        let Some(previous) = self.undo.pop() else {
            return false;
        };
        let current = std::mem::replace(&mut self.current, previous);
        self.redo.push(current);
        true
    }

    /// Redo one step. Returns `true` if anything changed.
    pub fn redo(&mut self) -> bool {
        self.gesture = None;
        let Some(next) = self.redo.pop() else {
            return false;
        };
        let current = std::mem::replace(&mut self.current, next);
        self.undo.push(current);
        true
    }

    /// Whether undo is available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether redo is available.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Depth of the undo stack.
    #[must_use]
    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    /// Depth of the redo stack.
    #[must_use]
    pub fn redo_depth(&self) -> usize {
        self.redo.len()
    }

    /// Mark the current document as the saved state.
    pub fn mark_saved(&mut self) {
        self.saved = self.current.clone();
    }

    /// Whether the current document differs from the last saved state.
    #[must_use]
    pub fn is_dirty(&self) -> bool
    where
        T: PartialEq,
    {
        self.current != self.saved
    }

    /// Change the maximum number of undo steps.
    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit.max(1);
        while self.undo.len() > self.limit {
            self.undo.remove(0);
        }
        if self.redo.len() > self.limit {
            self.redo.truncate(self.limit);
        }
    }

    /// Maximum number of undo steps.
    #[must_use]
    pub fn limit(&self) -> usize {
        self.limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_and_redo_round_trip() {
        let mut history = History::new(0_i32);
        assert!(history.apply(|value| *value = 1));
        assert!(history.apply(|value| *value = 2));
        assert_eq!(*history.current(), 2);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        assert!(history.undo());
        assert_eq!(*history.current(), 1);
        assert!(history.can_redo());

        assert!(history.redo());
        assert_eq!(*history.current(), 2);
    }

    #[test]
    fn unchanged_mutation_does_not_create_a_step() {
        let mut history = History::new(1_i32);
        assert!(!history.apply(|value| *value = 1));
        assert!(!history.can_undo());
    }

    #[test]
    fn history_is_bounded() {
        let mut history = History::with_limit(0_i32, 3);
        for value in 1..=10 {
            history.apply(|current| *current = value);
        }
        assert_eq!(history.undo_depth(), 3);
        assert_eq!(*history.current(), 10);
        while history.undo() {}
        assert_eq!(*history.current(), 7);
    }

    #[test]
    fn undo_back_to_saved_state_is_clean() {
        let mut history = History::new(0_i32);
        history.apply(|value| *value = 1);
        history.mark_saved();
        assert!(!history.is_dirty());

        history.apply(|value| *value = 2);
        assert!(history.is_dirty());

        history.undo();
        assert!(!history.is_dirty());
        assert_eq!(*history.current(), 1);
    }

    #[test]
    fn a_new_edit_drops_redo() {
        let mut history = History::new(0_i32);
        history.apply(|value| *value = 1);
        history.undo();
        assert!(history.can_redo());
        history.apply(|value| *value = 5);
        assert!(!history.can_redo());
    }

    #[test]
    fn a_gesture_becomes_one_undo_step() {
        let mut history = History::new(0_i32);
        history.begin_gesture();
        *history.current_mut() = 1;
        *history.current_mut() = 2;
        history.end_gesture();
        assert_eq!(history.undo_depth(), 1);
        assert_eq!(*history.current(), 2);
        history.undo();
        assert_eq!(*history.current(), 0);
    }

    #[test]
    fn an_unchanged_gesture_does_not_create_history() {
        let mut history = History::new(0_i32);
        history.begin_gesture();
        history.end_gesture();
        assert!(!history.can_undo());
    }

    #[test]
    fn replace_resets_history_and_save_point() {
        let mut history = History::new(0_i32);
        history.apply(|value| *value = 1);
        history.replace(42);
        assert_eq!(*history.current(), 42);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert!(!history.is_dirty());
    }
}
