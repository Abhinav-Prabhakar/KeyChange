//! MIDI input and the simulated keyboard.
//!
//! Hardware and the demo player both emit raw MIDI bytes into one channel, so
//! the engine cannot tell them apart. That means the simulation exercises the
//! exact same parser and path as a real piano.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use midir::{MidiInput, MidiInputConnection};
use smol::channel::{Receiver, Sender};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Midi,
    Demo,
    DemoEnd,
}

#[derive(Clone, Debug)]
pub struct Packet {
    pub bytes: Vec<u8>,
    pub source: Source,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MidiStatus {
    NoDevice,
    Connected(String),
    Error(String),
}

impl MidiStatus {
    pub fn is_live(&self) -> bool {
        matches!(self, MidiStatus::Connected(_))
    }
}

/// Something the UI should mention without polling every frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiEvent {
    Connected,
    Disconnected,
    DevicesChanged,
}

pub struct MidiHub {
    pub status: MidiStatus,
    /// Port names currently plugged in, in system order.
    pub devices: Vec<String>,
    /// `None` follows the first available port; `Some` pins a name.
    pub selected: Option<String>,
    tx: Sender<Packet>,
    pub rx: Receiver<Packet>,
    conn: Option<MidiInputConnection<()>>,
    demo: Option<Arc<AtomicBool>>,
    pub demo_running: bool,
}

impl MidiHub {
    pub fn new() -> Self {
        let (tx, rx) = smol::channel::unbounded();
        let mut hub = Self {
            status: MidiStatus::NoDevice,
            devices: Vec::new(),
            selected: None,
            tx,
            rx,
            conn: None,
            demo: None,
            demo_running: false,
        };
        hub.select(None);
        hub
    }

    /// Re-read the system ports. Connecting is separate so the caller can pick
    /// a remembered device by name instead of blindly taking the first one.
    pub fn refresh_devices(&mut self) {
        self.devices = match MidiInput::new("KeyChange") {
            Ok(input) => input
                .ports()
                .iter()
                .filter_map(|port| input.port_name(port).ok())
                .collect(),
            Err(_) => Vec::new(),
        };
    }

    /// Open a port, or the first available one when `name` is `None`. Selecting
    /// a device that is not plugged in leaves the hub waiting rather than
    /// silently jumping to another keyboard.
    pub fn select(&mut self, name: Option<String>) {
        self.conn = None;
        self.selected = name;
        self.refresh_devices();
        self.status = MidiStatus::NoDevice;
        let tx = self.tx.clone();
        let input = match MidiInput::new("KeyChange") {
            Ok(input) => input,
            Err(err) => {
                self.status = MidiStatus::Error(err.to_string());
                return;
            }
        };
        let ports = input.ports();
        let wanted = self.selected.clone();
        let chosen = ports.iter().find_map(|port| {
            let port_name = input
                .port_name(port)
                .unwrap_or_else(|_| "MIDI keyboard".to_string());
            if wanted.as_ref().is_none_or(|want| want == &port_name) {
                Some((port.clone(), port_name))
            } else {
                None
            }
        });
        let Some((port, name)) = chosen else {
            return;
        };
        match input.connect(
            &port,
            "keychange-in",
            move |_stamp, message, _| {
                let _ = tx.try_send(Packet {
                    bytes: message.to_vec(),
                    source: Source::Midi,
                });
            },
            (),
        ) {
            Ok(conn) => {
                self.conn = Some(conn);
                self.status = MidiStatus::Connected(name);
            }
            Err(err) => {
                self.status = MidiStatus::Error(err.to_string());
            }
        }
    }

    pub fn reconnect(&mut self) {
        let selected = self.selected.clone();
        self.select(selected);
    }

    /// Notice a cable pulled or a keyboard plugged in. Returns an event only
    /// when the state actually changed, so the UI can show a toast once.
    pub fn poll(&mut self) -> Option<MidiEvent> {
        let before = self.devices.clone();
        self.refresh_devices();
        let changed = before != self.devices;
        if let MidiStatus::Connected(name) = &self.status {
            if !self.devices.iter().any(|device| device == name) {
                self.conn = None;
                self.status = MidiStatus::NoDevice;
                return Some(MidiEvent::Disconnected);
            }
        } else if !self.devices.is_empty() {
            let wanted = self.selected.clone();
            if wanted
                .as_ref()
                .is_none_or(|want| self.devices.iter().any(|d| d == want))
            {
                let was_live = self.status.is_live();
                self.reconnect();
                if !was_live && self.status.is_live() {
                    return Some(MidiEvent::Connected);
                }
            }
        }
        changed.then_some(MidiEvent::DevicesChanged)
    }

    pub fn start_demo(&mut self) {
        self.stop_demo();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let tx = self.tx.clone();
        self.demo = Some(cancel);
        self.demo_running = true;
        thread::spawn(move || run_demo(tx, flag));
    }

    pub fn stop_demo(&mut self) {
        if let Some(flag) = self.demo.take() {
            flag.store(true, Ordering::Relaxed);
        }
        self.demo_running = false;
    }
}

impl Default for MidiHub {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiMsg {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8 },
    Control { cc: u8, value: u8 },
    Aftertouch { pressure: u8 },
    PitchBend { value: i16 },
}

/// Decode one complete MIDI message. Unknown and system messages are ignored.
pub fn parse(bytes: &[u8]) -> Option<MidiMsg> {
    let status = *bytes.first()?;
    if status < 0x80 {
        return None;
    }
    let kind = status & 0xF0;
    match kind {
        0x80 => Some(MidiMsg::NoteOff {
            note: *bytes.get(1)? & 0x7F,
        }),
        0x90 => {
            let note = *bytes.get(1)? & 0x7F;
            let velocity = *bytes.get(2)? & 0x7F;
            if velocity == 0 {
                Some(MidiMsg::NoteOff { note })
            } else {
                Some(MidiMsg::NoteOn { note, velocity })
            }
        }
        0xB0 => Some(MidiMsg::Control {
            cc: *bytes.get(1)? & 0x7F,
            value: *bytes.get(2)? & 0x7F,
        }),
        0xD0 => Some(MidiMsg::Aftertouch {
            pressure: *bytes.get(1)? & 0x7F,
        }),
        0xE0 => {
            let lsb = (*bytes.get(1)? & 0x7F) as i16;
            let msb = (*bytes.get(2)? & 0x7F) as i16;
            Some(MidiMsg::PitchBend {
                value: ((msb << 7) | lsb) - 8192,
            })
        }
        _ => None,
    }
}

pub fn note_on(note: u8, velocity: u8) -> Vec<u8> {
    vec![0x90, note & 0x7F, velocity & 0x7F]
}

pub fn note_off(note: u8) -> Vec<u8> {
    vec![0x80, note & 0x7F, 0]
}

pub fn cc(control: u8, value: u8) -> Vec<u8> {
    vec![0xB0, control & 0x7F, value & 0x7F]
}

pub fn aftertouch(pressure: u8) -> Vec<u8> {
    vec![0xD0, pressure & 0x7F]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemoStep {
    pub at_ms: u64,
    pub bytes: Vec<u8>,
}

/// A short scripted study: a dozen word chords across three banks, a typo that
/// quantizes, and an autocomplete accepted by aftertouch. Runs through the real
/// MIDI parser.
pub fn demo_script() -> Vec<DemoStep> {
    let mut raw: Vec<(u64, Vec<u8>)> = Vec::new();
    let triads: [[u8; 3]; 6] = [
        [48, 52, 55], // the
        [50, 53, 57], // and
        [52, 55, 59], // you
        [53, 57, 60], // that
        [55, 59, 62], // have
        [57, 60, 64], // with
    ];
    let mut at = 200u64;
    for triad in triads {
        for (i, n) in triad.iter().enumerate() {
            let start = at + i as u64 * 12;
            raw.push((start, note_on(*n, 96)));
            raw.push((start + 160, note_off(*n)));
        }
        at += 640;
    }

    // "teh" → the space arrives with the next note → quantized to "the".
    for (n, start) in [(50u8, 4_400u64), (48, 4_700), (60, 4_900)] {
        raw.push((start, note_on(n, 72)));
        raw.push((start + 90, note_off(n)));
    }
    raw.push((5_050, cc(64, 127))); // sustain on
    for (n, start) in [(60u8, 5_600u64), (48, 5_920), (65, 6_140)] {
        raw.push((start, note_on(n, 72)));
        raw.push((start + 90, note_off(n)));
    }
    raw.push((5_760, cc(64, 0))); // sustain off after the gap

    // "hel" → wheel to hello → aftertouch accepts.
    raw.push((6_260, cc(1, 0))); // wheel → first candidate
    raw.push((6_520, aftertouch(80)));
    raw.push((6_900, cc(11, 96))); // expression opens the gate
    raw.push((7_100, cc(64, 127))); // leave the pedal glowing

    // The low bank: the same six pads below C3 type six more words.
    let low_triads: [[u8; 3]; 3] = [
        [36, 40, 43], // of
        [38, 41, 45], // to
        [40, 43, 47], // in
    ];
    let mut at = 7_300u64;
    for triad in low_triads {
        for (i, n) in triad.iter().enumerate() {
            let start = at + i as u64 * 12;
            raw.push((start, note_on(*n, 92)));
            raw.push((start + 160, note_off(*n)));
        }
        at += 640;
    }

    // Extra triads use a black key: this, but, from.
    let extra_triads: [[u8; 3]; 3] = [
        [61, 65, 68], // this
        [62, 66, 69], // but
        [63, 67, 70], // from
    ];
    for triad in extra_triads {
        for (i, n) in triad.iter().enumerate() {
            let start = at + i as u64 * 12;
            raw.push((start, note_on(*n, 94)));
            raw.push((start + 160, note_off(*n)));
        }
        at += 640;
    }
    raw.push((at, cc(64, 0))); // settle the pedal after the last word

    raw.sort_by_key(|(at, _)| *at);
    raw.into_iter()
        .map(|(at, bytes)| DemoStep { at_ms: at, bytes })
        .collect()
}

pub fn run_demo(tx: Sender<Packet>, cancel: Arc<AtomicBool>) {
    let mut last = 0u64;
    for step in demo_script() {
        let wait = step.at_ms.saturating_sub(last);
        if wait > 0 {
            thread::sleep(Duration::from_millis(wait));
        }
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        if tx
            .send_blocking(Packet {
                bytes: step.bytes,
                source: Source::Demo,
            })
            .is_err()
        {
            return;
        }
        last = step.at_ms;
    }
    let _ = tx.send_blocking(Packet {
        bytes: Vec::new(),
        source: Source::DemoEnd,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{Engine, BLACK_MIDI, WHITE_MIDI};

    fn feed(engine: &mut Engine, bytes: &[u8], now: u64) {
        match parse(bytes) {
            Some(MidiMsg::NoteOn { note, velocity }) => {
                engine.note_on(note, velocity, now);
            }
            Some(MidiMsg::NoteOff { note }) => engine.note_off(note),
            Some(MidiMsg::Control { cc, value }) => {
                engine.control(cc, value, now);
            }
            Some(MidiMsg::Aftertouch { pressure }) => {
                engine.aftertouch(pressure, now);
            }
            Some(MidiMsg::PitchBend { value }) => {
                engine.pitch_bend(value);
            }
            None => {}
        }
    }

    #[test]
    fn parses_note_on_off_and_velocity_zero() {
        assert_eq!(
            parse(&[0x90, 60, 100]),
            Some(MidiMsg::NoteOn {
                note: 60,
                velocity: 100
            })
        );
        assert_eq!(parse(&[0x90, 60, 0]), Some(MidiMsg::NoteOff { note: 60 }));
        assert_eq!(parse(&[0x80, 48, 64]), Some(MidiMsg::NoteOff { note: 48 }));
    }

    #[test]
    fn parses_cc_aftertouch_and_bend() {
        assert_eq!(
            parse(&[0xB0, 64, 127]),
            Some(MidiMsg::Control { cc: 64, value: 127 })
        );
        assert_eq!(
            parse(&[0xD0, 42]),
            Some(MidiMsg::Aftertouch { pressure: 42 })
        );
        assert_eq!(parse(&[0xE0, 0, 64]), Some(MidiMsg::PitchBend { value: 0 }));
        assert_eq!(
            parse(&[0xE0, 127, 127]),
            Some(MidiMsg::PitchBend { value: 8191 })
        );
        assert_eq!(
            parse(&[0xE0, 0, 0]),
            Some(MidiMsg::PitchBend { value: -8192 })
        );
    }

    #[test]
    fn ignores_malformed_and_system_messages() {
        assert_eq!(parse(&[]), None);
        assert_eq!(parse(&[0xF8]), None);
        assert_eq!(parse(&[0x90, 60]), None);
        assert_eq!(parse(&[0x30, 60, 1]), None);
    }

    #[test]
    fn demo_script_runs_through_the_engine() {
        let mut engine = Engine::new();
        for step in demo_script() {
            feed(&mut engine, &step.bytes, step.at_ms);
        }
        let text = engine.text();
        assert!(text.contains("the and you that have with"), "{text}");
        assert!(text.contains("the hello"), "{text}");
        assert!(text.contains("of to in"), "{text}");
        assert!(text.contains("this but from"), "{text}");
        assert!(engine.corrections >= 1);
        assert!(engine.chords_played >= 12);
    }

    #[test]
    fn demo_is_monotonic() {
        let script = demo_script();
        assert!(script.windows(2).all(|w| w[0].at_ms <= w[1].at_ms));
        assert!(script.len() > 40);
    }

    #[test]
    fn raw_midi_round_trips_to_engine() {
        let mut engine = Engine::new();
        for (i, note) in WHITE_MIDI.iter().take(3).enumerate() {
            feed(&mut engine, &note_on(*note, 90), i as u64 * 400);
        }
        assert_eq!(engine.text(), "eta");
        feed(&mut engine, &note_on(BLACK_MIDI[0], 105), 2_000);
        assert_eq!(engine.text(), "eta,");
    }

    #[test]
    fn pitch_bend_edge_triggers_once() {
        let mut engine = Engine::new();
        engine.note_on(60, 70, 0);
        engine.note_on(48, 70, 500);
        engine.note_on(65, 70, 900);
        let n = engine.candidates().len();
        assert!(n > 1);
        engine.pitch_bend(7000);
        assert_eq!(engine.candidate_index(), 1);
        engine.pitch_bend(7100);
        assert_eq!(engine.candidate_index(), 1);
        engine.pitch_bend(0);
        engine.pitch_bend(-7000);
        assert_eq!(engine.candidate_index(), 0);
    }
}
