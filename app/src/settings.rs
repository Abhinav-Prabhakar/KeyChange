//! Saved settings: layouts, velocity, gate, layer and learned CC numbers.
//!
//! The engine stays pure. This module is the only place that knows a file
//! exists, and every failure falls back to the built-in defaults rather than
//! stopping the instrument.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::engine::{builtin_profiles, CcMap, Engine, Layer, LayoutProfile};

pub const VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub version: u32,
    pub profiles: Vec<LayoutProfile>,
    pub profile_index: usize,
    pub base_velocity: u8,
    pub timing_ms: u64,
    pub layer: Layer,
    pub cc_map: CcMap,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: VERSION,
            profiles: builtin_profiles(),
            profile_index: 0,
            base_velocity: 90,
            timing_ms: 120,
            layer: Layer::Abc,
            cc_map: CcMap::default(),
        }
    }
}

impl Settings {
    pub fn from_engine(engine: &Engine) -> Self {
        Self {
            version: VERSION,
            profiles: engine.profiles.clone(),
            profile_index: engine.profile_index,
            base_velocity: engine.base_velocity,
            timing_ms: engine.timing_ms,
            layer: engine.layer,
            cc_map: engine.cc_map,
        }
    }

    /// Apply a loaded file without trusting it. Bad values are repaired rather
    /// than rejected so a half-edited file never bricks the instrument.
    pub fn apply_to(self, engine: &mut Engine) {
        if !self.profiles.is_empty() && self.profiles.iter().all(|p| p.letters.len() == 15) {
            engine.profiles = self.profiles;
            engine.profile_index = self.profile_index.min(engine.profiles.len() - 1);
        }
        if (1..=127).contains(&self.base_velocity) {
            engine.base_velocity = self.base_velocity;
        }
        if (40..=240).contains(&self.timing_ms) {
            engine.timing_ms = self.timing_ms;
        }
        engine.layer = self.layer;
        engine.cc_map = self.cc_map;
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    pub fn from_json(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| e.to_string())
    }

    pub fn exists() -> bool {
        path().exists()
    }

    pub fn load() -> Self {
        let path = path();
        let Ok(text) = fs::read_to_string(&path) else {
            return Self::default();
        };
        Self::from_json(&text).unwrap_or_else(|_| {
            // Keep the unreadable file around for a post-mortem instead of
            // silently deleting the user's layouts.
            let _ = fs::rename(&path, path.with_extension("json.bad"));
            Self::default()
        })
    }

    pub fn save(&self) -> Result<(), String> {
        let path = path();
        save_to(&path, self)
    }
}

pub fn save_to(path: &Path, settings: &Settings) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    }
    let json = settings.to_json()?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json).map_err(|e| format!("could not write settings: {e}"))?;
    fs::rename(&tmp, path).map_err(|e| format!("could not replace settings: {e}"))?;
    Ok(())
}

/// Where the app keeps its settings. `KC_SETTINGS_PATH` wins, which keeps
/// tests and portable installs out of the real config directory.
pub fn path() -> PathBuf {
    if let Ok(custom) = env::var("KC_SETTINGS_PATH") {
        if !custom.trim().is_empty() {
            return PathBuf::from(custom);
        }
    }
    if cfg!(target_os = "macos") {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home).join("Library/Application Support/KeyChange/settings.json");
        }
    }
    if let Ok(config) = env::var("XDG_CONFIG_HOME") {
        if !config.trim().is_empty() {
            return PathBuf::from(config).join("KeyChange/settings.json");
        }
    }
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join(".config/KeyChange/settings.json");
    }
    PathBuf::from("keychange-settings.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Control;

    #[test]
    fn settings_round_trip_through_json() {
        let mut engine = Engine::new();
        engine.set_base_velocity(112);
        engine.set_timing(180);
        engine.set_layer(Layer::Symbols);
        engine.duplicate_as_custom();
        engine.cc_map.assign(Control::Sustain, 80);

        let saved = Settings::from_engine(&engine);
        let text = saved.to_json().unwrap();
        let loaded = Settings::from_json(&text).unwrap();
        let mut fresh = Engine::new();
        loaded.apply_to(&mut fresh);

        assert_eq!(fresh.base_velocity, 112);
        assert_eq!(fresh.timing_ms, 180);
        assert_eq!(fresh.layer, Layer::Symbols);
        assert_eq!(fresh.profiles.len(), 5);
        assert_eq!(fresh.active_profile().name, "Custom 1");
        assert_eq!(fresh.cc_map.sustain, 80);
    }

    #[test]
    fn corrupt_or_partial_settings_are_repaired() {
        let text = r#"{
            "profiles": [{"name":"Broken","letters":["a"],"custom":true}],
            "profile_index": 9,
            "base_velocity": 0,
            "timing_ms": 9999
        }"#;
        let loaded = Settings::from_json(text).unwrap();
        let mut engine = Engine::new();
        loaded.apply_to(&mut engine);
        assert_eq!(engine.profiles.len(), 4);
        assert_eq!(engine.base_velocity, 90);
        assert_eq!(engine.timing_ms, 120);
        assert_eq!(engine.profile_index, 0);
    }

    #[test]
    fn unknown_json_fields_are_ignored() {
        let text = r#"{"version": 1, "from_the_future": true}"#;
        let loaded = Settings::from_json(text).unwrap();
        assert_eq!(loaded.version, 1);
        assert!(!loaded.profiles.is_empty());
    }

    #[test]
    fn saving_and_loading_a_file_works() {
        let dir = std::env::temp_dir().join(format!("kc-settings-{}", std::process::id()));
        let path = dir.join("settings.json");
        let mut engine = Engine::new();
        engine.set_base_velocity(101);
        save_to(&path, &Settings::from_engine(&engine)).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let loaded = Settings::from_json(&text).unwrap();
        assert_eq!(loaded.base_velocity, 101);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
