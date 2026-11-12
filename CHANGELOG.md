# Changelog

## 0.5.0 — 2026-11-12

The instrument learns you back.

**New**
- **The gesture sheet plays.** Click any miniature keyboard or dot ruler and
  the chord or bigram runs through the real engine, lighting the keys you would
  have pressed. The fastest way to learn a mapping.
- **Practice mode** (`⌘P`). A random word or bigram appears in the score header;
  play it to build a streak. Misses count, flash the target keys, and the best
  streak stays for the session. Stop from the header or with `⌘P`.
- **A gate you can touch.** Click or drag the 40–240 ms track, or tap **TAP**
  three times to derive the window from your own hands.
- **MIDI input picker** in `⌘,`: AUTO plus every connected port, remembered by
  name. Hot-plug is polled every couple of seconds, with a toast when a
  keyboard arrives or a cable is pulled.
- **Homebrew cask**: `brew install --cask Abhinav-Prabhakar/tap/keychange`.
- **Built-in updater**: `⌘U` checks GitHub Releases, puts a version badge in the
  status bar, downloads the DMG, verifies the published SHA-256, installs and
  relaunches. If the app bundle is not writable, it opens the DMG instead of
  touching anything.

**Changed**
- The status bar carries the update badge and the learned MIDI device.
- The gesture footer now names click-to-play, practice and update.
- Settings remember the pinned MIDI input.

## 0.4.1 — 2026-11-12

The gesture sheet, redrawn.

**Changed**
- Every word chord in the Gestures tab now carries a miniature keyboard with its
  three keys lit, and every bigram gets a dot ruler: the slope shows the
  direction, the number of dots shows the interval. The old note-name and
  semitone labels are gone.
- The sheet is arranged in aligned banks — six main pads, six low, twelve sharp
  and minor, then up, down and high intervals — with a rule and a count for each
  family.
- The deck hint and the status bar now share the same right edge.
- The gesture, map, MIDI, empty and performance screenshots were refreshed.

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
