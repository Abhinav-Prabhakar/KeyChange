# KeyChange for the desktop

A native MIDI text instrument in one window. Plug in a piano, play a letter, and
watch it land in a live score. Loud notes come out **bold**, soft notes come out
*italic*, chords fire whole words, and the staff snaps sloppy hands into perfect
spelling.

Built in Rust with [GPUI](https://gpui.rs) (the renderer behind Zed) and
[`midir`](https://github.com/Boddlnagg/midir) for MIDI I/O. No browser, no build
step, no account.

<img src="assets/screenshot.png" alt="The KeyChange desktop instrument after a simulated performance — the transcript reads “the and you that have with the hello”, with a staff of notes and the NOW tile showing the current word" width="100%">

## Run it

```bash
cd app
cargo run
```

Press **⌘D** (or click **DEMO**) to hear the instrument play itself. The
simulated player emits real MIDI bytes through the same parser as hardware, so
you can evaluate everything without a piano plugged in. **KC_DEMO=1 cargo run**
starts the simulation on launch.

macOS 14+ is the current target; GPUI builds on Linux and Windows too but only
macOS has been exercised here.

## The whole instrument, one screen

Everything is always visible. Nothing is a tab, a modal, or a settings page —
the two overlays (**Tab** for the map, **/** for gestures) are reference cards,
not required steps.

| Region | What it does |
|---|---|
| **Toolbar** | Layer switch (`abc / ABC / 123`), live CC lights, profile picker, MIDI status, demo, shortcuts. |
| **Score** | The transcript, a 20-note staff, and the right rail: NOW (note / interval / chord), WPM and a wheel history. Hover any key or pad to reveal numbers in the rail. |
| **Chord pads** | Six triads — `the and you that have with` — each showing its three keys and its share of English. Click or play them. |
| **Keyboard** | Fifteen white keys (Huffman letters) and ten black keys (punctuation), with heat strips, live held-note rings and per-key velocity. |
| **Deck** | Sustain / soft / sostenuto pedals, a 40–240 ms gate slider, mod-wheel scrubber and base velocity. |
| **Status bar** | MIDI state (`MIDI LIVE`, `SIMULATED MIDI · n NOTES`, `NO MIDI`), notice line, profile quality. |

## Playing it

### On a MIDI keyboard

| Gesture | Result |
|---|---|
| One key | Letter (or punctuation on a black key) |
| Key in another octave | Same letter — octaves are free |
| Three notes (a pad triad) | Whole word: `the`, `and`, `you`, `that`, `have`, `with` |
| Two notes, 1–9 semitones | Bigram: `th`, `he`, `in`, `er`, `an`, `ing`, `ed` |
| Two notes, an octave apart | Repeat the last completed word |
| Sustain (CC64) | Word gaps — spaces follow legato |
| Soft (CC67) | Cycle to the next layer |
| Sostenuto (CC66) | Delete the last word (an undo pedal) |
| Mod wheel (CC1) | Scrub autocomplete candidates |
| Aftertouch | Accept the selected candidate |
| Expression (CC11) | Set the chord gate, 40–240 ms |

Chord detection is purely time-based: notes inside the gate window (default
120 ms) form a gesture, and separate gestures type separate letters. That makes
the instrument forgiving of rolled chords without ever swallowing a fast run.

### On a computer keyboard

| Keys | Action |
|---|---|
| `A S D F G H J K L ; ' 1 2 3 4` | The fifteen white keys |
| `W E T Y U O P [ ] \` | The ten black keys |
| `Space` | Commit a space (sustain-style) |
| `Enter` | Commit the trailing word |
| `↑ / ↓` | Scrub autocomplete candidates |
| `← / →` or `[ / ]` | Previous / next layer |
| `⌫` | Delete a character |
| `Esc` | Close the overlay or cancel remapping |
| `Tab` | Toggle the map overlay |
| `/` | Toggle the gesture reference |
| `⌘D` | Start / stop the simulated performance |
| `⌘C` | Copy the transcript |
| `⌘Z` | Undo the last edit |
| `⌘K` | Clear the score |
| `⌘Q` | Quit |

Clicking a key plays it — click high for a soft note, low for a hard one. Pad
and pedal clicks are the same messages the hardware sends.

## Profiles

Four layouts ship: **Default** (Huffman-optimal English), **Chromatic**, **Jazz**
and **Coder**. Open **Tab** to see the letter-frequency map with each key's
effort, and a live quality score. **Duplicate** clones the active layout into a
custom profile; **Remap** swaps two white keys by clicking their columns. Score
is `Σ frequency × effort`, so layouts can be compared against the optimum.

Custom profiles are in-memory only — persistence and a per-key character picker
are the next step.

## How it is built

```text
app/
├── src/
│   ├── main.rs      Window, bundled fonts, focus, KC_DEMO.
│   ├── engine.rs    Pure state machine: notes, gestures, layers, pedals,
│   │                candidates, WPM, profiles, undo. No GPUI, no threads.
│   ├── midi.rs      Raw byte parser, MidiHub with reconnecting hardware input,
│   │                and the simulated player. Hardware and demo share a path.
│   └── ui.rs        The single-screen GPUI view, overlays and all interactions.
├── assets/fonts/    Unbounded, Space Grotesk, JetBrains Mono, Instrument Serif
│                    (OFL, licences included).
└── Cargo.toml
```

The engine never knows where a note came from. `midi::parse` turns raw bytes
into `MidiMsg`s, `Engine` turns messages into text, and `ui.rs` only draws and
forwards input. That split is what makes an interaction testable without a
window.

Fonts are bundled so the instrument looks the same on a fresh machine; there is
no font dependency on the OS.

## Tests

```bash
cargo test
```

46 tests cover the parts that are easy to break: velocity buckets, layer
precision, chord timing, interval and octave gestures, sustain gaps, quantize
and candidate acceptance, WPM, undo/delete, profile scoring and remapping, MIDI
byte parsing, and the demo script. The UI is verified by screenshot, not by
unit test.

## Known rough edges

- Custom profiles do not persist between launches.
- MIDI Learn is not implemented; the map is fixed per profile.
- The simulation is a fixed script, not a generative player.
- Linux and Windows builds are untested.

---

<p align="center"><sub>The design system behind this screen lives in <a href="../design.md">design.md</a>.<br>
The playable landing page lives at <a href="https://abhinav-prabhakar.github.io/KeyChange/">abhinav-prabhakar.github.io/KeyChange</a>.</sub></p>
