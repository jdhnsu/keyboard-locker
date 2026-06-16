use std::collections::HashSet;

use super::rules::{Config, KeyRule};

pub fn evaluate(
    config: &Config,
    active_app: Option<&str>,
    key_code: u32,
    modifiers: &HashSet<u32>,
) -> bool {
    if let Some(app_name) = active_app {
        if let Some(app_rule) = find_app_rule(&config.app_rules, app_name) {
            return whitelist_match(&app_rule.rules, key_code, modifiers);
        }
    }

    whitelist_match(&config.rules, key_code, modifiers)
}

fn find_app_rule<'a>(
    app_rules: &'a [super::rules::AppRule],
    process_name: &str,
) -> Option<&'a super::rules::AppRule> {
    let lower = process_name.to_lowercase();
    app_rules
        .iter()
        .find(|ar| ar.process_names.iter().any(|n| n.to_lowercase() == lower))
}

fn whitelist_match(rules: &[KeyRule], key_code: u32, modifiers: &HashSet<u32>) -> bool {
    rules
        .iter()
        .any(|r| r.allowed && r.matches(key_code, modifiers))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locker::rules::{AppRule, Config, KeyRule};

    fn test_config() -> Config {
        let mut c = Config::default();
        c.rules = vec![
            KeyRule {
                key: 0x41,
                label: "A".into(),
                allowed: true,
                modifiers: None,
            },
            KeyRule {
                key: 0x20,
                label: "Space".into(),
                allowed: true,
                modifiers: None,
            },
            KeyRule {
                key: 0x53,
                label: "S".into(),
                allowed: false,
                modifiers: None,
            },
        ];
        c
    }

    #[test]
    fn test_whitelist_allows_listed() {
        let config = test_config();
        let mods = HashSet::new();
        assert!(evaluate(&config, None, 0x41, &mods));
        assert!(evaluate(&config, None, 0x20, &mods));
    }

    #[test]
    fn test_whitelist_blocks_unlisted() {
        let config = test_config();
        let mods = HashSet::new();
        assert!(!evaluate(&config, None, 0x44, &mods));
        assert!(!evaluate(&config, None, 0x5A, &mods));
    }

    #[test]
    fn test_whitelist_blocks_disallowed() {
        let config = test_config();
        let mods = HashSet::new();
        assert!(!evaluate(&config, None, 0x53, &mods));
    }

    #[test]
    fn test_app_rule_priority() {
        let mut config = Config::default();
        config.app_rules = vec![AppRule {
            process_names: vec!["notepad.exe".into()],
            rules: vec![KeyRule {
                key: 0x41,
                label: "A".into(),
                allowed: true,
                modifiers: None,
            }],
        }];
        let mods = HashSet::new();
        assert!(evaluate(&config, Some("notepad.exe"), 0x41, &mods));
        assert!(!evaluate(&config, Some("notepad.exe"), 0x53, &mods));
    }
}
