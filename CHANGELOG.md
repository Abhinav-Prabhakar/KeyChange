# Changelog

## 0.4.0 — 2026-11-12

More words, more bigrams, same keyboard.

**Added**
- Twelve more word chords on sharp and minor triads:
  `this but from they will one about would there what when like`.
- A low word bank: the same six triad shapes below C3 type
  `of to in it for not`, so a 49-key controller covers twelve words.
- Directional bigrams: the seven core intervals now type a second word when the
  second note is lower — `to of is it or on at`.
- Eleven high-octave bigrams above C5 — `re en nd ti es te al ar st nt ng` —
  which also unlocks the intervals the main table skips.
- The gesture sheet is grouped into banks: main chords, low octave, extra
  triads, up intervals, down intervals, high octave.
- The status line reports the full count (`24 WORDS · 25 BIGRAMS`) and the deck
  hint points at the low and high banks.
- The simulated performance plays all three word banks.
- 63 tests, including uniqueness across every chord and coverage of every bank.

**Changed**
- Chord detection now identifies low-bank words by their lowest note, which lets
  the F, G and A shapes cross C3 without being mistaken for the main bank.

## 0.3.1 — 2026-11-12

The small things that make it feel finished.

**Added**
- Smart punctuation spacing: `hello ,` becomes `hello,` the moment the key lands.
- Live session stats in the status bar: words, characters and corrections.
- A live MIDI monitor in the MIDI hover: note, CC, aftertouch or bend, decoded.
- Per-control MIDI reset (↺) beside each row in MIDI Learn.
- A `?` shortcut for the gesture sheet, with a full shortcut row in the card.
- A one-line first-run tip for new installs.

**Changed**
- `⌘K` clearing the score is now undoable with `⌘Z`.
- The landing page download button points at GitHub Releases and labels itself
  for the visitor's operating system.
- Release downloads are documented as macOS 14+ on Apple Silicon, 3.4 MB.

## 0.3.0 — 2026-11-12

The first installable build.

**Added**
- `KeyChange.app` bundle with a proper icon, and a 3.4 MB drag-to-Applications DMG.
- Saved layouts, active profile, layer, velocity, gate and learned CCs in
  `~/Library/Application Support/KeyChange/settings.json`.
- MIDI Learn (**⌘,**): teach the app any sustain pedal, expression knob or CC,
  with automatic swaps when a number is already in use.
- Corrupt settings are moved aside instead of blocking launch.
- 54 tests, including CC routing, learning swaps and settings round-trips.
- MIT license.

**Changed**
- Layers, velocity and gate now survive a restart.
- The deck hint and MIDI overlay document the new shortcuts.

## 0.2.0 — 2026-11-11

The native instrument.

**Added**
- Rust + GPUI single-screen app: transcript, staff, chord pads, 15-key piano,
  control deck, Huffman map and gesture reference.
- Real MIDI input through `midir`, raw byte parser, reconnecting device hub.
- Simulated MIDI performance (**⌘D**) through the same parser as hardware.
- QWERTY fallback, layers, velocity emphasis, sustain gaps, quadratize,
  autocomplete, octave repeat, interval bigrams, WPM and profiles.
