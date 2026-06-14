use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRule {
    pub key: u32,
    pub label: String,
    pub allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifiers: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRule {
    pub process_names: Vec<String>,
    #[serde(default)]
    pub rules: Vec<KeyRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub rules: Vec<KeyRule>,
    #[serde(default)]
    pub app_rules: Vec<AppRule>,
    #[serde(default)]
    pub auto_unlock_timeout: Option<u64>,
    pub unlock_combo: Vec<u32>,
    pub version: u32,
}

impl Default for Config {
    fn default() -> Self {
        let all_keys: &[(u32, &str)] = &[
            (0x1B, "Esc"),
            (0x70, "F1"), (0x71, "F2"), (0x72, "F3"), (0x73, "F4"),
            (0x74, "F5"), (0x75, "F6"), (0x76, "F7"), (0x77, "F8"),
            (0x78, "F9"), (0x79, "F10"), (0x7A, "F11"), (0x7B, "F12"),
            (0xC0, "`"), (0x31, "1"), (0x32, "2"), (0x33, "3"),
            (0x34, "4"), (0x35, "5"), (0x36, "6"), (0x37, "7"),
            (0x38, "8"), (0x39, "9"), (0x30, "0"), (0xBD, "-"), (0xBB, "="),
            (0x08, "Bksp"),
            (0x09, "Tab"),
            (0x51, "Q"), (0x57, "W"), (0x45, "E"), (0x52, "R"), (0x54, "T"),
            (0x59, "Y"), (0x55, "U"), (0x49, "I"), (0x4F, "O"), (0x50, "P"),
            (0xDB, "["), (0xDD, "]"), (0xDC, "\\"),
            (0x14, "CapsLock"),
            (0x41, "A"), (0x53, "S"), (0x44, "D"), (0x46, "F"), (0x47, "G"),
            (0x48, "H"), (0x4A, "J"), (0x4B, "K"), (0x4C, "L"),
            (0xBA, ";"), (0xDE, "'"),
            (0x0D, "Enter"),
            (0xA0, "LShift"), (0x5A, "Z"), (0x58, "X"), (0x43, "C"),
            (0x56, "V"), (0x42, "B"), (0x4E, "N"), (0x4D, "M"),
            (0xBC, ","), (0xBE, "."), (0xBF, "/"), (0xA1, "RShift"),
            (0xA2, "LCtrl"), (0x5B, "LWin"), (0xA4, "LAlt"),
            (0x20, "Space"),
            (0xA5, "RAlt"), (0x5C, "RWin"), (0xA3, "RCtrl"),
            (0x2D, "Insert"), (0x24, "Home"), (0x21, "PgUp"),
            (0x2E, "Delete"), (0x23, "End"), (0x22, "PgDn"),
            (0x26, "Up"), (0x28, "Down"), (0x25, "Left"), (0x27, "Right"),
            (0x90, "NumLk"), (0x91, "ScrLk"), (0x13, "Pause"),
            (0x60, "Num0"), (0x61, "Num1"), (0x62, "Num2"), (0x63, "Num3"), (0x64, "Num4"),
            (0x65, "Num5"), (0x66, "Num6"), (0x67, "Num7"), (0x68, "Num8"), (0x69, "Num9"),
            (0x6A, "Num*"), (0x6B, "Num+"),
            (0x6D, "Num-"), (0x6E, "Num."), (0x6F, "Num/"),
        ];

        let allowed_defaults: &[&str] = &[];

        let rules: Vec<KeyRule> = all_keys.iter().map(|(k, label)| {
            KeyRule {
                key: *k,
                label: label.to_string(),
                allowed: allowed_defaults.contains(label),
                modifiers: None,
            }
        }).collect();

        Config {
            rules,
            app_rules: Vec::new(),
            auto_unlock_timeout: Some(300),
            unlock_combo: vec![0xA2, 0xA4, 0x4C],
            version: 2,
        }
    }
}

impl KeyRule {
    pub fn matches(&self, key_code: u32, modifiers: &std::collections::HashSet<u32>) -> bool {
        if self.key != key_code {
            return false;
        }
        if let Some(ref required_mods) = self.modifiers {
            for m in required_mods {
                if !modifiers.contains(m) {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockEvent {
    Locked,
    Unlocked,
    KeyBlocked { key: u32 },
    KeyAllowed { key: u32 },
    ComboProgress { matched: usize, total: usize },
}