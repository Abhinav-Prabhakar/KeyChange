//! KeyChange's typing engine.
//!
//! Everything here is pure state: no windows, no MIDI ports, no clocks. Callers
//! feed it notes, controls and timestamps; it answers with text, gestures and
//! visual state. That keeps the instrument deterministic and testable.

use std::collections::{HashSet, VecDeque};
use std::sync::OnceLock;

// ────────────────────────────── physical layout ──────────────────────────────

/// The 15 white keys drawn on screen, C3 → C5.
pub const WHITE_MIDI: [u8; 15] = [
    48, 50, 52, 53, 55, 57, 59, 60, 62, 64, 65, 67, 69, 71, 72,
];
/// White-key index each black key is engraved after.
pub const BLACK_SLOTS: [usize; 10] = [0, 1, 3, 4, 5, 7, 8, 10, 11, 12];
/// The 10 black keys, C#3 → A#4.
pub const BLACK_MIDI: [u8; 10] = [49, 51, 54, 56, 58, 61, 63, 66, 68, 70];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyKind {
    White(usize),
    Black(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Abc,
    Upper,
    Symbols,
}

impl Layer {
    pub fn next(self, delta: i32) -> Self {
        let i = match self {
            Layer::Abc => 0,
            Layer::Upper => 1,
            Layer::Symbols => 2,
        };
        match (i + delta).rem_euclid(3) {
            0 => Layer::Abc,
            1 => Layer::Upper,
            _ => Layer::Symbols,
        }
    }
}

pub const NUM_ROW: [&str; 15] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "!", "?", ".", ",", "↵",
];
pub const BLACK_GLYPHS: [&str; 10] = [",", ".", ";", "'", ":", "!", "?", "—", "_", "…"];

pub const WORDS: [&str; 6] = ["the", "and", "you", "that", "have", "with"];
/// Display root for the six diatonic word chords (C, D, E, F, G, A).
pub const WORD_ROOTS: [&str; 6] = ["C", "D", "E", "F", "G", "A"];
/// The six triad shapes as pitch classes, in [`WORDS`] order.
pub const WORD_TRIADS: [[i32; 3]; 6] = [
    [0, 4, 7],  // C  the
    [2, 5, 9],  // Dm and
    [4, 7, 11], // Em you
    [5, 9, 0],  // F  that
    [7, 11, 2], // G  have
    [9, 0, 4],  // Am with
];

/// Two-note intervals, by semitone distance, in the order we teach them.
pub const INTERVALS: [(i32, &str); 7] = [
    (1, "th"),
    (2, "he"),
    (3, "in"),
    (4, "er"),
    (5, "an"),
    (7, "ing"),
    (9, "ed"),
];

/// Chords shown on the pad bank, computed from the live layout so the pad
/// diagrams always match the keys you actually press.
pub struct ChordPad {
    pub word: &'static str,
    pub root: &'static str,
    pub keys: [usize; 3],
    pub notes: [u8; 3],
}

pub fn chord_pads() -> &'static [ChordPad] {
    static PADS: OnceLock<Vec<ChordPad>> = OnceLock::new();
    PADS.get_or_init(|| {
        let mut out = Vec::new();
        for i in 0..WORDS.len() {
            let word = WORDS[i];
            let root = WORD_ROOTS[i];
            let triad = WORD_TRIADS[i];
            let mut keys = [0usize; 3];
            let mut notes = [0u8; 3];
            // Pick the octave in the drawn range; the root is always a white key.
            let root_pc = triad[0];
            let base = WHITE_MIDI
                .iter()
                .position(|n| (*n as i32 % 12) == root_pc)
                .unwrap_or(0);
            for (i, pc) in triad.iter().enumerate() {
                let note = WHITE_MIDI
                    .iter()
                    .copied()
                    .skip(base)
                    .find(|n| (*n as i32 % 12) == *pc)
                    .unwrap_or(WHITE_MIDI[base]);
                notes[i] = note;
                keys[i] = WHITE_MIDI
                    .iter()
                    .position(|n| *n == note)
                    .unwrap_or(base);
            }
            out.push(ChordPad {
                word,
                root,
                keys,
                notes,
            });
        }
        out
    })
}

pub fn note_name(note: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (note as i32 / 12) - 1;
    format!("{}{}", NAMES[(note % 12) as usize], octave)
}

pub fn white_step(note: u8) -> Option<usize> {
    const PCS: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
    PCS.iter().position(|pc| *pc == (note as i32 % 12))
}

pub fn black_offset(note: u8) -> Option<usize> {
    const PCS: [i32; 5] = [1, 3, 6, 8, 10];
    PCS.iter().position(|pc| *pc == (note as i32 % 12))
}

/// Where a note lands on the drawn two-octave keyboard.
pub fn key_for_note(note: u8) -> Option<KeyKind> {
    if let Some(step) = white_step(note) {
        if note == 72 {
            return Some(KeyKind::White(14));
        }
        let oct = if note > 72 {
            1
        } else {
            ((note as i32 - 48).div_euclid(12)).clamp(0, 1) as usize
        };
        return Some(KeyKind::White((oct * 7 + step).min(14)));
    }
    let off = black_offset(note)?;
    let oct = if note > 72 {
        1
    } else {
        ((note as i32 - 48).div_euclid(12)).clamp(0, 1) as usize
    };
    Some(KeyKind::Black((oct * 5 + off).min(9)))
}

// ──────────────────────────────── profiles ──────────────────────────────────

pub const DEFAULT_LETTERS: [&str; 15] = [
    "e", "t", "a", "o", "i", "n", "s", "h", "r", "d", "l", "c", "u", "m", "w",
];

#[derive(Clone, Debug)]
pub struct LayoutProfile {
    pub name: String,
    pub letters: Vec<String>,
    pub custom: bool,
}

pub fn builtin_profiles() -> Vec<LayoutProfile> {
    let profile = |name: &str, letters: &[&str]| LayoutProfile {
        name: name.to_string(),
        letters: letters.iter().map(|s| s.to_string()).collect(),
        custom: false,
    };
    vec![
        profile("Default", &DEFAULT_LETTERS),
        profile(
            "Chromatic",
            &[
                "c", "d", "e", "f", "g", "a", "b", "c", "d", "e", "f", "g", "a", "b", "c",
            ],
        ),
        profile(
            "Jazz",
            &[
                "e", "a", "t", "7", "9", "o", "i", "#", "b", "n", "s", "h", "r", "d", "l",
            ],
        ),
        profile(
            "Coder",
            &[
                "a", "s", "d", "f", "g", "h", "j", "k", "l", ";", "{", "}", "(", ")", "↵",
            ],
        ),
    ]
}

/// English letter frequency, percent. Drives heat strips and map scores.
pub fn freq_for(glyph: &str) -> f32 {
    let g = glyph.to_lowercase();
    match g.as_str() {
        "e" => 12.70,
        "t" => 9.06,
        "a" => 8.17,
        "o" => 7.51,
        "i" => 6.97,
        "n" => 6.75,
        "s" => 6.33,
        "h" => 6.09,
        "r" => 5.99,
        "d" => 4.25,
        "l" => 4.03,
        "c" => 2.78,
        "u" => 2.76,
        "m" => 2.41,
        "w" => 2.36,
        "f" => 2.23,
        "g" => 2.02,
        "y" => 1.97,
        "p" => 1.93,
        "b" => 1.49,
        "v" => 0.98,
        "k" => 0.77,
        "j" => 0.15,
        "x" => 0.15,
        "q" => 0.095,
        "z" => 0.07,
        _ => 1.20,
    }
}

/// Physical effort of a white key: centre keys are cheapest.
pub fn effort_for_key(i: usize) -> f32 {
    match i {
        0..=8 => 1.0,
        9..=11 => 1.6,
        _ => 2.4,
    }
}

pub fn profile_score(letters: &[String]) -> f32 {
    letters
        .iter()
        .enumerate()
        .map(|(i, g)| freq_for(g) * effort_for_key(i))
        .sum()
}

/// 0–100, against the easiest possible assignment.
pub fn profile_quality(letters: &[String]) -> f32 {
    let mut freqs: Vec<f32> = letters.iter().map(|g| freq_for(g)).collect();
    freqs.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let ideal: f32 = freqs.iter().enumerate().map(|(i, f)| f * effort_for_key(i)).sum();
    let actual = profile_score(letters);
    if actual <= 0.0 {
        0.0
    } else {
        (ideal / actual * 100.0).clamp(0.0, 100.0)
    }
}

// ──────────────────────────────── text state ────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharKind {
    Letter,
    Symbol,
    Space,
    Word,
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub ch: char,
    pub kind: CharKind,
    pub vel: u8,
    pub corrected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gesture {
    Single,
    Interval(&'static str),
    Word(&'static str),
    Repeat,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Correction {
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug)]
pub struct NoteOutcome {
    pub key: Option<KeyKind>,
    pub gesture: Gesture,
    pub correction: Option<Correction>,
    pub resolved: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RecentNote {
    pub note: u8,
    pub vel: u8,
    pub index: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pedal {
    Sustain,
    Soft,
    Sostenuto,
}

#[derive(Clone, Debug)]
struct SessionNote {
    note: u8,
    step: Option<usize>,
    glyph: String,
}

#[derive(Clone, Debug)]
struct Session {
    start: usize,
    notes: Vec<SessionNote>,
    last_at: u64,
}

const TYPO_MAP: [(&str, &str); 8] = [
    ("teh", "the"),
    ("adn", "and"),
    ("hte", "the"),
    ("waht", "what"),
    ("taht", "that"),
    ("jsut", "just"),
    ("recieve", "receive"),
    ("seperate", "separate"),
];

const DICTIONARY: &[&str] = &[
    "the", "and", "you", "that", "have", "with", "this", "from", "they", "will",
    "would", "there", "their", "what", "about", "which", "when", "make", "like",
    "time", "just", "know", "take", "people", "into", "year", "your", "good",
    "some", "could", "them", "than", "then", "look", "only", "come", "over",
    "think", "also", "back", "after", "use", "two", "how", "our", "work",
    "first", "well", "way", "even", "new", "want", "because", "any", "these",
    "give", "day", "most", "us", "hello", "help", "held", "helm", "hear",
    "heart", "heavy", "here", "house", "sound", "song", "play", "playing",
    "words", "word", "loud", "louder", "music", "piano", "keys", "keyboard",
    "type", "typing", "instrument", "note", "notes", "chord", "chords",
    "space", "spaces", "sustain", "soft", "layer", "layers", "velocity",
    "quantize", "staff", "preset", "presets", "encore", "tempo", "timing",
    "press", "pressing", "hold", "holding", "pull", "push", "pedal", "pedals",
    "left", "right", "hand", "hands", "listen", "listening", "little",
    "letter", "letters", "light", "lights", "language", "loudest", "flow",
    "flowing", "glide", "gliding", "spell", "spelling", "speak", "speaking",
];

// ───────────────────────────────── engine ───────────────────────────────────

pub struct Engine {
    pub cells: Vec<Cell>,
    pub layer: Layer,
    pub base_velocity: u8,
    pub sustain: bool,
    pub soft: bool,
    pub sostenuto: bool,
    pub timing_ms: u64,
    pub profiles: Vec<LayoutProfile>,
    pub profile_index: usize,
    pub edit_mode: bool,
    pub swap_pick: Option<usize>,
    pub held: HashSet<u8>,
    pub recent: VecDeque<RecentNote>,
    pub node_seq: u64,
    pub notes_played: u32,
    pub chords_played: u32,
    pub corrections: u32,
    pub candidate_index: usize,
    pub last_token: Option<String>,
    pub last_correction: Option<(Correction, u64)>,
    pub wpm_history: VecDeque<u32>,
    pub bend: f32,
    pub last_bend_side: i8,
    session: Option<Session>,
    char_log: VecDeque<(u64, u32)>,
    undo_stack: Vec<Vec<Cell>>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            layer: Layer::Abc,
            base_velocity: 90,
            sustain: false,
            soft: false,
            sostenuto: false,
            timing_ms: 120,
            profiles: builtin_profiles(),
            profile_index: 0,
            edit_mode: false,
            swap_pick: None,
            held: HashSet::new(),
            recent: VecDeque::new(),
            node_seq: 0,
            notes_played: 0,
            chords_played: 0,
            corrections: 0,
            candidate_index: 0,
            last_token: None,
            last_correction: None,
            wpm_history: VecDeque::new(),
            bend: 0.0,
            last_bend_side: 0,
            session: None,
            char_log: VecDeque::new(),
            undo_stack: Vec::new(),
        }
    }

    pub fn text(&self) -> String {
        self.cells.iter().map(|c| c.ch).collect()
    }

    pub fn active_profile(&self) -> &LayoutProfile {
        &self.profiles[self.profile_index]
    }

    pub fn preview_white(&self, i: usize) -> String {
        let raw = &self.active_profile().letters[i.min(14)];
        match self.layer {
            Layer::Abc | Layer::Upper => raw.clone(),
            Layer::Symbols => NUM_ROW[i.min(14)].to_string(),
        }
    }

    pub fn preview_black(&self, b: usize) -> String {
        BLACK_GLYPHS[b.min(9)].to_string()
    }

    fn typed_white(&self, i: usize, velocity: u8) -> String {
        let raw = self.active_profile().letters[i.min(14)].clone();
        match self.layer {
            Layer::Abc => {
                if velocity >= 105 {
                    raw.to_uppercase()
                } else {
                    raw
                }
            }
            Layer::Upper => raw.to_uppercase(),
            Layer::Symbols => NUM_ROW[i.min(14)].to_string(),
        }
    }

    pub fn white_heat(&self, i: usize) -> f32 {
        (freq_for(&self.active_profile().letters[i.min(14)]) / 12.7).clamp(0.06, 1.0)
    }

    pub fn white_effort(&self, i: usize) -> f32 {
        effort_for_key(i)
    }

    pub fn quality(&self) -> f32 {
        profile_quality(&self.active_profile().letters)
    }


    // ── notes ────────────────────────────────────────────────────────────────

    pub fn note_on(&mut self, note: u8, velocity: u8, now: u64) -> NoteOutcome {
        let velocity = velocity.clamp(1, 127);
        let key = key_for_note(note);
        self.notes_played = self.notes_played.saturating_add(1);
        self.node_seq = self.node_seq.wrapping_add(1);
        self.recent.push_back(RecentNote {
            note,
            vel: velocity,
            index: self.node_seq,
        });
        while self.recent.len() > 28 {
            self.recent.pop_front();
        }

        // A session survives while a note is held or while we are inside the
        // rolled-chord window. Otherwise the next note starts fresh.
        let stale = match &self.session {
            None => true,
            Some(s) => now.saturating_sub(s.last_at) > self.timing_ms,
        };
        if stale {
            self.session = None;
        }
        if let Some(s) = &self.session {
            if s.notes.len() >= 8 {
                self.session = None;
            }
        }
        self.held.insert(note);

        if self.session.is_none() {
            self.snapshot();
            let ends_space = self
                .cells
                .last()
                .map(|c| c.ch.is_whitespace())
                .unwrap_or(true);
            if self.sustain && !self.cells.is_empty() && !ends_space {
                self.push_raw(' ', CharKind::Space, 70);
                self.quantize_last_word(now);
            }
            self.session = Some(Session {
                start: self.cells.len(),
                notes: Vec::new(),
                last_at: now,
            });
        }

        let (glyph, kind, step) = match key {
            Some(KeyKind::White(i)) => (
                self.typed_white(i, velocity),
                CharKind::Letter,
                Some(i),
            ),
            Some(KeyKind::Black(b)) => (
                self.preview_black(b),
                CharKind::Symbol,
                None,
            ),
            None => (String::new(), CharKind::Letter, None),
        };
        if !glyph.is_empty() {
            for ch in glyph.chars() {
                self.push_raw(ch, kind, velocity);
            }
            self.record_chars(glyph.chars().count() as u32, now);
        }

        let session_note = SessionNote { note, step, glyph };
        if let Some(s) = &mut self.session {
            s.notes.push(session_note);
            s.last_at = now;
        }

        let mut out = NoteOutcome {
            key,
            gesture: Gesture::Single,
            correction: None,
            resolved: None,
        };

        // Always rebuild from the session's notes so a failed chord attempt
        // restores the letters it temporarily replaced.
        let (text, gesture, finalize) = self.session_output().unwrap_or_else(|| {
            let glyphs = self
                .session
                .as_ref()
                .map(|s| s.notes.iter().map(|n| n.glyph.as_str()).collect::<String>())
                .unwrap_or_default();
            (glyphs, Gesture::Single, false)
        });
        if !text.is_empty() || self.session.as_ref().map(|s| !s.notes.is_empty()).unwrap_or(false) {
            let start = self.session.as_ref().map(|s| s.start).unwrap_or(0);
            let current: String = self
                .cells
                .iter()
                .skip(start)
                .map(|c| c.ch)
                .collect();
            if current != text {
                self.cells.truncate(start);
                let kind = if matches!(gesture, Gesture::Word(_) | Gesture::Repeat) {
                    CharKind::Word
                } else {
                    CharKind::Letter
                };
                for ch in text.chars() {
                    self.push_raw(ch, kind, velocity);
                }
                self.record_chars(text.chars().count() as u32, now);
            }
            out.gesture = gesture;
            out.resolved = Some(text.clone());
            if matches!(gesture, Gesture::Word(_) | Gesture::Repeat) {
                if !text.trim().is_empty() {
                    self.last_token = Some(text.trim().to_string());
                }
                self.chords_played = self.chords_played.saturating_add(1);
            }
            if text.ends_with(char::is_whitespace) {
                out.correction = self.quantize_last_word(now);
            }
            if finalize {
                self.session = None;
            }
        }

        self.sample_wpm(now);
        out
    }

    pub fn note_off(&mut self, note: u8) {
        self.held.remove(&note);
    }

    pub fn all_notes_off(&mut self) {
        self.held.clear();
    }

    /// Build the ideal output for the current session. `None` means "just the
    /// letters", `finalize` means the gesture is complete and the next note
    /// starts a new one.
    fn session_output(&self) -> Option<(String, Gesture, bool)> {
        let s = self.session.as_ref()?;
        if s.notes.is_empty() {
            return None;
        }
        let all_white = s.notes.iter().all(|n| n.step.is_some());
        if all_white {
            if s.notes.len() == 3 {
                let mut pcs: Vec<i32> = s
                    .notes
                    .iter()
                    .filter_map(|n| n.step.map(|_| (n.note as i32) % 12))
                    .collect();
                pcs.sort_unstable();
                pcs.dedup();
                if pcs.len() == 3 {
                    for (triad, word) in WORD_TRIADS.iter().zip(WORDS) {
                        let mut t = triad.to_vec();
                        t.sort_unstable();
                        if t == pcs {
                            return Some((format!("{word} "), Gesture::Word(word), true));
                        }
                    }
                }
            } else if s.notes.len() == 2 {
                let a = s.notes[0].note as i32;
                let b = s.notes[1].note as i32;
                let (lo, hi) = if a < b { (a, b) } else { (b, a) };
                if hi > lo && (hi - lo) % 12 == 0 {
                    let token = self.last_token.clone()?;
                    let gap = if s.start == 0
                        || self
                            .cells
                            .get(s.start.saturating_sub(1))
                            .map(|c| c.ch.is_whitespace())
                            .unwrap_or(true)
                    {
                        ""
                    } else {
                        " "
                    };
                    return Some((format!("{gap}{token} "), Gesture::Repeat, true));
                }
                let d = (hi - lo).rem_euclid(12);
                if let Some((_, bigram)) = INTERVALS.iter().find(|(semi, _)| *semi == d) {
                    return Some((bigram.to_string(), Gesture::Interval(bigram), false));
                }
            }
        }
        None
    }

    // ── pedals & controls ────────────────────────────────────────────────────

    pub fn control(&mut self, cc: u8, value: u8, now: u64) -> Option<NoteOutcome> {
        match cc {
            1 => {
                self.set_wheel(value);
                None
            }
            11 => {
                self.timing_ms = 40 + ((value as u64) * 200 / 127);
                None
            }
            64 => {
                self.sustain = value >= 64;
                None
            }
            66 => {
                if value >= 64 && !self.sostenuto {
                    self.sostenito_press(now);
                }
                self.sostenuto = value >= 64;
                None
            }
            67 => {
                let was = self.soft;
                self.soft = value >= 64;
                if self.soft && !was {
                    self.cycle_layer(1);
                }
                None
            }
            123 => {
                self.all_notes_off();
                None
            }
            _ => None,
        }
    }

    pub fn aftertouch(&mut self, _pressure: u8, now: u64) -> bool {
        self.accept_candidate(now)
    }

    pub fn pitch_bend(&mut self, value: i16) -> bool {
        self.bend = (value as f32 / 8192.0).clamp(-1.0, 1.0);
        let side = if self.bend > 0.35 {
            1
        } else if self.bend < -0.35 {
            -1
        } else {
            0
        };
        let fire = side != 0 && side != self.last_bend_side;
        self.last_bend_side = side;
        if fire {
            self.scrub_candidates(side as i32);
        }
        fire
    }

    pub fn toggle_pedal(&mut self, pedal: Pedal, now: u64) {
        match pedal {
            Pedal::Sustain => self.sustain = !self.sustain,
            Pedal::Soft => {
                self.soft = true;
                self.cycle_layer(1);
            }
            Pedal::Sostenuto => self.sostenito_press(now),
        }
    }

    fn sostenito_press(&mut self, _now: u64) {
        self.sostenuto = true;
        self.delete_word();
    }

    pub fn cycle_layer(&mut self, delta: i32) {
        self.layer = self.layer.next(delta);
        self.session = None;
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.layer = layer;
        self.session = None;
    }

    pub fn set_base_velocity(&mut self, velocity: u8) {
        self.base_velocity = velocity.clamp(20, 127);
    }

    pub fn set_timing(&mut self, ms: u64) {
        self.timing_ms = ms.clamp(40, 240);
    }

    // ── text editing ─────────────────────────────────────────────────────────

    fn snapshot(&mut self) {
        self.undo_stack.push(self.cells.clone());
        if self.undo_stack.len() > 60 {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) -> bool {
        self.session = None;
        if let Some(prev) = self.undo_stack.pop() {
            self.cells = prev;
            true
        } else {
            false
        }
    }

    fn push_raw(&mut self, ch: char, kind: CharKind, vel: u8) {
        self.cells.push(Cell {
            ch,
            kind,
            vel,
            corrected: false,
        });
        if self.cells.len() > 420 {
            let drop = self.cells.len() - 420;
            self.cells.drain(0..drop);
        }
    }

    fn insert_text(&mut self, text: &str, kind: CharKind, vel: u8, now: u64) {
        self.snapshot();
        self.session = None;
        let ends_space = self
            .cells
            .last()
            .map(|c| c.ch.is_whitespace())
            .unwrap_or(true);
        if self.sustain && !self.cells.is_empty() && !ends_space {
            self.push_raw(' ', CharKind::Space, 70);
            self.quantize_last_word(now);
        }
        for ch in text.chars() {
            self.push_raw(ch, kind, vel);
        }
        self.record_chars(text.chars().count() as u32, now);
        if !text.trim().is_empty() {
            self.last_token = Some(text.trim().to_string());
        }
    }

    pub fn type_space(&mut self, now: u64) {
        self.insert_text(" ", CharKind::Space, 70, now);
        self.quantize_last_word(now);
        self.recompute_last_token();
        self.sample_wpm(now);
    }

    /// Finish the current word (Enter): quantize it, then move on.
    pub fn commit_word(&mut self, now: u64) {
        self.insert_text(" ", CharKind::Space, 70, now);
        self.quantize_last_word(now);
        self.recompute_last_token();
        self.sample_wpm(now);
    }

    pub fn backspace(&mut self) -> bool {
        self.session = None;
        self.snapshot();
        if self.cells.pop().is_some() {
            self.recompute_last_token();
            true
        } else {
            false
        }
    }

    pub fn delete_word(&mut self) {
        self.session = None;
        self.snapshot();
        while self.cells.last().map(|c| c.ch.is_whitespace()).unwrap_or(false) {
            self.cells.pop();
        }
        while self.cells.last().map(|c| !c.ch.is_whitespace()).unwrap_or(false) {
            self.cells.pop();
        }
        while self.cells.last().map(|c| c.ch.is_whitespace()).unwrap_or(false) {
            self.cells.pop();
        }
        self.recompute_last_token();
    }

    pub fn clear(&mut self) {
        self.session = None;
        self.snapshot();
        self.cells.clear();
        self.last_token = None;
        self.char_log.clear();
        self.wpm_history.clear();
    }

    fn recompute_last_token(&mut self) {
        let text = self.text();
        self.last_token = text
            .split_whitespace()
            .last()
            .map(|s| s.to_string());
    }

    // ── quantize ─────────────────────────────────────────────────────────────

    fn quantize_last_word(&mut self, now: u64) -> Option<Correction> {
        // Find the trailing word: skip trailing whitespace, then collect letters.
        let mut end = self.cells.len();
        while end > 0 && self.cells[end - 1].ch.is_whitespace() {
            end -= 1;
        }
        let mut start = end;
        while start > 0 && !self.cells[start - 1].ch.is_whitespace() {
            start -= 1;
        }
        if start == end {
            return None;
        }
        let word: String = self.cells[start..end].iter().map(|c| c.ch).collect();
        let lower = word.to_lowercase();
        let (_, fixed) = TYPO_MAP.iter().find(|(typo, _)| *typo == lower)?;
        if fixed.chars().count() != word.chars().count() {
            return None;
        }
        for (i, ch) in fixed.chars().enumerate() {
            if let Some(cell) = self.cells.get_mut(start + i) {
                cell.ch = ch;
                cell.corrected = true;
            }
        }
        self.corrections = self.corrections.saturating_add(1);
        let correction = Correction {
            from: word.clone(),
            to: fixed.to_string(),
        };
        self.last_correction = Some((correction.clone(), now));
        if self.last_token.as_deref() == Some(lower.as_str()) {
            self.last_token = Some(fixed.to_string());
        }
        Some(correction)
    }

    // ── autocomplete ─────────────────────────────────────────────────────────

    pub fn trailing_prefix(&self) -> String {
        let text = self.text();
        let tail = text
            .rsplit(|c: char| !c.is_alphabetic() && c != '\'')
            .next()
            .unwrap_or("");
        tail.to_lowercase()
    }

    pub fn candidates(&self) -> Vec<&'static str> {
        let prefix = self.trailing_prefix();
        let mut out: Vec<&'static str> = Vec::new();
        if !prefix.is_empty() && prefix.len() <= 14 {
            for word in DICTIONARY {
                if word.starts_with(&prefix) && *word != prefix {
                    out.push(word);
                }
                if out.len() >= 4 {
                    break;
                }
            }
            if out.is_empty() {
                if let Some(exact) = DICTIONARY.iter().find(|w| **w == prefix) {
                    out.push(exact);
                }
            }
        }
        out
    }

    pub fn candidate_index(&self) -> usize {
        let n = self.candidates().len();
        if n == 0 || self.candidate_index >= n {
            0
        } else {
            self.candidate_index
        }
    }

    pub fn scrub_candidates(&mut self, delta: i32) {
        let candidates = self.candidates();
        if candidates.is_empty() {
            self.candidate_index = 0;
            return;
        }
        let n = candidates.len() as i32;
        self.candidate_index = (self.candidate_index as i32 + delta).rem_euclid(n) as usize;
    }

    pub fn set_wheel(&mut self, value: u8) {
        let n = self.candidates().len();
        if n == 0 {
            self.candidate_index = 0;
        } else {
            self.candidate_index = ((value as f32 / 127.0) * (n as f32 - 1.0))
                .round()
                .clamp(0.0, (n - 1) as f32) as usize;
        }
    }

    pub fn accept_candidate(&mut self, now: u64) -> bool {
        let candidates = self.candidates();
        let index = self.candidate_index.min(candidates.len().saturating_sub(1));
        let Some(word) = candidates.get(index).copied() else {
            return false;
        };
        let prefix = self.trailing_prefix();
        if !word.starts_with(&prefix) {
            return false;
        }
        let rest = &word[prefix.len()..];
        if rest.is_empty() {
            self.type_space(now);
            return true;
        }
        self.insert_text(rest, CharKind::Letter, self.base_velocity, now);
        self.insert_text(" ", CharKind::Space, 70, now);
        self.last_token = Some(word.to_string());
        self.quantize_last_word(now);
        self.candidate_index = 0;
        self.sample_wpm(now);
        true
    }

    // ── map editing ──────────────────────────────────────────────────────────

    pub fn set_profile(&mut self, index: usize) {
        if index < self.profiles.len() {
            self.profile_index = index;
            self.edit_mode = false;
            self.swap_pick = None;
        }
    }

    pub fn duplicate_as_custom(&mut self) {
        let src = self.active_profile().letters.clone();
        let n = self.profiles.iter().filter(|p| p.custom).count() + 1;
        self.profiles.push(LayoutProfile {
            name: format!("Custom {n}"),
            letters: src,
            custom: true,
        });
        self.profile_index = self.profiles.len() - 1;
        self.edit_mode = true;
        self.swap_pick = None;
    }

    pub fn begin_edit(&mut self) {
        if !self.active_profile().custom {
            self.duplicate_as_custom();
        } else {
            self.edit_mode = true;
        }
        self.swap_pick = None;
    }

    pub fn cancel_edit(&mut self) {
        self.edit_mode = false;
        self.swap_pick = None;
    }

    /// In remap mode, pick two keys (or heat-strip columns) to swap.
    pub fn pick_swap(&mut self, index: usize) -> bool {
        match self.swap_pick {
            None => {
                self.swap_pick = Some(index);
                false
            }
            Some(prev) if prev == index => {
                self.swap_pick = None;
                false
            }
            Some(prev) => {
                if self.active_profile().custom {
                    self.profiles[self.profile_index].letters.swap(prev, index);
                }
                self.swap_pick = None;
                true
            }
        }
    }

    // ── stats ────────────────────────────────────────────────────────────────

    fn record_chars(&mut self, count: u32, now: u64) {
        if count == 0 {
            return;
        }
        self.char_log.push_back((now, count));
        let cutoff = now.saturating_sub(120_000);
        while self.char_log.front().map(|(t, _)| *t < cutoff).unwrap_or(false) {
            self.char_log.pop_front();
        }
    }

    fn sample_wpm(&mut self, now: u64) {
        let wpm = self.wpm(now);
        self.wpm_history.push_back(wpm);
        while self.wpm_history.len() > 36 {
            self.wpm_history.pop_front();
        }
    }

    pub fn wpm(&self, now: u64) -> u32 {
        let cutoff = now.saturating_sub(60_000);
        let chars: u32 = self
            .char_log
            .iter()
            .filter(|(t, _)| *t >= cutoff)
            .map(|(_, n)| *n)
            .sum();
        if chars == 0 {
            return 0;
        }
        let first = self
            .char_log
            .iter()
            .find(|(t, _)| *t >= cutoff)
            .map(|(t, _)| *t)
            .unwrap_or(now);
        let elapsed_ms = now.saturating_sub(first).max(2_500).min(60_000);
        let minutes = elapsed_ms as f32 / 60_000.0;
        ((chars as f32 / 5.0) / minutes)
            .round()
            .clamp(0.0, 240.0) as u32
    }

    pub fn held_white(&self) -> [bool; 15] {
        let mut out = [false; 15];
        for note in &self.held {
            if let Some(KeyKind::White(i)) = key_for_note(*note) {
                out[i] = true;
            }
        }
        out
    }

    pub fn held_black(&self) -> [bool; 10] {
        let mut out = [false; 10];
        for note in &self.held {
            if let Some(KeyKind::Black(i)) = key_for_note(*note) {
                out[i] = true;
            }
        }
        out
    }

    pub fn key_qwerty(&mut self, key: &str, now: u64) -> Option<NoteOutcome> {
        let note = qwerty_note(key)?;
        let velocity = if BLACK_MIDI.contains(&note) {
            105
        } else {
            self.base_velocity
        };
        Some(self.note_on(note, velocity, now))
    }

    /// Release the note a QWERTY key played, so held-key visuals stay honest.
    pub fn key_qwerty_release(&mut self, key: &str) {
        if let Some(note) = qwerty_note(key) {
            self.note_off(note);
        }
    }
}

/// The QWERTY fallback maps a computer key to one of the drawn piano keys.
pub fn qwerty_note(key: &str) -> Option<u8> {
    let white = [
        ("a", 0usize),
        ("s", 1),
        ("d", 2),
        ("f", 3),
        ("g", 4),
        ("h", 5),
        ("j", 6),
        ("k", 7),
        ("l", 8),
        (";", 9),
        ("'", 10),
        ("1", 11),
        ("2", 12),
        ("3", 13),
        ("4", 14),
    ];
    if let Some((_, i)) = white.iter().find(|(k, _)| *k == key) {
        return Some(WHITE_MIDI[*i]);
    }
    let black = [
        ("w", 0usize),
        ("e", 1),
        ("t", 2),
        ("y", 3),
        ("u", 4),
        ("o", 5),
        ("p", 6),
        ("[", 7),
        ("]", 8),
        ("\\", 9),
    ];
    black
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, b)| BLACK_MIDI[*b])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> Engine {
        Engine::new()
    }

    fn chord(e: &mut Engine, notes: &[u8], t: u64) {
        for n in notes {
            e.note_on(*n, 90, t);
        }
    }

    #[test]
    fn white_keys_type_the_default_letters() {
        let mut e = engine();
        e.note_on(WHITE_MIDI[0], 90, 0);
        e.note_on(WHITE_MIDI[1], 90, 500);
        e.note_on(WHITE_MIDI[2], 90, 1000);
        assert_eq!(e.text(), "eta");
    }

    #[test]
    fn velocity_above_105_is_uppercase() {
        let mut e = engine();
        e.note_on(WHITE_MIDI[0], 120, 0);
        assert_eq!(e.text(), "E");
    }

    #[test]
    fn upper_layer_forces_caps_including_soft_velocity() {
        let mut e = engine();
        e.set_layer(Layer::Upper);
        e.note_on(WHITE_MIDI[0], 40, 0);
        assert_eq!(e.text(), "E");
    }

    #[test]
    fn symbols_layer_types_numbers() {
        let mut e = engine();
        e.set_layer(Layer::Symbols);
        e.note_on(WHITE_MIDI[0], 90, 0);
        e.note_off(WHITE_MIDI[0]);
        e.note_on(WHITE_MIDI[1], 90, 500);
        assert_eq!(e.text(), "12");
    }

    #[test]
    fn black_keys_type_punctuation_and_are_blue_symbols() {
        let mut e = engine();
        e.note_on(BLACK_MIDI[0], 105, 0);
        assert_eq!(e.text(), ",");
        assert_eq!(e.cells[0].kind, CharKind::Symbol);
    }

    #[test]
    fn any_octave_maps_to_its_engraved_key() {
        let mut e = engine();
        e.note_on(60, 90, 0); // C4 is the eighth white key
        assert_eq!(e.text(), "h");
        e.note_on(48, 90, 500); // C3 is the first
        assert_eq!(e.text(), "he");
    }

    #[test]
    fn c_major_triad_types_the() {
        let mut e = engine();
        chord(&mut e, &[48, 52, 55], 0);
        assert_eq!(e.text(), "the ");
        assert_eq!(e.chords_played, 1);
    }

    #[test]
    fn d_minor_triad_types_and() {
        let mut e = engine();
        chord(&mut e, &[50, 53, 57], 10);
        assert_eq!(e.text(), "and ");
    }

    #[test]
    fn every_word_chord_matches_its_root_diap() {
        let mut e = engine();
        let expected = ["the ", "and ", "you ", "that ", "have ", "with "];
        for (i, triad) in WORD_TRIADS.iter().enumerate() {
            let notes: Vec<u8> = triad
                .iter()
                .map(|pc| {
                    (48..=72)
                        .find(|n| (*n as i32 % 12) == *pc && white_step(*n).is_some())
                        .unwrap()
                })
                .collect();
            chord(&mut e, &notes, i as u64 * 400);
            assert!(e.text().starts_with(expected[i]));
            e.clear();
        }
    }

    #[test]
    fn two_note_interval_types_bigram() {
        let mut e = engine();
        e.note_on(48, 90, 0);
        e.note_on(53, 90, 30); // C + F = perfect fourth → "an"
        assert_eq!(e.text(), "an");
    }

    #[test]
    fn interval_does_not_wait_for_note_off() {
        let mut e = engine();
        chord(&mut e, &[48, 52], 0);
        assert_eq!(e.text(), "er");
    }

    #[test]
    fn octave_interval_repeats_last_token() {
        let mut e = engine();
        chord(&mut e, &[48, 52, 55], 0);
        e.note_off(48);
        e.note_off(52);
        e.note_off(55);
        chord(&mut e, &[48, 60], 1000);
        assert_eq!(e.text(), "the the ");
        assert_eq!(e.chords_played, 2);
    }

    #[test]
    fn a_third_note_that_breaks_the_triad_restores_letters() {
        let mut e = engine();
        e.note_on(48, 90, 0); // e
        e.note_on(52, 90, 20); // e + a → "er"
        assert_eq!(e.text(), "er");
        e.note_on(59, 90, 40); // + s: no triad, back to letters
        assert_eq!(e.text(), "eas");
    }

    #[test]
    fn rolled_chord_within_window_still_resolves() {
        let mut e = engine();
        e.note_on(48, 90, 0);
        e.note_off(48);
        e.note_on(52, 90, 50);
        e.note_off(52);
        e.note_on(55, 90, 80);
        assert_eq!(e.text(), "the ");
    }

    #[test]
    fn notes_after_window_are_separate() {
        let mut e = engine();
        e.note_on(48, 90, 0);
        e.note_off(48);
        e.note_on(52, 90, 500);
        assert_eq!(e.text(), "ea");
    }

    #[test]
    fn sustain_inserts_word_gaps() {
        let mut e = engine();
        e.sustain = true;
        e.note_on(48, 90, 0);
        e.note_off(48);
        e.note_on(50, 90, 500);
        assert_eq!(e.text(), "e t");
    }

    #[test]
    fn soft_pedal_cycles_layers() {
        let mut e = engine();
        e.toggle_pedal(Pedal::Soft, 0);
        assert_eq!(e.layer, Layer::Upper);
        e.toggle_pedal(Pedal::Soft, 10);
        assert_eq!(e.layer, Layer::Symbols);
        e.toggle_pedal(Pedal::Soft, 20);
        assert_eq!(e.layer, Layer::Abc);
    }

    #[test]
    fn soft_cc_only_fires_on_transition() {
        let mut e = engine();
        e.control(67, 127, 0);
        assert_eq!(e.layer, Layer::Upper);
        e.control(67, 90, 10);
        assert_eq!(e.layer, Layer::Upper);
    }

    #[test]
    fn sostenuto_deletes_the_last_word() {
        let mut e = engine();
        chord(&mut e, &[48, 52, 55], 10);
        chord(&mut e, &[50, 53, 57], 600);
        assert_eq!(e.text(), "the and ");
        e.toggle_pedal(Pedal::Sostenuto, 1000);
        assert_eq!(e.text(), "the");
    }

    #[test]
    fn cc_sustain_toggles() {
        let mut e = engine();
        e.control(64, 127, 0);
        assert!(e.sustain);
        e.control(64, 0, 10);
        assert!(!e.sustain);
    }

    #[test]
    fn timing_is_set_by_expression_cc() {
        let mut e = engine();
        e.control(11, 0, 0);
        assert_eq!(e.timing_ms, 40);
        e.control(11, 127, 0);
        assert_eq!(e.timing_ms, 240);
    }

    #[test]
    fn quantize_fixes_teh_and_marks_cells() {
        let mut e = engine();
        e.note_on(50, 70, 100); // t
        e.note_on(48, 70, 600); // e
        e.note_on(60, 70, 1100); // h
        assert_eq!(e.text(), "teh");
        e.type_space(1600);
        assert_eq!(e.text(), "the ");
        assert!(e.cells[0].corrected);
        assert_eq!(
            e.last_correction.as_ref().map(|(c, _)| c.to.clone()),
            Some("the".to_string())
        );
    }

    #[test]
    fn backspace_and_undo() {
        let mut e = engine();
        chord(&mut e, &[48, 52, 55], 0);
        assert_eq!(e.text(), "the ");
        assert!(e.backspace());
        assert_eq!(e.text(), "the");
        assert!(e.undo());
        assert_eq!(e.text(), "the ");
    }

    #[test]
    fn candidates_follow_the_trailing_word() {
        let mut e = engine();
        e.note_on(60, 70, 10); // h
        e.note_on(48, 70, 500); // e
        e.note_on(65, 70, 900); // l
        assert_eq!(e.text(), "hel");
        let c = e.candidates();
        assert!(c.contains(&"hello"));
        assert!(c.contains(&"help"));
    }

    #[test]
    fn aftertouch_accepts_the_selected_candidate() {
        let mut e = engine();
        e.note_on(60, 70, 10);
        e.note_on(48, 70, 500);
        e.note_on(65, 70, 900);
        let first = e.candidates()[0];
        assert!(e.accept_candidate(1000));
        assert!(e.text().starts_with(first));
        assert!(e.text().ends_with(' '));
    }

    #[test]
    fn wheel_scrubs_candidates() {
        let mut e = engine();
        e.note_on(60, 70, 10);
        e.note_on(48, 70, 500);
        e.note_on(65, 70, 900);
        let n = e.candidates().len();
        assert!(n > 1);
        e.control(1, 127, 1000);
        assert_eq!(e.candidate_index(), n - 1);
        e.control(1, 0, 1100);
        assert_eq!(e.candidate_index(), 0);
    }

    #[test]
    fn wpm_uses_the_rolling_minute() {
        let mut e = engine();
        for i in 0..50 {
            e.note_on(48, 70, i * 100);
            e.note_off(48);
        }
        let wpm = e.wpm(5000);
        assert!(wpm > 0 && wpm <= 240, "wpm {wpm}");
    }

    #[test]
    fn remap_swaps_two_letters_in_a_custom_profile() {
        let mut e = engine();
        e.begin_edit();
        assert!(e.active_profile().custom);
        assert!(!e.pick_swap(0));
        assert!(e.pick_swap(1));
        assert_eq!(e.preview_white(0), "t");
        assert_eq!(e.preview_white(1), "e");
    }

    #[test]
    fn qwerty_map_plays_the_piano() {
        let mut e = engine();
        assert!(e.key_qwerty("a", 0).is_some());
        assert!(e.key_qwerty("w", 300).is_some());
        assert_eq!(e.text(), "e,");
    }

    #[test]
    fn key_for_note_covers_the_two_drawn_octaves() {
        assert_eq!(key_for_note(48), Some(KeyKind::White(0)));
        assert_eq!(key_for_note(60), Some(KeyKind::White(7)));
        assert_eq!(key_for_note(72), Some(KeyKind::White(14)));
        assert_eq!(key_for_note(49), Some(KeyKind::Black(0)));
        assert_eq!(key_for_note(70), Some(KeyKind::Black(9)));
    }

    #[test]
    fn chord_pads_match_the_layout() {
        for pad in chord_pads() {
            let mut e = engine();
            for note in pad.notes {
                e.note_on(note, 90, 0);
            }
            assert!(e.text().starts_with(pad.word), "{:?}", e.text());
        }
    }

    #[test]
    fn profile_quality_prefers_huffman_default() {
        let default = builtin_profiles()[0].letters.clone();
        let chromatic = builtin_profiles()[1].letters.clone();
        assert!(profile_quality(&default) > profile_quality(&chromatic));
        assert!((profile_quality(&default) - 100.0).abs() < 0.01);
    }

    #[test]
    fn score_and_quality_are_sane() {
        let e = engine();
        assert!(profile_score(&e.active_profile().letters) > 0.0);
        assert!(e.quality() > 95.0);
    }

    #[test]
    fn a_held_note_does_not_hold_the_session_open_forever() {
        let mut e = engine();
        e.note_on(48, 90, 0); // held, never released
        e.note_on(52, 90, 500); // far outside the 120 ms window
        assert_eq!(e.text(), "ea");
    }

    #[test]
    fn qwerty_release_clears_the_pressed_key() {
        let mut e = engine();
        e.key_qwerty("a", 0);
        assert!(e.held.contains(&WHITE_MIDI[0]));
        e.key_qwerty_release("a");
        assert!(e.held.is_empty());
    }

    #[test]
    fn octave_repeat_uses_the_completed_word_and_adds_one_gap() {
        let mut e = engine();
        chord(&mut e, &[48, 52, 55], 0);
        for n in [48, 52, 55] {
            e.note_off(n);
        }
        chord(&mut e, &[48, 60], 1_000);
        assert_eq!(e.text(), "the the ");
        assert_eq!(e.last_token.as_deref(), Some("the"));
    }

    #[test]
    fn space_commits_a_repeat_token() {
        let mut e = engine();
        e.note_on(48, 90, 0); // e
        e.note_on(50, 90, 500); // t
        e.note_on(60, 90, 900); // h
        e.type_space(1_300);
        assert_eq!(e.text(), "eth ");
        assert_eq!(e.last_token.as_deref(), Some("eth"));
        chord(&mut e, &[48, 60], 2_000);
        assert_eq!(e.text(), "eth eth ");
    }

    #[test]
    fn acceptance_keeps_the_full_completed_word() {
        let mut e = engine();
        e.note_on(60, 70, 0); // h
        e.note_on(48, 70, 400); // e
        e.note_on(65, 70, 800); // l
        let first = e.candidates()[0];
        assert!(e.accept_candidate(1_200));
        assert!(e.text().starts_with(first));
        assert_eq!(e.last_token.as_deref(), Some(first));
    }

    #[test]
    fn sustain_space_is_not_doubled() {
        let mut e = engine();
        e.sustain = true;
        e.type_space(0);
        e.note_on(48, 90, 10);
        assert_eq!(e.text(), " e");
    }
}
