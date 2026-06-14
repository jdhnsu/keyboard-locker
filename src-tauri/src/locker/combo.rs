use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ComboTracker {
    sequence: Vec<u32>,
    current: usize,
    pressed_keys: HashSet<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComboResult {
    InProgress,
    Matched,
    Reset,
}

impl ComboTracker {
    pub fn new(sequence: Vec<u32>) -> Self {
        ComboTracker {
            sequence,
            current: 0,
            pressed_keys: HashSet::new(),
        }
    }

    pub fn feed_key_press(&mut self, key_code: u32) -> ComboResult {
        self.pressed_keys.insert(key_code);

        if self.current < self.sequence.len() && key_code == self.sequence[self.current] {
            let all_prior_pressed = self.sequence[..self.current]
                .iter()
                .all(|k| self.pressed_keys.contains(k));

            if all_prior_pressed {
                self.current += 1;
                if self.current >= self.sequence.len() {
                    self.reset();
                    return ComboResult::Matched;
                }
                return ComboResult::InProgress;
            }
        }

        self.reset();
        ComboResult::Reset
    }

    pub fn feed_key_release(&mut self, key_code: u32) {
        self.pressed_keys.remove(&key_code);
    }

    pub fn reset(&mut self) {
        self.current = 0;
        self.pressed_keys.clear();
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.current, self.sequence.len())
    }

    pub fn is_idle(&self) -> bool {
        self.current == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sequence() {
        let mut tracker = ComboTracker::new(vec![29, 56, 38]); // Ctrl, Alt, L
        assert_eq!(tracker.feed_key_press(29), ComboResult::InProgress);
        assert_eq!(tracker.feed_key_press(56), ComboResult::InProgress);
        assert_eq!(tracker.feed_key_press(38), ComboResult::Matched);
    }

    #[test]
    fn test_wrong_key_resets() {
        let mut tracker = ComboTracker::new(vec![29, 56, 38]);
        tracker.feed_key_press(29);
        assert_eq!(tracker.feed_key_press(99), ComboResult::Reset);
        assert!(tracker.is_idle());
    }

    #[test]
    fn test_out_of_order_resets() {
        let mut tracker = ComboTracker::new(vec![29, 56, 38]);
        assert_eq!(tracker.feed_key_press(56), ComboResult::Reset);
        assert!(tracker.is_idle());
        assert_eq!(tracker.feed_key_press(29), ComboResult::InProgress);
    }

    #[test]
    fn test_release_does_not_reset() {
        let mut tracker = ComboTracker::new(vec![29, 56, 38]);
        tracker.feed_key_press(29);
        assert_eq!(tracker.current, 1);
        tracker.feed_key_release(29);
        assert_eq!(tracker.current, 1); // progress preserved
    }
}