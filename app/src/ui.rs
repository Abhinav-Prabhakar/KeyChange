//! The single screen. One instrument: score, pads, keys, pedals.

use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    div, point, px, rgb, rgba, AnyElement, BoxShadow, ClipboardItem, Context, DefiniteLength, Div,
    FocusHandle, Focusable, FontWeight, KeyDownEvent, KeyUpEvent, Keystroke, MouseButton, Render,
    SharedString, Window,
};

use crate::engine::{
    chord_pads, freq_for, note_name, Control, Engine, KeyKind, Layer, Pedal, BLACK_MIDI,
    BLACK_SLOTS, DEFAULT_LETTERS, WHITE_MIDI,
};
use crate::midi::{self, MidiHub, MidiMsg, MidiStatus, Packet, Source};
use crate::settings::Settings;

// ─────────────────────────────────── palette ───────────────────────────────

fn paper() -> gpui::Rgba {
    rgb(0xf6edda)
}
fn card() -> gpui::Rgba {
    rgb(0xfffbf1)
}
fn card_warm() -> gpui::Rgba {
    rgb(0xfff3d2)
}
fn side() -> gpui::Rgba {
    rgb(0xefe6d0)
}
fn hair() -> gpui::Rgba {
    rgb(0xe0d5bd)
}
fn hair_dark() -> gpui::Rgba {
    rgb(0xc9bda3)
}
fn ink() -> gpui::Rgba {
    rgb(0x17130c)
}
fn ink_soft() -> gpui::Rgba {
    rgb(0x3a342a)
}
fn muted() -> gpui::Rgba {
    rgb(0x6f6a5e)
}
fn faint() -> gpui::Rgba {
    rgb(0x9a927f)
}
fn red() -> gpui::Rgba {
    rgb(0xff2e1f)
}
fn blue() -> gpui::Rgba {
    rgb(0x2233ff)
}
fn yellow() -> gpui::Rgba {
    rgb(0xffc700)
}
fn green() -> gpui::Rgba {
    rgb(0x0e7a4f)
}
fn ivory() -> gpui::Rgba {
    rgb(0xfffdf6)
}
fn keydark() -> gpui::Rgba {
    rgb(0x1b1915)
}
fn keydark2() -> gpui::Rgba {
    rgb(0x24211b)
}

const UI: &str = "Space Grotesk";
const DISPLAY: &str = "Unbounded";
const MONO: &str = "JetBrains Mono";
const SERIF: &str = "Instrument Serif";

fn op(c: gpui::Rgba, a: f32) -> gpui::Hsla {
    let h: gpui::Hsla = c.into();
    h.opacity(a)
}

fn soft_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: rgba(0x17130c1a).into(),
        offset: point(px(0.), px(10.)),
        blur_radius: px(28.),
        spread_radius: px(-6.),
    }]
}

fn float_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: rgba(0x17130c30).into(),
        offset: point(px(0.), px(24.)),
        blur_radius: px(60.),
        spread_radius: px(-12.),
    }]
}

fn mono(size: f32) -> Div {
    div()
        .font_family(MONO)
        .font_weight(FontWeight::MEDIUM)
        .text_size(px(size))
        .text_color(muted())
}

fn caps(text: impl Into<SharedString>, size: f32) -> Div {
    mono(size).child(text.into())
}

// ─────────────────────────────────── state ─────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hover {
    White(usize),
    Black(usize),
    Pad(usize),
    Pedal(usize),
    Gate,
    Wheel,
    Profile,
    Midi,
    Demo,
    Help,
    MapCol(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverlayTab {
    Map,
    Gestures,
    Learn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NoticeKind {
    Info,
    Good,
}

struct Notice {
    text: String,
    at: u64,
    kind: NoticeKind,
}

struct Pulse {
    seq: u64,
    at: u64,
    target: Option<KeyKind>,
    pedal: Option<usize>,
}

pub struct KeyChangeApp {
    focus: FocusHandle,
    engine: Engine,
    hub: MidiHub,
    started: Instant,
    hover: Option<Hover>,
    hint: Option<String>,
    overlay: Option<OverlayTab>,
    edit_mode: bool,
    notice: Option<Notice>,
    pulse: Pulse,
    seq: u64,
    demo_notes: u32,
    caret_on: bool,
    learn: Option<Control>,
}

impl KeyChangeApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let hub = MidiHub::new();
        let rx = hub.rx.clone();
        cx.spawn(async move |this, cx| {
            while let Ok(packet) = rx.recv().await {
                if this
                    .update(cx, |this, cx| this.handle_packet(packet, cx))
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        let now = Instant::now();
        let mut engine = Engine::new();
        Settings::load().apply_to(&mut engine);
        let mut app = Self {
            focus: cx.focus_handle(),
            engine,
            hub,
            started: now,
            hover: None,
            hint: None,
            overlay: None,
            edit_mode: false,
            notice: None,
            pulse: Pulse {
                seq: 0,
                at: 0,
                target: None,
                pedal: None,
            },
            seq: 0,
            demo_notes: 0,
            caret_on: true,
            learn: None,
        };
        if std::env::var("KC_DEMO").is_ok() {
            app.toggle_demo();
        }
        cx.spawn(async move |this, cx| loop {
            smol::Timer::after(Duration::from_millis(540)).await;
            if this
                .update(cx, |this, cx| {
                    this.caret_on = !this.caret_on;
                    cx.notify();
                })
                .is_err()
            {
                break;
            }
        })
        .detach();
        app
    }

    fn now(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    /// Write layouts, velocity, gate, layer and learned CCs. Failure is a
    /// notice, never an interruption.
    fn persist(&mut self) {
        if let Err(err) = Settings::from_engine(&self.engine).save() {
            self.set_notice(format!("Settings not saved · {err}"), NoticeKind::Info);
        }
    }

    fn set_notice(&mut self, text: impl Into<String>, kind: NoticeKind) {
        self.notice = Some(Notice {
            text: text.into(),
            at: self.now(),
            kind,
        });
    }

    fn toggle_demo(&mut self) {
        if self.hub.demo_running {
            self.hub.stop_demo();
            self.set_notice("Simulation stopped", NoticeKind::Info);
        } else {
            self.engine.clear();
            self.engine.sustain = false;
            self.engine.set_layer(Layer::Abc);
            self.demo_notes = 0;
            self.hub.start_demo();
            self.set_notice("Simulated MIDI · play your words loud", NoticeKind::Good);
        }
    }

    #[allow(clippy::collapsible_match)]
    fn handle_packet(&mut self, packet: Packet, cx: &mut Context<Self>) {
        if packet.source == Source::DemoEnd {
            self.hub.demo_running = false;
            self.set_notice(
                format!("Simulation complete · {} notes", self.demo_notes),
                NoticeKind::Good,
            );
            cx.notify();
            return;
        }
        let message = midi::parse(&packet.bytes);
        if packet.source == Source::Demo && matches!(&message, Some(MidiMsg::NoteOn { .. })) {
            self.demo_notes = self.demo_notes.saturating_add(1);
        }
        let now = self.now();
        match message {
            Some(MidiMsg::NoteOn { note, velocity }) => {
                let outcome = self.engine.note_on(note, velocity, now);
                if let Some(k) = outcome.key {
                    self.flash(k, None);
                }
                if let Some(c) = outcome.correction {
                    self.set_notice(format!("{} → {}", c.from, c.to), NoticeKind::Good);
                }
            }
            Some(MidiMsg::NoteOff { note }) => self.engine.note_off(note),
            Some(MidiMsg::Control { cc, value }) => {
                if let Some(control) = self.learn.take() {
                    self.engine.cc_map.assign(control, cc);
                    self.persist();
                    self.set_notice(
                        format!("{} learned · CC {cc}", control.label()),
                        NoticeKind::Good,
                    );
                } else if let Some(outcome) = self.engine.control(cc, value, now) {
                    if let Some(k) = outcome.key {
                        self.flash(k, None);
                    }
                }
            }
            Some(MidiMsg::Aftertouch { pressure }) => {
                if self.engine.aftertouch(pressure, now) {
                    self.set_notice("Completion accepted", NoticeKind::Good);
                }
            }
            Some(MidiMsg::PitchBend { value }) => {
                self.engine.pitch_bend(value);
            }
            None => {}
        }
        cx.notify();
    }

    fn flash(&mut self, key: KeyKind, pedal: Option<usize>) {
        self.seq = self.seq.wrapping_add(1);
        self.pulse = Pulse {
            seq: self.seq,
            at: self.now(),
            target: Some(key),
            pedal,
        };
    }

    fn pulse_alive(&self, seq: u64, ms: u64) -> bool {
        seq == self.pulse.seq && self.now().saturating_sub(self.pulse.at) < ms
    }

    fn set_hover(&mut self, hover: Option<Hover>, hint: Option<String>) {
        self.hover = hover;
        self.hint = hint;
    }

    fn hover_is(&self, target: Hover) -> bool {
        self.hover == Some(target)
    }

    fn hover_pad(&self) -> Option<usize> {
        match self.hover {
            Some(Hover::Pad(i)) => Some(i),
            _ => None,
        }
    }

    fn current_word(&self) -> String {
        self.engine
            .text()
            .split_whitespace()
            .last()
            .unwrap_or("")
            .to_string()
    }

    fn active_pad(&self) -> Option<usize> {
        let word = self.current_word();
        chord_pads().iter().position(|p| p.word == word)
    }

    fn click_note(&mut self, note: u8, velocity: u8, key: KeyKind, cx: &mut Context<Self>) {
        let now = self.now();
        let outcome = self.engine.note_on(note, velocity, now);
        if let Some(c) = outcome.correction {
            self.set_notice(format!("{} → {}", c.from, c.to), NoticeKind::Good);
        }
        self.flash(key, None);
        cx.spawn(async move |this, cx| {
            smol::Timer::after(Duration::from_millis(150)).await;
            this.update(cx, |this, cx| {
                this.engine.note_off(note);
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    fn play_pad(&mut self, index: usize, cx: &mut Context<Self>) {
        let pads = chord_pads();
        let Some(pad) = pads.get(index) else { return };
        let now = self.now();
        let notes = pad.notes;
        for n in notes {
            self.engine.note_on(n, self.engine.base_velocity, now);
        }
        self.seq = self.seq.wrapping_add(1);
        self.pulse = Pulse {
            seq: self.seq,
            at: now,
            target: None,
            pedal: None,
        };
        cx.spawn(async move |this, cx| {
            smol::Timer::after(Duration::from_millis(170)).await;
            this.update(cx, |this, cx| {
                for n in notes {
                    this.engine.note_off(n);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    fn on_keystroke(&mut self, keystroke: &Keystroke, cx: &mut Context<Self>) {
        let key = keystroke.key.as_str();
        let modifiers = keystroke.modifiers;
        let command = modifiers.platform || modifiers.control;
        let now = self.now();
        if command {
            match key {
                "d" => self.toggle_demo(),
                "z" => {
                    self.engine.undo();
                }
                "c" => {
                    let text = self.engine.text();
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                    self.set_notice("Copied to clipboard", NoticeKind::Info);
                }
                "k" => {
                    self.engine.clear();
                    self.set_notice("Cleared", NoticeKind::Info);
                }
                "q" => {
                    cx.quit();
                }
                "," => {
                    self.overlay = if self.overlay == Some(OverlayTab::Learn) {
                        None
                    } else {
                        Some(OverlayTab::Learn)
                    };
                    self.learn = None;
                }
                _ => return,
            }
            cx.notify();
            return;
        }
        if key == "escape" {
            if self.overlay.is_some() {
                self.overlay = None;
                self.learn = None;
                self.edit_mode = false;
                self.engine.cancel_edit();
            } else if self.engine.edit_mode {
                self.engine.cancel_edit();
                self.edit_mode = false;
            }
            cx.notify();
            return;
        }
        if key == "tab" {
            self.learn = None;
            self.overlay = match self.overlay {
                Some(OverlayTab::Map) => None,
                _ => Some(OverlayTab::Map),
            };
            cx.notify();
            return;
        }
        if key == "/" {
            self.overlay = Some(OverlayTab::Gestures);
            self.learn = None;
            cx.notify();
            return;
        }
        if self.overlay.is_some() {
            return;
        }
        match key {
            "backspace" | "delete" => {
                self.engine.backspace();
            }
            "space" => self.engine.type_space(now),
            "enter" => self.engine.commit_word(now),
            "left" | "[" => {
                self.engine.cycle_layer(-1);
                self.persist();
            }
            "right" | "]" => {
                self.engine.cycle_layer(1);
                self.persist();
            }
            "up" => self.engine.scrub_candidates(-1),
            "down" => self.engine.scrub_candidates(1),
            _ => {
                if let Some(outcome) = self.engine.key_qwerty(key, now) {
                    if let Some(k) = outcome.key {
                        self.flash(k, None);
                    }
                    if let Some(c) = outcome.correction {
                        self.set_notice(format!("{} → {}", c.from, c.to), NoticeKind::Good);
                    }
                }
            }
        }
        cx.notify();
    }

    // ── chrome ──────────────────────────────────────────────────────────────

    fn toolbar(&self, cx: &mut Context<Self>) -> Div {
        let layer_btn = |this: &Self, layer: Layer, label: &'static str, cx: &mut Context<Self>| {
            let on = this.engine.layer == layer;
            div()
                .id(label)
                .px(px(11.))
                .py(px(4.))
                .rounded(px(6.))
                .bg(if on { card() } else { side() })
                .text_color(if on { ink() } else { muted() })
                .font_family(MONO)
                .font_weight(if on {
                    FontWeight::BOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_size(px(10.5))
                .shadow(if on { soft_shadow() } else { Vec::new() })
                .cursor_pointer()
                .hover(|s| s.text_color(ink()))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.engine.set_layer(layer);
                        this.set_hover(None, None);
                        cx.notify();
                    }),
                )
                .child(label)
        };

        let midi_live = self.hub.status.is_live();
        let midi_dot = if self.hub.demo_running {
            yellow()
        } else if midi_live {
            green()
        } else if matches!(self.hub.status, MidiStatus::Error(_)) {
            red()
        } else {
            faint()
        };
        let midi_text = if self.hub.demo_running {
            "SIM · DEMO".to_string()
        } else {
            match &self.hub.status {
                MidiStatus::NoDevice => "NO MIDI".to_string(),
                MidiStatus::Connected(name) => name.to_uppercase(),
                MidiStatus::Error(_) => "MIDI ERROR".to_string(),
            }
        };

        let profile = self.engine.active_profile().name.clone();
        let quality = self.engine.quality();

        div()
            .flex()
            .flex_row()
            .items_center()
            .h(px(58.))
            .pl(px(84.))
            .pr(px(16.))
            .gap(px(14.))
            .bg(paper())
            // brand
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(9.))
                    .child(
                        div()
                            .size(px(28.))
                            .rounded(px(8.))
                            .bg(yellow())
                            .border_1()
                            .border_color(ink())
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .font_family(DISPLAY)
                                    .font_weight(FontWeight::BLACK)
                                    .text_size(px(15.))
                                    .text_color(ink())
                                    .child("♪"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(1.))
                            .child(
                                div()
                                    .font_family(DISPLAY)
                                    .font_weight(FontWeight::EXTRA_BOLD)
                                    .text_size(px(12.))
                                    .text_color(ink())
                                    .child("KEYCHANGE"),
                            )
                            .child(caps("PLAY YOUR WORDS LOUD", 6.5).text_color(faint())),
                    ),
            )
            .child(div().w(px(1.)).h(px(22.)).bg(hair()))
            // layers
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(2.))
                    .p(px(2.))
                    .rounded(px(8.))
                    .bg(side())
                    .child(layer_btn(self, Layer::Abc, "abc", cx))
                    .child(layer_btn(self, Layer::Upper, "ABC", cx))
                    .child(layer_btn(self, Layer::Symbols, "123", cx)),
            )
            // hover inspector hint (tucked)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h_full()
                    .child(
                        mono(9.5)
                            .text_color(faint())
                            .when_some(self.hint.clone(), |el, hint| el.child(hint))
                            .when(self.hint.is_none(), |el| {
                                el.child("\u{266A}  MOD WHEEL  ·  AFTERTOUCH  ·  SUSTAIN")
                            }),
                    ),
            )
            // profile
            .child(
                div()
                    .id("profile")
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.))
                    .h(px(30.))
                    .px(px(10.))
                    .rounded(px(8.))
                    .bg(card())
                    .border_1()
                    .border_color(hair())
                    .cursor_pointer()
                    .hover(|s| s.border_color(hair_dark()).bg(card_warm()))
                    .on_hover(cx.listener(|this, hovered, _, cx| {
                        if *hovered {
                            this.set_hover(
                                Some(Hover::Profile),
                                Some("Map — Huffman order, remap keys".to_string()),
                            );
                        } else if this.hover_is(Hover::Profile) {
                            this.set_hover(None, None);
                        }
                        cx.notify();
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.overlay = Some(OverlayTab::Map);
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .w(px(22.))
                            .h(px(14.))
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .justify_center()
                            .child(div().h(px(3.)).w(px(22.)).rounded_full().bg(ink()))
                            .child(div().h(px(3.)).w(px(16.)).rounded_full().bg(ink()))
                            .child(div().h(px(3.)).w(px(10.)).rounded_full().bg(ink())),
                    )
                    .child(
                        div()
                            .font_family(UI)
                            .font_weight(FontWeight::MEDIUM)
                            .text_size(px(11.))
                            .text_color(ink())
                            .child(profile),
                    )
                    .child(
                        div().w(px(26.)).h(px(3.)).rounded_full().bg(hair()).child(
                            div()
                                .w(px(26.0 * (quality / 100.0)))
                                .h_full()
                                .rounded_full()
                                .bg(green()),
                        ),
                    ),
            )
            // MIDI
            .child(
                div()
                    .id("midi")
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(7.))
                    .h(px(30.))
                    .px(px(10.))
                    .rounded(px(8.))
                    .cursor_pointer()
                    .hover(|s| s.bg(card()))
                    .on_hover(cx.listener(|this, hovered, _, cx| {
                        if *hovered {
                            let hint = match &this.hub.status {
                                MidiStatus::Connected(name) => format!("MIDI LIVE — {name}"),
                                MidiStatus::Error(err) => format!("MIDI error — {err}"),
                                MidiStatus::NoDevice => {
                                    "No MIDI found — click to scan again".to_string()
                                }
                            };
                            this.set_hover(Some(Hover::Midi), Some(hint));
                        } else if this.hover_is(Hover::Midi) {
                            this.set_hover(None, None);
                        }
                        cx.notify();
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.hub.reconnect();
                            let text = match &this.hub.status {
                                MidiStatus::Connected(name) => format!("MIDI connected · {name}"),
                                _ => "No MIDI device found".to_string(),
                            };
                            this.set_notice(text, NoticeKind::Info);
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .size(px(7.))
                            .rounded_full()
                            .bg(midi_dot)
                            .shadow(if midi_live {
                                vec![BoxShadow {
                                    color: op(green(), 0.35),
                                    offset: point(px(0.), px(0.)),
                                    blur_radius: px(7.),
                                    spread_radius: px(0.),
                                }]
                            } else {
                                Vec::new()
                            }),
                    )
                    .child(
                        mono(9.)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(if midi_live || self.hub.demo_running {
                                ink()
                            } else {
                                muted()
                            })
                            .max_w(px(150.))
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(midi_text),
                    ),
            )
            // demo
            .child(
                div()
                    .id("demo")
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(7.))
                    .h(px(32.))
                    .px(px(12.))
                    .rounded(px(8.))
                    .bg(if self.hub.demo_running { red() } else { ink() })
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.86))
                    .on_hover(cx.listener(|this, hovered, _, cx| {
                        if *hovered {
                            this.set_hover(
                                Some(Hover::Demo),
                                Some(if this.hub.demo_running {
                                    "Stop the simulated performance".to_string()
                                } else {
                                    "Hear a simulated MIDI keyboard (⌘D)".to_string()
                                }),
                            );
                        } else if this.hover_is(Hover::Demo) {
                            this.set_hover(None, None);
                        }
                        cx.notify();
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.toggle_demo();
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .font_family(MONO)
                            .text_size(px(9.))
                            .text_color(yellow())
                            .child(if self.hub.demo_running { "■" } else { "▶" }),
                    )
                    .child(
                        div()
                            .font_family(MONO)
                            .font_weight(FontWeight::BOLD)
                            .text_size(px(9.))
                            .text_color(paper())
                            .child(if self.hub.demo_running {
                                "STOP"
                            } else {
                                "DEMO"
                            }),
                    ),
            )
            // help
            .child(
                div()
                    .id("help")
                    .size(px(28.))
                    .rounded_full()
                    .border_1()
                    .border_color(hair())
                    .bg(card())
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| s.bg(card_warm()).border_color(hair_dark()))
                    .on_hover(cx.listener(|this, hovered, _, cx| {
                        if *hovered {
                            this.set_hover(
                                Some(Hover::Help),
                                Some("Chords, intervals and pedals (/) ".to_string()),
                            );
                        } else if this.hover_is(Hover::Help) {
                            this.set_hover(None, None);
                        }
                        cx.notify();
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.overlay = Some(OverlayTab::Gestures);
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .font_family(MONO)
                            .font_weight(FontWeight::BOLD)
                            .text_size(px(11.))
                            .text_color(ink())
                            .child("?"),
                    ),
            )
    }

    // ── score ───────────────────────────────────────────────────────────────

    fn stage(&self, cx: &mut Context<Self>) -> Div {
        let candidates = self.engine.candidates();
        let active_candidate = self.engine.candidate_index();
        let recent: Vec<_> = self.engine.recent.iter().cloned().collect();

        div()
            .flex_1()
            .min_h(px(250.))
            .rounded(px(16.))
            .bg(card())
            .border_1()
            .border_color(hair())
            .shadow(soft_shadow())
            .p(px(20.))
            .flex()
            .flex_col()
            .gap(px(10.))
            // score header
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(7.))
                            .child(
                                div()
                                    .size(px(5.))
                                    .rounded_full()
                                    .bg(if self.hub.demo_running {
                                        yellow()
                                    } else if self.hub.status.is_live() {
                                        green()
                                    } else {
                                        faint()
                                    }),
                            )
                            .child(caps("SCORE", 8.5).text_color(faint())),
                    )
                    .child(
                        mono(8.5)
                            .text_color(faint())
                            .child("C3 — C5  ·  15 KEYS  ·  6 WORDS"),
                    ),
            )
            // body: transcript + rail
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .gap(px(18.))
                    .min_h(px(0.))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .min_w(px(0.))
                            .child(self.transcript(cx))
                            .child(self.candidates_row(candidates, active_candidate, cx))
                            .child(self.staff(recent)),
                    )
                    .child(self.rail()),
            )
    }

    fn transcript(&self, cx: &mut Context<Self>) -> Div {
        let mut row = div()
            .flex()
            .flex_wrap()
            .items_end()
            .gap_y(px(2.))
            .overflow_hidden()
            .w_full()
            .min_h(px(72.));

        if self.engine.cells.is_empty() {
            row = row.child(
                div()
                    .font_family(SERIF)
                    .italic()
                    .text_size(px(37.))
                    .text_color(faint())
                    .child("play your words loud"),
            );
        } else {
            let mut word: Vec<AnyElement> = Vec::new();
            let mut children: Vec<AnyElement> = Vec::new();
            let flush = |word: &mut Vec<AnyElement>, children: &mut Vec<AnyElement>| {
                if !word.is_empty() {
                    children.push(
                        div()
                            .flex()
                            .flex_row()
                            .items_end()
                            .children(word.drain(..))
                            .into_any_element(),
                    );
                }
            };
            for cell in &self.engine.cells {
                if cell.ch.is_whitespace() {
                    flush(&mut word, &mut children);
                    children.push(div().w(px(10.)).into_any_element());
                    continue;
                }
                let color = if cell.corrected {
                    green()
                } else if cell.kind == crate::engine::CharKind::Symbol {
                    blue()
                } else {
                    ink()
                };
                let weight = if cell.kind == crate::engine::CharKind::Word {
                    FontWeight::SEMIBOLD
                } else if cell.vel >= 105 {
                    FontWeight::BOLD
                } else if cell.vel >= 75 {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                };
                let mut el = div()
                    .font_family(UI)
                    .font_weight(weight)
                    .text_size(px(37.))
                    .line_height(px(46.))
                    .text_color(color)
                    .child(cell.ch.to_string());
                if cell.corrected {
                    el = el.bg(op(green(), 0.12)).rounded(px(3.)).px(px(1.));
                }
                if cell.vel < 45 {
                    el = el.italic();
                }
                word.push(el.into_any_element());
            }
            flush(&mut word, &mut children);
            row = row.children(children);
        }

        let caret = div()
            .w(px(3.))
            .h(px(38.))
            .ml(px(2.))
            .rounded(px(1.))
            .bg(ink())
            .opacity(if self.caret_on { 1.0 } else { 0.0 });
        row = row.child(caret);

        let _ = cx;
        div()
            .flex_1()
            .w_full()
            .min_h(px(72.))
            .flex()
            .items_center()
            .child(row)
    }

    fn candidates_row(
        &self,
        candidates: Vec<&'static str>,
        active: usize,
        cx: &mut Context<Self>,
    ) -> Div {
        if candidates.is_empty() {
            return div().h(px(28.)).flex().items_center().child(
                mono(8.5)
                    .text_color(faint())
                    .child("PLAY  TWO NOTES  ·  INTERVAL      THREE  ·  WORD"),
            );
        }
        let mut row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .h(px(28.));
        for (i, word) in candidates.into_iter().enumerate() {
            let on = i == active;
            row = row.child(
                div()
                    .id(("cand", i))
                    .h(px(26.))
                    .px(px(11.))
                    .rounded_full()
                    .border_1()
                    .border_color(if on { ink() } else { hair() })
                    .bg(if on { ink() } else { card() })
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .hover(|s| s.bg(card_warm()))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.engine.candidate_index = i;
                            if this.engine.accept_candidate(this.now()) {
                                this.set_notice("Completion accepted", NoticeKind::Good);
                            }
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .font_family(UI)
                            .font_weight(if on {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::MEDIUM
                            })
                            .text_size(px(11.))
                            .text_color(if on { yellow() } else { ink() })
                            .child(word),
                    ),
            );
        }
        row.child(div().flex_1()).child(
            mono(8.)
                .text_color(faint())
                .child("WHEEL ⇅  ·  AFTERTOUCH ⏎"),
        )
    }

    fn staff(&self, recent: Vec<crate::engine::RecentNote>) -> Div {
        let mut lines = div().absolute().top(px(8.)).left(px(0.)).w_full().h_full();
        for i in 0..5 {
            lines = lines.child(
                div()
                    .absolute()
                    .top(px(14. + i as f32 * 11.))
                    .left(px(0.))
                    .w_full()
                    .h(px(1.))
                    .bg(op(ink(), 0.10)),
            );
        }
        let mut notes = div()
            .relative()
            .flex()
            .flex_row()
            .items_start()
            .h_full()
            .w_full();
        let start = recent.len().saturating_sub(20);
        for note in recent.iter().skip(start) {
            let pitch = note.note as f32;
            let y = (68. - ((pitch - 48.) / 24.).clamp(0., 1.) * 56.).clamp(6., 72.);
            let is_black = crate::engine::black_offset(note.note).is_some();
            let color = if is_black {
                blue()
            } else if note.vel >= 105 {
                red()
            } else {
                ink()
            };
            let fresh = note.index == self.engine.node_seq;
            notes = notes.child(
                div()
                    .flex_1()
                    .relative()
                    .h_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(div().h(px(y)))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .child(
                                div()
                                    .size(px(if fresh { 9. } else { 7. }))
                                    .rounded_full()
                                    .bg(color),
                            )
                            .child(div().w(px(1.)).h(px(15.)).bg(op(color, 0.55))),
                    ),
            );
        }
        div()
            .relative()
            .h(px(84.))
            .w_full()
            .rounded(px(8.))
            .bg(op(card_warm(), 0.35))
            .border_1()
            .border_color(op(hair(), 0.6))
            .overflow_hidden()
            .child(lines)
            .child(notes)
            .child(
                div()
                    .absolute()
                    .left(px(8.))
                    .bottom(px(6.))
                    .font_family(MONO)
                    .font_weight(FontWeight::BOLD)
                    .text_size(px(10.))
                    .text_color(faint())
                    .child("𝄞"),
            )
    }

    fn rail(&self) -> Div {
        let hover = self.hover;
        let mut glyph = "♪".to_string();
        let mut caption = "waiting for a note".to_string();
        let mut heat = 0.0f32;
        let mut effort = 0.0f32;
        let mut is_word = false;
        match hover {
            Some(Hover::White(i)) => {
                glyph = self.engine.preview_white(i);
                caption = format!(
                    "{}  ·  {}",
                    note_name(WHITE_MIDI[i.min(14)]),
                    qwerty_hint(i.min(14))
                );
                heat = self.engine.white_heat(i);
                effort = self.engine.white_effort(i);
            }
            Some(Hover::Black(b)) => {
                glyph = self.engine.preview_black(b);
                caption = format!("{}  ·  punctuation", note_name(BLACK_MIDI[b.min(9)]));
                heat = 0.35;
                effort = 2.3;
            }
            Some(Hover::Pad(i)) => {
                if let Some(pad) = chord_pads().get(i) {
                    glyph = pad.word.to_string();
                    caption = format!("{} MAJOR/MINOR  ·  TRIAD", pad.root);
                    is_word = true;
                }
            }
            _ => {
                let token = self.current_word();
                if !token.is_empty() {
                    glyph = token;
                    is_word = true;
                    caption = "current word".to_string();
                }
            }
        }
        let font_size = if glyph.chars().count() > 4 {
            13.
        } else if glyph.chars().count() > 2 {
            17.
        } else {
            26.
        };
        let tile_bg = if is_word { yellow() } else { card_warm() };
        let mut spark = div().flex().flex_row().items_end().gap(px(1.5)).h(px(26.));
        let hist: Vec<u32> = self.engine.wpm_history.iter().copied().collect::<Vec<_>>();
        for (i, v) in hist.iter().rev().take(24).rev().enumerate() {
            let h = 3.0 + (*v as f32 / 240.0).clamp(0.0, 1.0) * 23.0;
            spark = spark.child(div().w(px(2.5)).h(px(h)).rounded(px(1.)).bg(
                if i + 6 >= hist.len().min(24) {
                    op(ink(), 0.75)
                } else {
                    op(hair_dark(), 1.0)
                },
            ));
        }
        if hist.is_empty() {
            for _ in 0..12 {
                spark = spark.child(div().w(px(2.5)).h(px(3.)).rounded(px(1.)).bg(hair()));
            }
        }

        div()
            .w(px(146.))
            .flex()
            .flex_col()
            .gap(px(11.))
            .pl(px(16.))
            .border_l_1()
            .border_color(hair())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(5.))
                    .child(caps("NOW", 8.))
                    .child(
                        div()
                            .size(px(66.))
                            .rounded(px(13.))
                            .bg(tile_bg)
                            .border_1()
                            .border_color(if is_word { ink() } else { hair() })
                            .flex()
                            .items_center()
                            .justify_center()
                            .font_family(DISPLAY)
                            .font_weight(FontWeight::EXTRA_BOLD)
                            .text_size(px(font_size))
                            .text_color(ink())
                            .child(glyph),
                    )
                    .child(
                        mono(8.5)
                            .text_color(muted())
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(caption),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(5.))
                    .child(
                        div().h(px(4.)).w_full().rounded_full().bg(hair()).child(
                            div()
                                .h_full()
                                .rounded_full()
                                .bg(if heat > 0.7 {
                                    yellow()
                                } else if heat > 0.3 {
                                    blue()
                                } else {
                                    hair_dark()
                                })
                                .w(DefiniteLength::Fraction(heat.max(0.06))),
                        ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(3.))
                            .children((0..3).map(|i| {
                                div()
                                    .w(px(14.))
                                    .h(px(3.))
                                    .rounded(px(1.))
                                    .bg(if (i as f32) < effort { ink() } else { hair() })
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .mt_auto()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_baseline()
                            .gap(px(5.))
                            .child(
                                div()
                                    .font_family(MONO)
                                    .font_weight(FontWeight::BOLD)
                                    .text_size(px(24.))
                                    .text_color(ink())
                                    .child(self.engine.wpm(self.now()).to_string()),
                            )
                            .child(caps("WPM", 7.5)),
                    )
                    .child(spark)
                    .child(caps("MOD WHEEL → COMPLETE", 6.8).text_color(faint())),
            )
    }

    // ── pads ────────────────────────────────────────────────────────────────

    fn pads(&self, cx: &mut Context<Self>) -> Div {
        let active = self.active_pad();
        let hovered = self.hover_pad();
        let mut row = div()
            .flex()
            .flex_row()
            .gap(px(7.))
            .h(px(56.))
            .flex_shrink_0();
        for (i, pad) in chord_pads().iter().enumerate() {
            let on = active == Some(i) || hovered == Some(i);
            row = row.child(
                div()
                    .id(("pad", i))
                    .flex_1()
                    .h_full()
                    .px(px(10.))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(9.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(if on { ink() } else { hair() })
                    .bg(if on { card_warm() } else { card() })
                    .cursor_pointer()
                    .hover(|s| s.bg(card_warm()).border_color(ink()))
                    .on_hover(cx.listener(move |this, hovered, _, cx| {
                        if *hovered {
                            this.set_hover(
                                Some(Hover::Pad(i)),
                                Some(format!(
                                    "{} chord — {}  ·  play {} {} {}",
                                    pad.root, pad.word, pad.notes[0], pad.notes[1], pad.notes[2]
                                )),
                            );
                        } else if this.hover_is(Hover::Pad(i)) {
                            this.set_hover(None, None);
                        }
                        cx.notify();
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.play_pad(i, cx);
                        }),
                    )
                    .child(triad_diagram(i, on))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(1.))
                            .child(
                                div()
                                    .font_family(UI)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_size(px(13.))
                                    .text_color(if on { ink() } else { ink_soft() })
                                    .child(pad.word),
                            )
                            .child(caps(pad.root, 7.5).text_color(faint())),
                    ),
            );
        }
        row
    }

    // ── keyboard ────────────────────────────────────────────────────────────

    fn keyboard(&self, cx: &mut Context<Self>) -> Div {
        let held_white = self.engine.held_white();
        let held_black = self.engine.held_black();
        let hover_keys: Vec<usize> = self
            .hover_pad()
            .and_then(|i| chord_pads().get(i))
            .map(|p| p.keys.to_vec())
            .unwrap_or_default();

        let mut slots = div().flex().flex_row().size_full();

        for i in 0..15 {
            let glyph = self.engine.preview_white(i);
            let heat = self.engine.white_heat(i);
            let held = held_white[i] || self.engine.held.contains(&WHITE_MIDI[i]);
            let pulse = self.pulse.target == Some(KeyKind::White(i))
                && self.pulse_alive(self.pulse.seq, 210);
            let in_chord = hover_keys.contains(&i);
            let bg = if held || pulse {
                yellow()
            } else if in_chord {
                rgb(0xffe9a8)
            } else {
                ivory()
            };

            let white = div()
                .id(("white", i))
                .absolute()
                .top(px(0.))
                .left(px(0.))
                .w_full()
                .h_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_end()
                .pb(px(11.))
                .gap(px(5.))
                .bg(bg)
                .when(i < 14, |el| el.border_r_1().border_color(rgb(0xd9cfb9)))
                .cursor_pointer()
                .hover(|s| s.bg(card_warm()))
                .on_hover(cx.listener(move |this, hovered, _, cx| {
                    if *hovered {
                        let glyph = this.engine.preview_white(i);
                        this.set_hover(
                            Some(Hover::White(i)),
                            Some(format!(
                                "{} — types {}  ·  {:.1}% of English",
                                note_name(WHITE_MIDI[i]),
                                glyph,
                                freq_for(&glyph)
                            )),
                        );
                    } else if this.hover_is(Hover::White(i)) {
                        this.set_hover(None, None);
                    }
                    cx.notify();
                }))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        let note = WHITE_MIDI[i];
                        let vel = this.engine.base_velocity;
                        this.click_note(note, vel, KeyKind::White(i), cx);
                    }),
                )
                .child(
                    div()
                        .w(px(22.))
                        .h(px(3.))
                        .rounded_full()
                        .bg(op(blue(), 0.10 + heat * 0.55)),
                )
                .child(
                    div()
                        .font_family(DISPLAY)
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(18.))
                        .text_color(ink())
                        .child(glyph),
                )
                .child(
                    div()
                        .font_family(MONO)
                        .text_size(px(7.5))
                        .text_color(faint())
                        .child(qwerty_hint(i)),
                )
                .when(held || pulse, |el| {
                    el.child(
                        div()
                            .absolute()
                            .top(px(10.))
                            .left(px(0.))
                            .w_full()
                            .h(px(40.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .w(px(30.))
                                    .h(px(30.))
                                    .rounded_full()
                                    .border_2()
                                    .border_color(op(ink(), 0.30)),
                            ),
                    )
                });

            let mut slot = div()
                .relative()
                .h_full()
                .w(DefiniteLength::Fraction(1.0 / 15.0))
                .child(white);

            if let Some(b) = BLACK_SLOTS.iter().position(|s| *s == i) {
                let glyph = self.engine.preview_black(b);
                let held = held_black[b] || self.engine.held.contains(&BLACK_MIDI[b]);
                let pulse = self.pulse.target == Some(KeyKind::Black(b))
                    && self.pulse_alive(self.pulse.seq, 210);
                slot = slot.child(
                    div()
                        .id(("black", b))
                        .absolute()
                        .top(px(0.))
                        .left(DefiniteLength::Fraction(0.70))
                        .w(DefiniteLength::Fraction(0.60))
                        .h(DefiniteLength::Fraction(0.62))
                        .rounded_b(px(5.))
                        .bg(if held || pulse { red() } else { keydark2() })
                        .border_1()
                        .border_color(rgb(0x0b0a08))
                        .shadow(vec![BoxShadow {
                            color: rgba(0x00000055).into(),
                            offset: point(px(2.), px(5.)),
                            blur_radius: px(6.),
                            spread_radius: px(0.),
                        }])
                        .flex()
                        .items_end()
                        .justify_center()
                        .pb(px(9.))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0x332e26)))
                        .on_hover(cx.listener(move |this, hovered, _, cx| {
                            if *hovered {
                                this.set_hover(
                                    Some(Hover::Black(b)),
                                    Some(format!(
                                        "{} — types {}",
                                        note_name(BLACK_MIDI[b]),
                                        this.engine.preview_black(b)
                                    )),
                                );
                            } else if this.hover_is(Hover::Black(b)) {
                                this.set_hover(None, None);
                            }
                            cx.notify();
                        }))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                let note = BLACK_MIDI[b];
                                this.click_note(note, 108, KeyKind::Black(b), cx);
                            }),
                        )
                        .child(
                            div()
                                .font_family(MONO)
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(10.))
                                .text_color(if held || pulse { paper() } else { yellow() })
                                .child(glyph),
                        ),
                );
            }

            slots = slots.child(slot);
        }

        div()
            .h(px(218.))
            .flex_shrink_0()
            .rounded(px(14.))
            .bg(keydark())
            .border_1()
            .border_color(ink())
            .shadow(vec![BoxShadow {
                color: rgba(0x17130c40).into(),
                offset: point(px(0.), px(12.)),
                blur_radius: px(22.),
                spread_radius: px(-8.),
            }])
            .p(px(6.))
            .child(
                div()
                    .w_full()
                    .h_full()
                    .relative()
                    .rounded(px(9.))
                    .overflow_hidden()
                    .child(slots),
            )
    }

    // ── pedals & controls ───────────────────────────────────────────────────

    fn deck(&self, cx: &mut Context<Self>) -> Div {
        let pedal = |this: &Self,
                     idx: usize,
                     label: &'static str,
                     icon: &'static str,
                     lit: bool,
                     cx: &mut Context<Self>| {
            let hovered = this.hover_is(Hover::Pedal(idx));
            div()
                .id(("pedal", idx))
                .w(px(58.))
                .h(px(54.))
                .rounded(px(9.))
                .border_1()
                .border_color(if lit || hovered { ink() } else { hair() })
                .bg(if lit { card_warm() } else { card() })
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(3.))
                .cursor_pointer()
                .hover(|s| s.bg(card_warm()).border_color(hair_dark()))
                .on_hover(cx.listener(move |this, hovered, _, cx| {
                    if *hovered {
                        let hint = match idx {
                            0 => "Sustain — puts a space between notes (CC64)".to_string(),
                            1 => "Soft — cycles the layer abc / ABC / 123 (CC67)".to_string(),
                            _ => "Sostenuto — deletes the last word (CC66)".to_string(),
                        };
                        this.set_hover(Some(Hover::Pedal(idx)), Some(hint));
                    } else if this.hover_is(Hover::Pedal(idx)) {
                        this.set_hover(None, None);
                    }
                    cx.notify();
                }))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        let pedal = match idx {
                            0 => Pedal::Sustain,
                            1 => Pedal::Soft,
                            _ => Pedal::Sostenuto,
                        };
                        this.engine.toggle_pedal(pedal, this.now());
                        this.seq = this.seq.wrapping_add(1);
                        this.pulse.pedal = Some(idx);
                        this.pulse.at = this.now();
                        this.pulse.seq = this.seq;
                        if idx == 2 {
                            this.set_notice("Word deleted", NoticeKind::Info);
                        }
                        cx.notify();
                    }),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_center()
                        .gap(px(5.))
                        .child(div().size(px(4.5)).rounded_full().bg(if lit {
                            green()
                        } else {
                            faint()
                        }))
                        .child(
                            div()
                                .font_family(MONO)
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(9.))
                                .text_color(if lit { ink() } else { muted() })
                                .child(label),
                        ),
                )
                .child(
                    div()
                        .font_family(if idx == 2 { MONO } else { UI })
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(if idx == 2 { 16. } else { 15. }))
                        .text_color(ink())
                        .child(icon),
                )
        };

        let levels = [48u8, 90, 118];
        let mut vel = div().flex().flex_row().items_end().gap(px(3.)).h(px(22.));
        for (i, level) in levels.iter().enumerate() {
            let v = *level;
            let active = (self.engine.base_velocity as i32 - v as i32).abs()
                < (if i == 0 { 22 } else { 18 });
            let h = 7. + i as f32 * 6.;
            vel = vel.child(
                div()
                    .id(("vel", i))
                    .w(px(9.))
                    .h(px(h))
                    .rounded(px(2.))
                    .bg(if active { ink() } else { hair_dark() })
                    .cursor_pointer()
                    .hover(|s| s.bg(yellow()))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.engine.set_base_velocity(v);
                            this.persist();
                            this.set_notice(format!("Velocity {v}"), NoticeKind::Info);
                            cx.notify();
                        }),
                    ),
            );
        }

        let gate = self.engine.timing_ms;
        let gate_frac = ((gate as f32 - 40.) / 200.).clamp(0.0, 1.0);
        let gate_btn = |delta: i32, cx: &mut Context<Self>| {
            let label = if delta < 0 { "−" } else { "+" };
            div()
                .id(if delta < 0 { "gate-dec" } else { "gate-inc" })
                .w(px(22.))
                .h(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(5.))
                .cursor_pointer()
                .hover(|s| s.bg(card_warm()))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        let next = (this.engine.timing_ms as i32 + delta * 40).clamp(40, 240);
                        this.engine.set_timing(next as u64);
                        this.persist();
                        this.set_notice(format!("Gate {} ms", next), NoticeKind::Info);
                        cx.notify();
                    }),
                )
                .child(
                    div()
                        .font_family(MONO)
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(11.))
                        .text_color(muted())
                        .child(label),
                )
        };

        let wheel_n = self.engine.candidates().len().max(1) as f32;
        let wheel_frac =
            (self.engine.candidate_index() as f32 / (wheel_n - 1.0).max(1.0)).clamp(0.0, 1.0);

        div()
            .h(px(64.))
            .flex_shrink_0()
            .rounded(px(12.))
            .bg(card())
            .border_1()
            .border_color(hair())
            .flex()
            .flex_row()
            .items_center()
            .gap(px(9.))
            .px(px(10.))
            // pedals
            .child(pedal(self, 0, "SUS", "⌣", self.engine.sustain, cx))
            .child(pedal(
                self,
                1,
                "SOFT",
                "◐",
                self.pulse.pedal == Some(1) && self.pulse_alive(self.pulse.seq, 260),
                cx,
            ))
            .child(pedal(self, 2, "DEL", "⌫", false, cx))
            .child(div().w(px(1.)).h(px(34.)).bg(hair()))
            // gate
            .child(
                div()
                    .id("gate")
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap(px(5.))
                    .px(px(6.))
                    .rounded(px(8.))
                    .cursor_pointer()
                    .hover(|s| s.bg(card_warm()))
                    .on_hover(cx.listener(|this, hovered, _, cx| {
                        if *hovered {
                            this.set_hover(
                                Some(Hover::Gate),
                                Some(format!(
                                    "Gate — notes within {} ms make a chord",
                                    this.engine.timing_ms
                                )),
                            );
                        } else if this.hover_is(Hover::Gate) {
                            this.set_hover(None, None);
                        }
                        cx.notify();
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(caps("GATE", 7.5))
                            .child(mono(7.5).text_color(faint()).child(format!("{} MS", gate))),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(4.))
                            .child(gate_btn(-1, cx))
                            .child(
                                div()
                                    .flex_1()
                                    .h(px(18.))
                                    .relative()
                                    .flex()
                                    .items_center()
                                    .child(
                                        div()
                                            .absolute()
                                            .left(px(0.))
                                            .right(px(0.))
                                            .h(px(1.))
                                            .bg(hair_dark()),
                                    )
                                    .child(
                                        div()
                                            .absolute()
                                            .left(px(2.))
                                            .h(px(6.))
                                            .w(px(6.))
                                            .rounded_full()
                                            .bg(ink()),
                                    )
                                    .child(
                                        div()
                                            .absolute()
                                            .left(DefiniteLength::Fraction(0.12 + gate_frac * 0.74))
                                            .h(px(8.))
                                            .w(px(2.))
                                            .bg(yellow())
                                            .rounded(px(1.)),
                                    )
                                    .child(
                                        div()
                                            .absolute()
                                            .right(px(2.))
                                            .h(px(6.))
                                            .w(px(6.))
                                            .rounded_full()
                                            .bg(ink()),
                                    )
                                    .child(
                                        div()
                                            .absolute()
                                            .bottom(px(0.))
                                            .left(px(6.))
                                            .right(px(6.))
                                            .h(px(1.))
                                            .bg(op(yellow(), 0.55)),
                                    ),
                            )
                            .child(gate_btn(1, cx)),
                    ),
            )
            .child(div().w(px(1.)).h(px(34.)).bg(hair()))
            // mod wheel
            .child(
                div()
                    .id("wheel")
                    .w(px(54.))
                    .h_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(4.))
                    .rounded(px(8.))
                    .cursor_pointer()
                    .hover(|s| s.bg(card_warm()))
                    .on_hover(cx.listener(|this, hovered, _, cx| {
                        if *hovered {
                            this.set_hover(
                                Some(Hover::Wheel),
                                Some(
                                    "Mod wheel — scroll completions; aftertouch accepts"
                                        .to_string(),
                                ),
                            );
                        } else if this.hover_is(Hover::Wheel) {
                            this.set_hover(None, None);
                        }
                        cx.notify();
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.engine.scrub_candidates(1);
                            cx.notify();
                        }),
                    )
                    .child(caps("MOD", 7.5))
                    .child(
                        div()
                            .relative()
                            .w(px(26.))
                            .h(px(34.))
                            .rounded(px(5.))
                            .bg(keydark2())
                            .border_1()
                            .border_color(ink())
                            .overflow_hidden()
                            .children((0..5).map(|i| {
                                div()
                                    .absolute()
                                    .top(px(4. + i as f32 * 6.))
                                    .left(px(3.))
                                    .right(px(3.))
                                    .h(px(1.))
                                    .bg(rgba(0xf6edda22))
                            }))
                            .child(
                                div()
                                    .absolute()
                                    .left(px(3.))
                                    .right(px(3.))
                                    .top(DefiniteLength::Fraction(0.08 + wheel_frac * 0.74))
                                    .h(px(3.))
                                    .rounded(px(1.))
                                    .bg(blue()),
                            ),
                    ),
            )
            .child(div().w(px(1.)).h(px(34.)).bg(hair()))
            // velocity
            .child(
                div()
                    .h_full()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap(px(6.))
                    .px(px(4.))
                    .child(caps("VEL", 7.5))
                    .child(vel),
            )
            // hint
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_end()
                    .gap(px(2.))
                    .child(caps("⌘D DEMO", 7.))
                    .child(caps("⌘C COPY · ⌘Z UNDO · TAB MAP · ⌘, MIDI", 7.).text_color(faint())),
            )
    }

    // ── status ──────────────────────────────────────────────────────────────

    fn status(&self) -> Div {
        let notice = self
            .notice
            .as_ref()
            .filter(|n| self.now().saturating_sub(n.at) < 6_000);
        let hint = self.hint.clone();
        let midi = if self.hub.demo_running {
            "SIMULATED MIDI · PLAYING".to_string()
        } else if self.demo_notes > 0 && !self.hub.status.is_live() {
            format!("SIMULATED MIDI · {} NOTES", self.demo_notes)
        } else {
            match &self.hub.status {
                MidiStatus::Connected(name) => format!("MIDI LIVE · {name}"),
                MidiStatus::Error(_) => "MIDI ERROR · click to scan".to_string(),
                MidiStatus::NoDevice => "NO MIDI · plug in, or ⌘D".to_string(),
            }
        };
        let left = hint.clone().unwrap_or(midi);
        let color = if let Some(n) = notice {
            match n.kind {
                NoticeKind::Good => green(),
                NoticeKind::Info => ink_soft(),
            }
        } else {
            muted()
        };
        div()
            .h(px(28.))
            .flex_shrink_0()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px(px(16.))
            .border_t_1()
            .border_color(hair())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        mono(8.5)
                            .text_color(if hint.is_some() { ink_soft() } else { muted() })
                            .child(left),
                    )
                    .when_some(notice, |el, n| {
                        el.child(div().w(px(1.)).h(px(10.)).bg(hair())).child(
                            mono(8.5)
                                .font_weight(FontWeight::BOLD)
                                .text_color(color)
                                .child(n.text.clone()),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(9.))
                    .child(
                        mono(8.)
                            .text_color(faint())
                            .child(self.engine.active_profile().name.to_uppercase()),
                    )
                    .child(
                        div().w(px(50.)).h(px(3.)).rounded_full().bg(hair()).child(
                            div()
                                .w(px(50.0 * (self.engine.quality() / 100.0)))
                                .h_full()
                                .rounded_full()
                                .bg(green()),
                        ),
                    )
                    .child(
                        mono(8.)
                            .text_color(faint())
                            .child(format!("{:.0}", self.engine.quality())),
                    ),
            )
    }

    // ── overlays ────────────────────────────────────────────────────────────

    fn overlay(&self, tab: OverlayTab, cx: &mut Context<Self>) -> AnyElement {
        let close = cx.listener(|this, _, _, cx| {
            this.overlay = None;
            this.learn = None;
            this.edit_mode = false;
            this.engine.cancel_edit();
            cx.notify();
        });
        let dismiss = cx.listener(|this, _, _, cx| {
            this.overlay = None;
            this.learn = None;
            this.edit_mode = false;
            this.engine.cancel_edit();
            cx.notify();
        });

        let tab_button = |label: &'static str, target: OverlayTab, cx: &mut Context<Self>| {
            let on = tab == target;
            div()
                .px(px(12.))
                .py(px(4.))
                .rounded(px(6.))
                .bg(if on { card() } else { side() })
                .text_size(px(10.))
                .font_family(MONO)
                .font_weight(FontWeight::BOLD)
                .text_color(if on { ink() } else { muted() })
                .cursor_pointer()
                .hover(|s| s.text_color(ink()))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.overlay = Some(target);
                        this.learn = None;
                        cx.notify();
                    }),
                )
                .child(label)
        };

        let tabs = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(3.))
            .p(px(2.))
            .rounded(px(8.))
            .bg(side())
            .child(tab_button("MAP", OverlayTab::Map, cx))
            .child(tab_button("GESTURES", OverlayTab::Gestures, cx))
            .child(tab_button("MIDI", OverlayTab::Learn, cx));

        let header = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(14.))
                    .child(
                        div()
                            .font_family(DISPLAY)
                            .font_weight(FontWeight::EXTRA_BOLD)
                            .text_size(px(18.))
                            .text_color(ink())
                            .child("INSTRUMENT"),
                    )
                    .child(tabs),
            )
            .child(
                div()
                    .size(px(28.))
                    .rounded_full()
                    .border_1()
                    .border_color(hair())
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| s.bg(card_warm()))
                    .on_mouse_down(MouseButton::Left, close)
                    .child(
                        div()
                            .font_family(MONO)
                            .font_weight(FontWeight::BOLD)
                            .text_size(px(13.))
                            .text_color(ink())
                            .child("×"),
                    ),
            );

        let body = match tab {
            OverlayTab::Map => self.map_body(cx),
            OverlayTab::Gestures => self.gestures_body(),
            OverlayTab::Learn => self.learn_body(cx),
        };

        let card = div()
            .w(px(660.))
            .max_h(px(600.))
            .rounded(px(16.))
            .bg(card())
            .border_1()
            .border_color(ink())
            .shadow(float_shadow())
            .p(px(22.))
            .flex()
            .flex_col()
            .gap(px(16.))
            .occlude()
            .child(header)
            .child(body);

        div()
            .absolute()
            .top(px(0.))
            .left(px(0.))
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .absolute()
                    .top(px(0.))
                    .left(px(0.))
                    .size_full()
                    .bg(rgba(0x17130caa))
                    .on_mouse_down(MouseButton::Left, dismiss),
            )
            .child(card)
            .into_any_element()
    }

    fn map_body(&self, cx: &mut Context<Self>) -> Div {
        let profile = self.engine.active_profile();
        let editing = self.engine.edit_mode;
        let mut chips = div().flex().flex_row().flex_wrap().gap(px(5.));
        for (i, p) in self.engine.profiles.iter().enumerate() {
            let on = i == self.engine.profile_index;
            chips = chips.child(
                div()
                    .id(("prof", i))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.))
                    .px(px(10.))
                    .py(px(5.))
                    .rounded_full()
                    .border_1()
                    .border_color(if on { ink() } else { hair() })
                    .bg(if on { yellow() } else { card() })
                    .cursor_pointer()
                    .hover(|s| s.bg(card_warm()))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.engine.set_profile(i);
                            this.edit_mode = false;
                            cx.notify();
                        }),
                    )
                    .child(div().size(px(5.)).rounded_full().bg(if p.custom {
                        blue()
                    } else {
                        green()
                    }))
                    .child(
                        div()
                            .font_family(UI)
                            .font_weight(FontWeight::MEDIUM)
                            .text_size(px(10.5))
                            .text_color(ink())
                            .child(p.name.clone()),
                    ),
            );
        }

        let mut strip = div()
            .flex()
            .flex_row()
            .items_end()
            .gap(px(3.))
            .h(px(148.))
            .w_full();
        for i in 0..15 {
            let g = self.engine.preview_white(i);
            let heat = self.engine.white_heat(i);
            let effort = self.engine.white_effort(i);
            let picked = self.engine.swap_pick == Some(i);
            let hover = self.hover_is(Hover::MapCol(i));
            strip = strip.child(
                div()
                    .id(("mapcol", i))
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_end()
                    .gap(px(5.))
                    .pb(px(7.))
                    .rounded(px(7.))
                    .border_1()
                    .border_color(if picked {
                        op(blue(), 1.0)
                    } else if hover {
                        op(hair_dark(), 1.0)
                    } else {
                        op(hair(), 0.0)
                    })
                    .bg(if picked {
                        op(blue(), 0.10)
                    } else {
                        op(card(), 1.0)
                    })
                    .cursor_pointer()
                    .hover(|s| s.bg(card_warm()))
                    .on_hover(cx.listener(move |this, hovered, _, cx| {
                        if *hovered {
                            this.set_hover(
                                Some(Hover::MapCol(i)),
                                Some(format!(
                                    "{}  ·  {:.1}% of English  ·  effort {:.0}",
                                    this.engine.preview_white(i),
                                    freq_for(&this.engine.preview_white(i)),
                                    this.engine.white_effort(i)
                                )),
                            );
                        } else if this.hover_is(Hover::MapCol(i)) {
                            this.set_hover(None, None);
                        }
                        cx.notify();
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            if this.engine.edit_mode && this.engine.pick_swap(i) {
                                this.persist();
                            }
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .font_family(DISPLAY)
                            .font_weight(FontWeight::BOLD)
                            .text_size(px(13.))
                            .text_color(ink())
                            .child(g.clone()),
                    )
                    .child(
                        div()
                            .relative()
                            .w(px(12.))
                            .h(px(70.))
                            .rounded(px(2.))
                            .bg(side())
                            .flex()
                            .flex_col()
                            .justify_end()
                            .child(
                                div()
                                    .w_full()
                                    .h(DefiniteLength::Fraction(heat.max(0.06)))
                                    .rounded(px(2.))
                                    .bg(op(blue(), 0.35 + heat * 0.65)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(2.))
                            .children((0..3).map(|d| {
                                div()
                                    .w(px(3.))
                                    .h(px(3.))
                                    .rounded_full()
                                    .bg(if (d as f32) < effort { muted() } else { hair() })
                            })),
                    ),
            );
        }

        let quality = self.engine.quality();
        div()
            .flex()
            .flex_col()
            .gap(px(14.))
            .child(chips)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .font_family(SERIF)
                            .italic()
                            .text_size(px(14.))
                            .text_color(muted())
                            .child("cheapest gestures first — frequency over effort"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(7.))
                            .child(mono(8.5).text_color(faint()).child(if editing {
                                "PICK TWO KEYS"
                            } else {
                                "SCORE"
                            }))
                            .when(editing, |el| {
                                el.child(
                                    div()
                                        .px(px(7.))
                                        .py(px(2.))
                                        .rounded(px(4.))
                                        .bg(blue())
                                        .child(
                                            mono(8.)
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(card())
                                                .child("REMAP"),
                                        ),
                                )
                            }),
                    ),
            )
            .child(strip)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(10.))
                    .child(
                        div().flex_1().h(px(5.)).rounded_full().bg(side()).child(
                            div()
                                .h_full()
                                .rounded_full()
                                .bg(if quality > 92.0 { green() } else { yellow() })
                                .w(DefiniteLength::Fraction(quality / 100.0)),
                        ),
                    )
                    .child(
                        mono(9.)
                            .font_weight(FontWeight::BOLD)
                            .text_color(ink())
                            .child(format!("{quality:.0}%")),
                    )
                    .child(div().w(px(1.)).h(px(20.)).bg(hair()))
                    .child(
                        div()
                            .h(px(28.))
                            .px(px(12.))
                            .rounded(px(6.))
                            .border_1()
                            .border_color(hair())
                            .bg(card())
                            .flex()
                            .items_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(card_warm()))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.engine.duplicate_as_custom();
                                    this.persist();
                                    this.edit_mode = true;
                                    cx.notify();
                                }),
                            )
                            .child(
                                mono(9.)
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(ink())
                                    .child("DUPLICATE"),
                            ),
                    )
                    .child(
                        div()
                            .h(px(28.))
                            .px(px(12.))
                            .rounded(px(6.))
                            .bg(if editing { ink() } else { card() })
                            .border_1()
                            .border_color(if editing { ink() } else { hair() })
                            .flex()
                            .items_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(if editing { ink() } else { card_warm() }))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    if this.engine.edit_mode {
                                        this.engine.cancel_edit();
                                        this.edit_mode = false;
                                    } else {
                                        this.engine.begin_edit();
                                        this.edit_mode = true;
                                    }
                                    cx.notify();
                                }),
                            )
                            .child(
                                mono(9.)
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if editing { yellow() } else { ink() })
                                    .child(if editing { "DONE" } else { "REMAP" }),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.))
                    .child(caps("PROFILE", 7.5))
                    .child(
                        mono(8.5)
                            .text_color(faint())
                            .child(profile.name.to_uppercase()),
                    ),
            )
    }

    fn learn_body(&self, cx: &mut Context<Self>) -> Div {
        let mut rows = div().flex().flex_col().gap(px(6.));
        for control in Control::ALL {
            let cc = control.get(&self.engine.cc_map);
            let listening = self.learn == Some(control);
            rows = rows.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(10.))
                    .px(px(10.))
                    .py(px(7.))
                    .rounded(px(8.))
                    .bg(if listening { card_warm() } else { side() })
                    .border_1()
                    .border_color(if listening { ink() } else { hair() })
                    .child(
                        div().w(px(84.)).child(
                            mono(8.5)
                                .font_weight(FontWeight::BOLD)
                                .text_color(ink())
                                .child(control.label()),
                        ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(11.))
                            .text_color(muted())
                            .child(control.hint()),
                    )
                    .child(
                        div()
                            .w(px(54.))
                            .py(px(2.))
                            .rounded(px(4.))
                            .bg(if listening { yellow() } else { card() })
                            .border_1()
                            .border_color(hair())
                            .flex()
                            .justify_center()
                            .child(
                                mono(8.5)
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(ink())
                                    .child(format!("CC {cc}")),
                            ),
                    )
                    .child(
                        div()
                            .id(("learn", control as usize))
                            .w(px(82.))
                            .h(px(26.))
                            .rounded(px(5.))
                            .bg(if listening { ink() } else { card() })
                            .border_1()
                            .border_color(if listening { ink() } else { hair() })
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(if listening { ink() } else { card_warm() }))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    if this.learn == Some(control) {
                                        this.learn = None;
                                        this.set_notice("Learning cancelled", NoticeKind::Info);
                                    } else {
                                        this.learn = Some(control);
                                        this.set_notice(
                                            format!("Move a pedal or knob for {}", control.label()),
                                            NoticeKind::Info,
                                        );
                                    }
                                    cx.notify();
                                }),
                            )
                            .child(
                                mono(8.5)
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if listening { yellow() } else { ink() })
                                    .child(if listening { "LISTENING…" } else { "LEARN" }),
                            ),
                    ),
            );
        }

        let reset = div()
            .h(px(28.))
            .px(px(12.))
            .rounded(px(6.))
            .border_1()
            .border_color(hair())
            .bg(card())
            .flex()
            .items_center()
            .cursor_pointer()
            .hover(|s| s.bg(card_warm()))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.engine.cc_map = Default::default();
                    this.learn = None;
                    this.persist();
                    this.set_notice("MIDI controls reset", NoticeKind::Info);
                    cx.notify();
                }),
            )
            .child(
                mono(8.5)
                    .font_weight(FontWeight::BOLD)
                    .text_color(ink())
                    .child("RESET TO DEFAULTS"),
            );

        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_end()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(3.))
                            .child(caps("MIDI LEARN", 8.))
                            .child(
                                div()
                                    .font_family(SERIF)
                                    .italic()
                                    .text_size(px(17.))
                                    .text_color(ink())
                                    .child("teach it your pedals"),
                            ),
                    )
                    .child(
                        mono(8.)
                            .text_color(faint())
                            .child("LEARNING A USED NUMBER SWAPS THE TWO CONTROLS"),
                    ),
            )
            .child(rows)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(12.))
                    .pt(px(4.))
                    .border_t_1()
                    .border_color(hair())
                    .child(
                        mono(8.)
                            .text_color(faint())
                            .child("AFTERTOUCH ACCEPTS A COMPLETION · PITCH BEND SCRUBS"),
                    )
                    .child(div().flex_1())
                    .child(reset),
            )
    }

    fn gestures_body(&self) -> Div {
        let mut chords = div().flex().flex_col().gap(px(7.));
        for (i, pad) in chord_pads().iter().enumerate() {
            chords = chords.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(10.))
                    .child(triad_diagram(i, false))
                    .child(
                        div()
                            .font_family(UI)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_size(px(12.))
                            .text_color(ink())
                            .child(pad.word),
                    )
                    .child(div().flex_1())
                    .child(mono(8.).text_color(faint()).child(format!(
                        "{} {} {}",
                        note_name(pad.notes[0]),
                        note_name(pad.notes[1]),
                        note_name(pad.notes[2])
                    ))),
            );
        }

        let mut intervals = div().flex().flex_col().gap(px(9.));
        for (semi, text) in crate::engine::INTERVALS {
            let dots = (0..semi.min(9))
                .map(|_| div().w(px(2.)).h(px(2.)).rounded_full().bg(op(ink(), 0.45)));
            intervals = intervals.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .w(px(96.))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(2.))
                            .child(div().size(px(7.)).rounded_full().bg(ink()))
                            .children(dots)
                            .child(div().size(px(7.)).rounded_full().bg(ink())),
                    )
                    .child(
                        div()
                            .font_family(UI)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_size(px(12.))
                            .text_color(ink())
                            .child(text),
                    )
                    .child(div().flex_1())
                    .child(
                        mono(8.)
                            .text_color(faint())
                            .child(format!("{semi} SEMITONES")),
                    ),
            );
        }

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(26.))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(caps("WORD CHORDS", 8.))
                            .child(chords),
                    )
                    .child(div().w(px(1.)).bg(hair()))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(caps("INTERVAL BIGRAMS", 8.))
                            .child(intervals),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(16.))
                    .pt(px(10.))
                    .border_t_1()
                    .border_color(hair())
                    .child(
                        mono(8.)
                            .text_color(faint())
                            .child("SUSTAIN = SPACE · SOFT = LAYER · SOSTENUTO = DELETE WORD"),
                    )
                    .child(div().flex_1())
                    .child(
                        mono(8.)
                            .text_color(faint())
                            .child("WHEEL = COMPLETE · AFTERTOUCH = ACCEPT"),
                    ),
            )
    }
}

fn qwerty_hint(i: usize) -> &'static str {
    const HINTS: [&str; 15] = [
        "A", "S", "D", "F", "G", "H", "J", "K", "L", ";", "'", "1", "2", "3", "4",
    ];
    HINTS[i.min(14)]
}

fn triad_diagram(index: usize, active: bool) -> Div {
    let pad = &chord_pads()[index.min(5)];
    let color = if active { yellow() } else { ink() };
    let heights = [11.0, 16.0, 13.0];
    // Each bar is the physical key height; the root is marked with a dot.
    div()
        .w(px(30.))
        .h(px(30.))
        .flex()
        .flex_row()
        .items_end()
        .justify_center()
        .gap(px(3.))
        .children((0..3).map(|i| {
            div()
                .w(px(5.))
                .h(px(heights[i]))
                .rounded(px(1.5))
                .bg(color)
                .when(i == 0, |el| {
                    el.child(div().size(px(2.)).rounded_full().bg(card()))
                })
        }))
        .child(
            div()
                .absolute()
                .font_family(MONO)
                .text_size(px(6.5))
                .text_color(faint())
                .child(pad.root),
        )
}

// ─────────────────────────────────── render ────────────────────────────────

impl Focusable for KeyChangeApp {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for KeyChangeApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Expire the short-lived notice without a timer.
        if let Some(notice) = &self.notice {
            if self.now().saturating_sub(notice.at) > 6_500 {
                self.notice = None;
            }
        }
        let overlay = self.overlay;
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(paper())
            .text_color(ink())
            .font_family(UI)
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| {
                if !ev.is_held {
                    this.on_keystroke(&ev.keystroke, cx);
                }
            }))
            .on_key_up(cx.listener(|this, ev: &KeyUpEvent, _, cx| {
                this.engine.key_qwerty_release(&ev.keystroke.key);
                cx.notify();
            }))
            .child(self.toolbar(cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.))
                    .gap(px(9.))
                    .px(px(12.))
                    .pt(px(10.))
                    .pb(px(9.))
                    .child(self.stage(cx))
                    .child(self.pads(cx))
                    .child(self.keyboard(cx))
                    .child(self.deck(cx)),
            )
            .child(self.status())
            .when_some(overlay, |el, tab| el.child(self.overlay(tab, cx)))
    }
}

// Keep the compiler honest about the constants the engine exports; these are
// used by the map overlay but referenced here for clarity.
#[allow(dead_code)]
const _DEFAULT_LETTERS: [&str; 15] = DEFAULT_LETTERS;
