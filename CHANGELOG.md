# Changelog

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
