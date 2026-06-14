use std::sync::Arc;
use parking_lot::RwLock;

use crate::locker::combo::ComboTracker;
use crate::locker::rules::Config;

#[derive(Debug)]
pub struct EngineState {
    pub locked: bool,
    pub config: Config,
    pub active_app: Option<String>,
    pub combo_tracker: ComboTracker,
    pub total_blocked: u64,
    pub total_allowed: u64,
    pub grab_active: bool,
}

impl EngineState {
    pub fn new(config: Config) -> Self {
        let combo_sequence = config.unlock_combo.clone();
        EngineState {
            locked: false,
            config,
            active_app: None,
            combo_tracker: ComboTracker::new(combo_sequence),
            total_blocked: 0,
            total_allowed: 0,
            grab_active: false,
        }
    }
}

pub type SharedState = Arc<RwLock<EngineState>>;