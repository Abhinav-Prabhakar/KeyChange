# KeyChange Design System

**Status:** living document · **Version:** 1.0 · **Last verified:** 2026-11-11 · **Site commit:** `4ec1da6`

This is the source of truth for the KeyChange landing page and anything added to this
repository next (including the native-app marketing pages and the product UI shell).
Read it before designing or building a new section, component, state or asset.

The key words **MUST**, **SHOULD** and **MAY** are used as in RFC 2119: MUST is a
requirement, SHOULD is a strong default you may break with a reason, MAY is optional.

---

## 0. How to use this document

1. **Reuse before inventing.** Almost every surface, button, label and card already has
   a pattern here. Find it and use its classes.
2. **Tokens over literals.** Colours, shadows, borders and radii come from `:root` in
   `style.css`. Do not hard-code a new hex unless it is a documented one-off (canvas
   drawing, generated confetti, etc.).
3. **Check the invariants.** The loader is one line. The navbar has no dancing bars. Text
   is never ink-on-ink. The page must work at 320 px. Audio never blocks the hero.
4. **Update this file** when you add a token, component, breakpoint, audio behaviour or
   content pattern. A component that is not documented here is not finished.

---

## 1. Product snapshot

**KeyChange** is a MIDI text instrument: plug in a piano, play a letter, and it types.
Velocity becomes emphasis, chords become shortcuts, the sustain pedal holds spaces, and a
staff view turns notes into sentences. The promise is **“Play your words loud.”**

- **Current artefact:** a single hand-built landing page that is also a working toy.
- **Stack:** semantic HTML, one hand-written CSS file, one vanilla JS file, Canvas 2D,
  Web Audio API, Web MIDI API, Google Fonts. No framework, no build step, no trackers, no
  cookies, no server code.
- **Hosting:** GitHub Pages from `main` at
  `https://abhinav-prabhakar.github.io/KeyChange/` (path is lowercase).
- **Product status:** the native app (macOS 14+ / Windows 11) is not built yet. The
  download CTA is a demo interaction, not a real download.
- **Audience:** pianists who type, producers, live coders, writers, accessibility users.

**Repo shape**

```
index.html            the whole page
style.css             tokens + every component
app.js                loader, piano, audio, canvas toys, MIDI, reveal
assets/                screenshots, OG card, audio samples
favicon* / manifest / robots / sitemap / .nojekyll
README.md             public documentation
design.md             this file
```

---

## 2. Brand

### 2.1 Concept

A **risograph gig-zine printed on cream paper**: ink-black type, fluorescent spot colours,
hard offset shadows, stickers, ticket stubs, vinyl, stompboxes and a stage light. It should
feel like a poster someone screen-printed for a show, not like a SaaS landing page.

The page is staged as a **setlist**: Side A is type, Side B is play. Sections read like
tracks; the finale is an encore ticket. Decoration is physical (paper, tape, ink, grain),
not glassy or gradient-heavy.

**Reference adjectives:** loud, inky, tactile, warm, witty, hand-set, analogue, confident.
**Anti-references:** soft gradients, glassmorphism, purple SaaS gradients, floating 3D
blobs, generic icon grids, muted “trusted by” grey.

### 2.2 Design principles

1. **The instrument is the hero.** The playable piano appears above the fold. Never replace
   it with an illustration or a screenshot.
2. **Print, not screen.** Hard 3–4 px borders, offset shadows, rotated paper, halftone
   dots, grain, duotone spot colour. Avoid blurred shadows and translucency.
3. **One loud idea per section.** A kicker, one display headline, one interactive object.
   If a section needs three ideas, it is two sections.
4. **Show, then tell.** Every claim has a toy next to it: a piano, a roll, a pedal, a chord,
   a staff, a cable. Copy supports the object.
5. **Legible first, loud second.** Fluorescent colour is a background for ink, never small
   white text on red. Contrast is non-negotiable (see §10).
6. **Fast and quiet underneath.** No trackers, no blocking audio, no layout shift. The
   fireworks are CSS and Canvas, not dependencies.

### 2.3 Voice & copy

Write like a stage announcement with a wink: short declaratives, instrument metaphors,
mono labels in caps, one music pun per screen at most.

**Do**

- Lead with verbs and concrete nouns: “Plug your piano.” “Type essays with glissandos.”
- Use the mapping formula **X → Y** for features: “Velocity → Bold”, “Sustain → Space”.
- Keep labels in `UPPERCASE MONO` with `·` or `—` separators.
- Pair a claim with a playful proof: “Sloppy hands, perfect spelling.”
- Use the house phrases sparingly: *Play your words loud · Type with your piano · Free
  forever · No account · No clack · 88 keys · One standing ovation.*

**Don’t**

- No corporate hedging (“seamless”, “revolutionary”, “empower”, “unlock”).
- No exclamation stacking; one `!` per section at most.
- No jargon without the musical payoff.
- No emoji in navigation, headings, metadata or the product UI. Emoji are allowed only as
  sticker-style icons inside feature cards, where they already exist.
- Do not rename product concepts: it is **velocity**, **sustain**, **quantize**, **staff**,
  **chords**, **presets**, **encore** — not “pressure”, “hold”, “autocorrect” alone.

### 2.4 Naming & casing

| Context | Form |
|---|---|
| Display / logo / badges | `KEYCHANGE` (all caps) |
| Body copy, metadata, README | `KeyChange` |
| Version | `v2.4`, mono superscript in the navbar |
| Tagline | “Play your words loud.” |
| Page title | `KeyChange — Type With Your Piano` |
| Sample credit | `SALAMANDER SAMPLES · CC BY 3.0` |

---

## 3. Foundations

### 3.1 Colour

All colours are defined once in `:root` in `style.css`.

| Token | Hex | Role |
|---|---|---|
| `--paper` | `#f6edda` | Page paper, light text on ink |
| `--paper2` | `#eee0bf` | **Unused** — reserved for a second paper tone |
| `--card` | `#fff8e8` | Raised paper cards, tickets, console |
| `--ink` | `#17130c` | Text, borders, dark panels, shadows |
| `--red` | `#ff2e1f` | Primary spot colour, emphasis blocks, LEDs |
| `--red-deep` | `#c81a0f` | Small text on light paper (kicker) |
| `--blue` | `#2233ff` | Secondary spot colour, alternate cards |
| `--yellow` | `#ffc700` | Highlight, active states, accents |
| `--green` | `#0e7a4f` | Success, MIDI live, free/confirmed states |
| `--line` | `var(--ink)` | **Unused** alias; use `--ink` |
| `--shadow` | `6px 6px 0 var(--ink)` | Large hard shadow |
| `--shadow-sm` | `4px 4px 0 var(--ink)` | Small hard shadow |

**Colour rules**

- **Ink on paper/yellow is the default.** Never ink on ink.
- **Yellow is a background, not a text colour** on light paper. Yellow text is allowed only
  on ink (e.g. footer credit, mono labels in dark panels).
- **Red and blue are backgrounds for large type and blocks.** Small copy on red uses ink.
  White on red is reserved for display sizes (≥ ~24 px bold), never for body or mono labels.
- **Green is a state colour**, not a decorative fill. Pair it with a word (LIVE, FREE) so
  colour is never the only signal.
- **Dark panels invert the paper**: ink background, paper text, yellow accents, paper cards.
- Spot colours never gradient into each other; gradients are only subtle paper/halftone
  textures and the piano-key whites.

**Measured contrast (WCAG 2.2)**

| Pair | Ratio | Verdict |
|---|---|---|
| ink `#17130c` on paper `#f6edda` | 15.9 | AAA, default body |
| ink on yellow `#ffc700` | 11.8 | AAA |
| ink on red `#ff2e1f` | 5.0 | AA normal text |
| paper on ink | 15.9 | AAA |
| white on blue `#2233ff` | 7.0 | AAA |
| white on green `#0e7a4f` | 5.4 | AA normal text |
| white on red | 3.7 | **large text only** |
| ink on green | 3.5 | **large text only** |
| `--red-deep` on card | 5.5 | AA normal text |

### 3.2 Surfaces & depth

- **Paper page:** `--paper` with a 1.2 px ink dot grid at 18 px, plus a faint blue ruled
  line every 46 px (`body::before`).
- **Light card:** `--card`, 3–4 px ink border, hard offset shadow, optional −2°…+2° rotation.
- **Dark stage:** `--ink` with a paper halftone dot grid at 18 px; full-bleed between
  sections with 4 px ink borders top and bottom.
- **Inside a dark stage,** cards flip back to paper. Never place ink text directly on the
  dark panel.
- **Shadows are hard offsets**, no blur: 4 px (small), 6 px (standard), 8–12 px (hero,
  encore, staff). Coloured shadows (red/blue/yellow) are used to separate a card from a
  same-colour background.
- **Tape:** a 92×26 px semi-transparent yellow/blue rectangle, 2 px ink border, rotated
  ±8°, pinned over the hero transcript and mixer channels.

### 3.3 Spacing, widths, radii

- **Spacing rhythm:** 6, 8, 10, 12, 14, 16, 18, 20, 22, 26, 28, 30, 40, 70–120 px.
  Section padding is `90px 22px` (`.panel`) or `70px 18px` (`.kc-section`); hero top
  padding is 120 px desktop / 92 px under 700 px to clear the fixed navbar.
- **Content widths:** 1220 px (standard panels), 1140 px (`.kc-section`, encore),
  1100 px (manifesto), 1060 px (piano stage), 820 px (FAQ), 640 px (lede).
- **Radii:** 0 for print blocks and the piano; 6 px for keys; 12–14 px for standard controls
  and buttons; 20 px only for the legacy giant button; 99 px for pills.
- **Borders:** 2 px for small UI/mono chips, 3 px for buttons and labels, 4 px for hero
  blocks and cards, 5–10 px only for the OG card frame.
- **Rotation:** the house range is −2°…+2°. Alternate cards get ±0.6°…±1.5°; on hover,
  cards straighten to 0° and lift 2–3 px. Never rotate body copy.

### 3.4 Z-index scale

| Layer | z |
|---|---|
| Paper ruled lines (`body::before`) | 0 |
| Floating music dust | 1 |
| Sections / hero content | 2 |
| Starbursts, ticker | 3 |
| Side-A strip | 40 |
| Pitch progress | 50 |
| Grain overlay | 60 |
| Topbar | 80 |
| Custom cursor | 99 |
| Loader | 200 |
| Toast | 300 |

Do not introduce intermediate values without updating this table. Grain sits above content
on purpose (print texture); it is pointer-events:none.

### 3.5 Motion tokens

| Token / usage | Value |
|---|---|
| UI hover / press | `.15s` ease |
| Letter pop | `.28s` |
| Toast slide | `.3s` |
| Loader fade | `.5s` |
| Scroll reveal | `.6s` opacity + transform |
| Blink | `1s` infinite (50% keyframe at opacity .35) |
| Float (note chip, scroll hint) | `2.2s` / `2s` infinite |
| Spin (starburst) | `14s` linear |
| Vinyl spin | `5s` linear |
| Wheel spin | `26s` linear |
| Ticker | `20s` linear, duplicated track |
| Confetti fall | `1.4–3s` per piece |

The only continuous animations are ambient: dust, waveform, piano roll, ticker, spinning
records/starbursts. They must all stop under `prefers-reduced-motion` (see §9).

---

## 4. Typography

### 4.1 Families and roles

| Family | Weights | Role |
|---|---|---|
| **Unbounded** | 400/600/800/900 | Display: logo, H1, section titles, buttons, labels on keys |
| **Anton** | 400 | Oversized `.mega` tail sections (Unbounded is the fallback) |
| **Instrument Serif** | 400 + italic | The human word inside a loud headline (“words”, “as music.”) |
| **Space Grotesk** | 400/500/700/900 | Body copy, cards, UI |
| **JetBrains Mono** | 400/700 | Every label: kickers, section numbers, meters, footnotes |

> `Archivo` and `Space Mono` are still requested from Google Fonts but are **unused**.
> Do not introduce them; remove them from the font URL when touching the head.

### 4.2 Scale

| Element | Size | Notes |
|---|---|---|
| Hero headline `.headline` | `clamp(56px, 11vw, 148px)` | Unbounded 900, line-height .92, uppercase, tracking −.02em |
| Mega `.mega` | `clamp(52px, 10vw, 140px)` | Anton, line-height .88; `.sm` = `clamp(44px, 8vw, 110px)` |
| Section title `.sec-title` | `clamp(34px, 5.4vw, 66px)` | Unbounded 900, line-height 1.0 |
| Manifesto | `clamp(28px, 4.6vw, 56px)` | Instrument Serif, line-height 1.1 |
| Hero sub | `clamp(15px, 2vw, 19px)` | Space Grotesk 500 |
| Lede | 17 px / `clamp(16px, 2.2vw, 21px)` | Max 620–640 px |
| Body | 14–16 px | Space Grotesk 400/500 |
| Mono label `.mono` | 11 px, tracking .14em, 700 | 10 px only for key sublabels/foot tabs |
| Kicker `.kicker` | 11 px mono, 900 | Paper pill with red border |
| Section label `.sec-label` | 12 px mono, 700, tracking 3px | Ink or yellow pill |
| Footer | 12 px mono | |

**Line length:** body copy 45–70 characters; ledes max 640 px; never full panel width.

### 4.3 Typographic rules

- Display type is **uppercase**; the serif accent inside it stays lowercase italic to create
  the “human word in a loud poster” contrast.
- Display headlines may mix three treatments: solid ink, paper knockout with ink stroke
  (`.outlined`), or a yellow/red block. Use at most **two** block colours per headline.
- Mono labels are always uppercase, with `·` between metadata items and `—` before a
  description. Examples: `02 / PIANO ROLL — LIVE PLAYGROUND`, `CH-01 • PLUG`.
- Numbers in setlists/section labels stay in the established sequence (see §5.4).
- Never letterspace body copy; reserve tracking for mono/display.
- Do not use more than one display family per headline block.

---

## 5. Layout

### 5.1 Page shell

```
fixed topbar (ink, 4px yellow bottom edge)
hero (paper, stage light, staff lines, piano)
ticker (red band, rotated)
manifesto (paper, serif card)
01–05 panels (light or dark, alternating rhythm)
⑪–⑭ tail sections (kc-section, light/dark alternating)
encore ticket + footer (inside ⑭)
fixed side strip (≥900), pitch progress (>700), grain, dust, cursor
```

The page is one continuous poster. There is **no sidebar, no multi-column app chrome and no
sticky section navigation** beyond the topbar.

### 5.2 Hero anatomy

1. Ambient layers: stage light cone, rotated staff lines, two starbursts, floating dust.
2. `.hero-top` mono status row (REC / BPM / latency).
3. Eyebrow pills (`FOR PIANISTS WHO TYPE`, `PLUG • PLAY • TYPE`).
4. H1 in two lines: `PLAY YOUR` / `words ♪ LOUD.`
5. Subcopy with a yellow-highlighted **is**.
6. Console: live transcript card + action buttons (`HEAR IT TYPE`, `CONNECT PIANO`).
7. Piano stage: waveform strip, control bar, 15 white keys + 10 black keys, pedal bar,
   footnotes.
8. Scroll hint.

Nothing may sit above the H1 except the fixed navbar and the status row. The H1 and console
MUST fit above the fold at 1440×900; the piano stage follows immediately (its top edge is
just visible at that height — 842 px in a 900 px viewport). Do not push the piano further
down, and keep it within one short scroll on mobile.

### 5.3 Section anatomy

Every content section follows one of two shells:

**Light `.panel`**

```html
<section class="panel" id="name">
  <div class="kicker mono">03 / PEDALBOARD — EVERY KEY DOES MORE</div>
  <h2 class="sec-title">Stompboxes <span class="serif italic">for typing.</span></h2>
  <p class="lede">One sentence that sets up the object.</p>
  <!-- one interactive object / grid -->
</section>
```

**Dark `.panel.dark` or `.kc-section.dark`**

```html
<section class="panel dark" id="name">
  <div class="kicker mono">04 / CHORD CHART — SHORTCUTS YOU CAN HEAR</div>
  <h2 class="sec-title">One finger types.<br><span class="serif italic">Three fingers command.</span></h2>
  <!-- paper cards inside -->
</section>
```

- Kicker first, then H2, then optional lede, then the object. No intro paragraph before the
  kicker.
- One H1 per page; sections use H2; cards use H3.
- Alternate light → dark to create setlist rhythm; never place two dark sections back to
  back without a light one between.
- Add `.reveal` to cards/groups below the fold. The observer adds `.in` at 15% visibility.

### 5.4 Section order and numbering

| # | Anchor | Section | Shell | Interactive object |
|---|---|---|---|---|
| — | `#top` | Hero | paper | Piano, live transcript, hear-it-type, metronome, MIDI connect |
| — | — | Ticker | red | Auto-scrolling mapping list |
| — | — | Manifesto / Overture | paper | Scroll-linked word-by-word reveal |
| 01 | `#how` | Soundcheck — how it works | paper | Three mixer channels with faders |
| 02 | `#play` | Piano roll playground | dark | Note-block canvas, EQ, velocity slider, waveform |
| 03 | `#pedals` | Pedalboard | paper | Six stompboxes, arpeggio + sustain demos |
| 04 | `#chords` | Chord chart | dark | Six chord buttons, circle of shortcuts |
| 05 | `#presets` | Record crate | paper | Four spinning vinyl presets |
| ⑪ | `#staff` | Quantize / staff | paper | Before/after spelling staff |
| ⑫ | `#plug` | Plug in your axe | dark | Enable Web MIDI, live status, cable pulse |
| ⑬ | `#setlist` | Tonight’s setlist | paper | Four audience cards |
| ⑮ | `#huffman` | Huffman piano | dark | Three word-macro chords, layer/velocity/timing chips, desktop app note |
| ⑭ | `#encore` | Encore / download | paper ticket | Tilting ticket, demo download, confetti, footer |

The jump from **05** to **⑪** is intentional setlist humour, not a bug. Do not renumber
existing sections; new sections continue at **⑮** (or are inserted without a number and
documented here). Never reuse an anchor.

### 5.5 Grids

| Pattern | Columns | Gap |
|---|---|---|
| `.mixer`, `.pedals`, `.chord-grid` | 3 | 16–20 px |
| `.crate` | 4 | 18 px |
| `.set-grid` | 4 (2 at ≤800, 1 at ≤560) | 12 px |
| `.roll-grid` | 1.6fr / .9fr | 18 px |
| `.staff-row` | flex, wraps | 16 px |
| Encore ticket | 120 px stub + 1fr | — |
| Piano | 15 flex white keys + absolute blacks | 0 |

### 5.6 Breakpoints

Breakpoints are grouped near the bottom of `style.css`; keep them there.

| Width | Changes |
|---|---|
| ≤900 px | Most multi-column card grids (mixer, pedals, chords, crate) become 1 column; `.roll-grid` 1 column; `.hide-m` and `.top-nav` hidden; custom cursor off (`cursor:auto`); starbursts hidden; `.set-grid` steps down separately at 800/560 |
| ≤800 px | `.set-grid` becomes 2 columns |
| ≤700 px | Hero padding 92 px; headline `clamp(30px,10.2vw,58px)`; hero lines `nowrap`; hero sub gets paper halo; pitch progress hidden; smaller pills; typed line 24 px |
| ≤560 px | Brand 14 px, version sup hidden; `.set-grid` 1 column; set-card H3 22 px |
| ≤420 px | MIDI dot hidden; ticket button compact; big buttons 13 px; white key labels 12 px and sublabels hidden; black key labels 9 px |

**Mobile invariants**

- No horizontal scrolling at 320 px.
- The H1 must stay on two lines at 320 px (`white-space:nowrap` per `.line`, scaled size).
- The piano must remain playable: keys shrink, labels shrink, sublabels disappear, never
  hide the keyboard.
- Touch targets ≥ 40×40 px where practical (piano keys are larger).

---

## 6. Components

### 6.1 Topbar

- Fixed, ink background, 4 px yellow bottom edge, hard shadow below, z 80.
- Left: brand block. `KEYCHANGE` on a yellow tile, rotated −3°, 2 px paper border, 2 px red
  shadow, plus a small `v2.4` superscript. The brand is a link to `#top`.
- Centre: 5 pill links (Soundcheck, Playground, Pedals, Chords, Spelling), 12 px bold
  uppercase; hover = yellow fill, ink text. Hidden ≤900 px.
- Right: MIDI status pill (`○ NO MIDI` grey → `● MIDI LIVE` green) + `GET TICKET ↓` button.
- The navbar **must not** contain animated VU bars or a long brand descriptor. The brand is
  only `KEYCHANGE` + version.

### 6.2 Buttons and links

| Class | Look | Use |
|---|---|---|
| `.btn` | Yellow, 3 px ink border, 14 px radius, 4 px hard shadow, 900 uppercase | Standard action |
| `.btn.ghost` | Paper card fill, same border/shadow | Secondary action inside a card |
| `.btn.big` | 16/24 px padding, 15 px text | Hero actions |
| `.btn-ticket` | Red, 2 px white border, 3 px yellow shadow, rotate 1° | Navbar CTA |
| `.btn-big` | Red, white large text, 6 px ink shadow, rotate −1° | Final download CTA |
| `.mini-btn` | Ink fill, paper text, 2 px ink border, 3 px red shadow | In-card demos |
| `.piano-controls button` | Paper card, 3 px ink border, 4 px shadow, mono 11 | Piano console |
| `.piano-controls button.on` | Yellow fill, pressed translate | Active toggle |

**States**

- Hover: lift 2 px, rotate 1° opposite the resting tilt, shadow grows; `.btn` turns red with
  white text (large text only).
- Active: translate +2 px, shadow collapses to 1 px.
- Disabled/off: keep the same silhouette; change fill to `#222` + grey border, never lower
  opacity below .6.
- Focus: **add `:focus-visible`** (currently missing globally — see §10.3). Use a 3 px
  yellow outer ring plus a 2 px ink gap; do not remove outlines.

### 6.3 Labels, pills and chips

- `.kicker` — paper pill, 3 px red border, `--red-deep` text, 900, rotate −2°, small shadow.
- `.sec-label` — mono 12 px, ink fill + yellow text, 2 px ink border, rotate −1°. Add
  `.light` on dark panels for yellow fill + ink text.
- `.pill` — 100 px radius, 3 px ink border, uppercase mono 11 px, paper fill; `.ghost` is
  blue with white text, rotated +1.2°.
- `.chip` — white fill, 2 px ink border, 3 px ink shadow, mono 12 px; used for supported
  devices and metadata tags.
- `.foot` — absolute bottom-right tab on stompboxes: yellow, 2 px ink border, 10 px mono,
  tracking .2em.

### 6.4 Display headlines

- `.sec-title` uses Unbounded 900 uppercase, with one `.serif.italic` phrase highlighted on
  a yellow band or solid yellow block.
- `.mega` uses Anton for the tail sections; `.stroke` is transparent fill with a 2.5 px ink
  stroke (paper stroke on dark).
- `.hl-y` = yellow block, ink text, rotate −1°; `.hl-r` = red text.
- On dark panels, the serif accent flips to ink-on-yellow.

**Do not** place a white-on-red small phrase inside a headline; use ink on yellow or white
on blue.

### 6.5 Cards

All cards: 3–4 px ink (or paper) border, hard offset shadow, paper or spot-colour fill,
optional ±1° rotation that straightens on hover.

| Card | Anatomy | Notes |
|---|---|---|
| `.channel` (mixer) | mono channel label, 54 px emoji icon, H3, one-line body, fader, value tab | 3-up; channel 2 is yellow, rotations alternate |
| `.pedalbox` | LED, 3 knobs, emoji + H3, body, prop, bottom `.foot` tab | 3-up; LEDs may blink; props are interactive |
| `.chord` | 3 mini key blocks, chord name, mono shortcut | On dark panel; paper card, blue shadow; `.playing` turns yellow |
| `.vinyl` | spinning disc, preset name, mono hand note | 4-up; each disc has its own repeating-radial palette |
| `.vu-card` | mono caption + EQ / slider / waveform | On dark panels, card fill, paper border, red shadow |
| `.set-card` | set number, H3, body, energy dots | Blue/red alternating, 4-up → 2 → 1 |
| `.staff-demo` | ruled white sheet, clef, words, barline, green shadow | Demonstrates before/after correction |
| `.plug-box` | dark inner box, cable line, big green button, status | Single object, centred |
| `.ticket` | stub + main panel | Encore only, see §6.12 |
| `.toast` | fixed bottom pill | See §6.8 |

**Card copy length:** H3 ≤ 4 words; body ≤ 18 words; mono note ≤ 34 characters. Cards are
scannable; long copy belongs in a lede or the manifesto.

### 6.6 Piano console

The piano is the product. Treat its geometry as fixed.

- Stage max width 1060 px (`.piano-stage`); the keyboard sits in `.piano-wrap.kc-piano`
  (`#1b1a16`, 4 px ink border, 10 px hard shadow, padding 14/14/10).
- Waveform strip 64 px on ink above the keys (`#wave`, 4 px ink border, no bottom border),
  then a mono fall hint line: `CLICK LOW ON A KEY = HIT IT HARDER = CAPS / BOLD`.
- Control bar: octave switch (C1 · abc / C2 · ABC / C3 · 123!?) on the left; buttons
  (OCT ◀, SUSTAIN, OCT ▶, CLEAR, ⌫ BEND, ♪ ON/OFF) on the right; wraps on mobile.
- Keyboard height 220 px. 15 white keys flex equally; 10 black keys absolute, width 5.4%,
  height 58%, left `calc((slot + 1) * (100% / 15) - 2.7%)`.
- White keys: ivory gradient, 2 px black border, 6 px bottom radius, Unbounded letter 21 px,
  mono QWERTY hint 10 px at 60% opacity.
- Black keys: near-black gradient, 2 px border, yellow mono symbol 12 px.
- Active white = yellow fill + translateY(3 px); active black = red + white + translateY(3 px).
- Click position maps to velocity: top of key = soft, bottom = hard, range 35–127. Black
  keys are fixed at 105.
- A `.burst` note sticker floats up from the pressed key for 0.9 s.
- Pedal bar: near-black strip, striped yellow/red fill when held, velocity track with a
  green→yellow→red gradient.
- The piano is C4–C6 (MIDI 60–84) across three layers; black keys are the symbol layer.
- `#piano` has `role="application"` and `aria-label="Playable piano typer"`. Global QWERTY
  and MIDI handlers provide the actual keyboard access; do not remove them.

**Never** change the number of white keys, the MIDI mapping or the layer strings without
updating the JS maps (`whiteIdx`, `layers`, `blackSlots`, `blackSyms`) and this section.

### 6.7 Loader

- Full-screen ink overlay, z 200, centered column: mono “KEYCHANGE PRESENTS”, giant word,
  progress bar, mono subline.
- Words cycle every 260 ms: `TUNING…` → `A = 440Hz ✓` → `VELOCITY ✓` → `PEDAL ✓` →
  `FORTISSIMO!`; fill advances 20% each; overlay fades after the last word +450 ms; a 3 s
  fallback always dismisses it.
- **One-line invariant:** `.loader-big` is a fixed-height flex row with `white-space:nowrap`;
  the inner `<span>` is scaled down by JS to fit its parent. The bar’s vertical position MUST
  be identical for every word at 1440, 800, 390 and 320 px. Do not remove `fit()`, the
  wrapper span, or the fixed-height flex box.
- The loader is decorative; never use it to gate content or wait for audio.

### 6.8 Toast

- Fixed bottom-center, ink fill, yellow text, Unbounded 800 18 px, 2 px yellow border, z 300.
- Enters via translateY(120 px) → 0 over .3 s; auto-dismiss 1.9 s.
- Use for confirmation only (`𝄢 QUANTIZED: teh → the ✓`, `MIDI CONNECTED`, `cleared`).
  Never use for errors that need action; those belong inline in the section.

### 6.9 Forms & sliders

The only native control is the velocity range input (`#velSlider`): full width,
`accent-color: var(--red)`, 28 px tall, labelled `Velocity (boldness)`. It drives the
mono value, the demo word (piano → forte) and future note velocities. Any new slider MUST
have a label and use the red accent.

### 6.10 Footer

- Inside the encore section, ink text on paper: three columns — copyright, sample credit
  link, “BACK TO STAGE ↑”.
- Mono 12 px, top border 2 px ink, gap 10 px, wraps.
- The sample credit is a real link and MUST remain while Salamander samples ship.

### 6.11 Decorative layers

| Layer | Behaviour | Rules |
|---|---|---|
| Music dust `#dust` | Canvas, 70 glyphs drifting up, 55% opacity | `aria-hidden`, pointer-events none |
| Grain `.grain` | Inline SVG noise, 16% multiply, fixed, z 60 | Static, no animation, pointer-events none |
| Stage light | Conic yellow rays masked radially; follows the mouse ±60 px on pointer devices | Decorative; static on touch, never hidden |
| Starbursts | 132 px clip-path starburst stickers, spin 14 s, colours red/green | Decorative only; hide ≤900 |
| Staff background | 5 ink lines, rotate −2°, behind hero | `aria-hidden` |
| Ticker | Red band, 4 px ink edges, rotated −1.4°, duplicated track | 20 s loop; text is decorative but readable |
| Side strip | Fixed left, rotated −90°, `★ SIDE A — TYPE ★ SIDE B — PLAY ★` | Budget exception: inline styles; move to CSS when touched |
| Pitch progress | Fixed right, 14×220 px, striped fill, scroll % | Hidden ≤700 |
| Custom cursor | 14 px red dot + 38 px ring, z 99; native cursor hidden on desktop | Intended for ≥900; restore `cursor:auto` and hide the custom elements on touch |

Decorative layers never intercept pointer events and never cover interactive controls.

### 6.12 Encore ticket

- Paper ticket on yellow, 3 px ink border, 12 px ink shadow, grid `120px 1fr`.
- Left stub: ink fill, yellow mono 12 px, vertical writing mode, dashed yellow divider.
- Main: section label, `STOP TYPING. START PERFORMING.` mega headline (red highlight),
  red download button, green free tag, fine print, mono footer note.
- Pointer movement tilts the ticket ±2° in Y (perspective 900 px). No tilt on touch.
- Confetti (26 mono stickers) fires on download click; each piece removes itself after 3.2 s.

---

## 7. Interaction model

### 7.1 Input map

| Input | Behaviour |
|---|---|
| Click/tap white key | Types the current layer’s letter; click Y sets velocity 35–127 |
| Click/tap black key | Types punctuation/symbol; velocity fixed at 105 |
| Click lower on a key | Harder hit → uppercase/bold emphasis |
| `A S D F G H J K L ;` | White keys 0–9 |
| `W E T Y U` | Declared in `qmap` but not wired (legacy) |
| `Z` / `←`, `X` / `→` | Previous/next layer (abc → ABC → 123!?) |
| `Space` (hold) | Sustain pedal + types a space; release damps voices |
| `Backspace` | Deletes the last typed character |
| Velocity slider | Sets global velocity 20–127 and the forte/piano demo |
| Octave buttons | Layer switch (same as Z/X) |
| `CLEAR` | Clears transcript (keeps the caret) |
| `⌫ BEND` | Deletes one character with a low tone and a toast |
| `♪ ON/OFF` | Mutes/unmutes audio |
| Metronome `OFF/ON` | 120 BPM click + blinking dot |
| Chord card click | Plays a C-major arpeggio and toasts a shortcut |
| Hold 3 piano keys | Fires a chord shortcut toast after 380 ms |
| `HEAR IT TYPE` | Plays “play your words loud” as notes/letters |
| `ENABLE WEB MIDI` | Requests MIDI access; maps `note % 15` to white keys (demo) |
| Encore download | Demo: plays a chord, fires confetti, toasts; does not download |

### 7.2 Typing rules

- Transcript is capped at 42 child nodes; oldest characters are removed first.
- Each character is a `<span class="nk">` inserted before the blinking caret.
- Font size scales with velocity (`22 × (0.9 + vel/127 × 0.9)` px); weight: 400 < 75,
  700 < 105, 900 above; italic below 45; fortissimo gets yellow fill + red text + ink border.
- Layer 1 forces uppercase for a–z; velocity > 100 also forces uppercase.
- Black-key symbols get `.black-k` (blue); sustained spaces get `.sus` (red) and render `␣`.
- **Quantize:** when the visible text ends in `teh`, the last three characters are replaced
  by `the`, each highlighted green, and a toast confirms. This is the only correction today.
- WPM is computed from characters in the last 60 s (`chars / 5 / minutes`, capped 240) and
  updates every second.
- No state is persisted; refreshing resets the transcript, WPM and layer.

### 7.3 Chords

- Card click plays a C-major arpeggio regardless of the card’s label (demo simplification)
  and rotates a random shortcut message into the toast.
- Holding three piano keys for 380 ms triggers the same toast; the on-screen keys flash.
- The circle of shortcuts shows 12 fixed actions: copy, paste, save, undo, bold, italic,
  find, enter, space, tab, delete, send.

### 7.4 MIDI

- Web MIDI only (Chrome/Edge, HTTPS). Failure shows an inline explanation, not an alert.
- Note-on maps `note % 15` to a white key and passes real velocity; note-off releases.
- This is a demo mapping. A real layout/preset system belongs in the product, not this page.
- The MIDI status pill is the only place the `live` green state appears.

### 7.5 Feedback & state

- Every action gets immediate visual feedback: key flash, note sticker, transcript character,
  canvas kick, EQ bar, toast.
- Audio failures are silent; the visual toy must keep working.
- Buttons reflect state with `.on` / `.active`, text changes (`ON`/`OFF`) and colour — never
  colour alone.
- Reduced-motion users get all state changes without transforms or loops.

---

## 8. Audio system

### 8.1 Source and license

- **Salamander Grand Piano V3** by Alexander Holm, CC BY 3.0. Original masters from
  archive.org, prepared as one sample every three semitones from A0 to C8.
- Files live in `assets/audio/piano/<note>.mp3` — 30 files, 96 kbps mono, 44.1 kHz, tails
  trimmed at −45 dB with a short fade, ~3.99 MB total.
- The `v8` (mezzo) velocity layer is used. Attribution lives in
  `assets/audio/piano/LICENSE.txt`, the footer and the README. **Do not remove attribution
  while these samples ship.**
- The source piano is stretch-tuned. `PIANO_TUNE` corrects each sample in cents so the
  instrument plays in equal temperament. If samples are replaced, re-measure the tuning
  map; do not delete it silently.

### 8.2 Loading strategy

- Audio starts **after** `window.load`, inside `requestIdleCallback` (1.8 s timeout).
- The middle register (MIDI 45–81, 13 files, ~1.63 MB) loads first; outer octaves load in
  idle chunks of three.
- `navigator.connection.saveData` or a 2G effective type skips background loading; the
  middle register remains.
- The first pointer/key gesture may create/resume the AudioContext inside the gesture.
- Until a sample is decoded, the additive `synthNote` fallback plays; within six semitones
  of a loaded sample, the sample is pitched instead.
- Never move sample loading before `load`, and never block rendering or the loader on audio.

### 8.3 Signal graph

```
sample (or synth) → lowpass → per-note gain → stereo panner → master
master → lowshelf 190 Hz +2.5 dB → highshelf 7.2 kHz −2 dB
       → compressor (−14 dB, knee 16, 3:1, 4 ms attack, 280 ms release)
       → bus 0.95 → dry 0.93 → destination
                 → convolver (1.7 s, decay 3.4) → wet 0.14 → destination
```

- Master gain `0.88`.
- Voice lowpass: `min(16000, 1500 + vel^1.5 × 8500 + max(0, midi−48) × 10)` Hz, Q 0.25.
- Voice gain peak: `2.2 × vel^1.15 + 0.06`; attack 6 ms; dry notes settle to 0.85 then 0.18
  (0.3 s / 0.5 s time constants); pedal notes settle to 0.95 (0.6 s).
- Pan: `(midi − 62) / 54`, clamped ±0.35.
- Playback rate: `2^((midi − sampleMidi)/12) × 2^(−tuneCents/1200)`.
- Stop the source after `buffer.duration / rate + 0.05 s`; disconnect all nodes on end.
- Sustain release damps every live voice (`setTargetAtTime(0.0001, now, 0.12)`).

### 8.4 Event sounds

- Metronome click: square 1760 Hz, 65 ms, gain `0.05 × vel/127`.
- No UI sound effects other than notes, the metronome click and demo arpeggios.
- Sound toggle stops all new voices; existing tails finish naturally.

### 8.5 Rules for new audio

- New sounds MUST reuse `master` (never connect directly to `destination`).
- New instruments MUST respect `soundOn`, the AudioContext gesture rules and reduced motion.
- Any new sample set needs a license file, attribution in the footer/README and a loading
  strategy that does not block the hero.
- Keep total audio under ~4 MB unless there is a measured reason; prefer mono 96–128 kbps
  MP3 for compatibility.

---

## 9. Motion

- Motion is **print coming alive**: paper lifts, stickers float, records spin, characters
  pop. Nothing eases softly like a mobile app; use short, snappy transitions and linear
  loops.
- Entrance motion: `.reveal` starts 30 px low with a 0.5° tilt, fades in over .6 s when 15%
  visible. Do not animate whole sections on every scroll event.
- Interaction motion: hover lift + rotate; active press; key flash 170 ms; letter pop 280 ms.
- Ambient loops: dust, waveform, piano roll, ticker, discs, starburst, wheel. Keep their
  speeds as listed in §3.5.
- **Reduced motion:** the global block at the end of `style.css` collapses all animation and
  transition durations, forces `.reveal` visible, and removes spin/float from the ambient
  objects. New animated components MUST still communicate state with this block active.
- Avoid parallax, scroll-jacking, autoplaying carousels and anything that moves text the user
  is reading.

---

## 10. Accessibility

### 10.1 Baseline

- Target: WCAG 2.2 AA. Current Lighthouse (mobile, local, 2026-11-11): **accessibility 100,
  best practices 100, SEO 100**, performance 81 (LCP 3.5 s under throttling — known gap).
- `lang="en"`, one H1, logical heading order, landmark `<header>`, `<main>`, `<footer>`.
- The decorative layers (`#dust`, grain, staff lines, starbursts) are `aria-hidden` or
  pointer-events:none; the roll/waveform canvases carry no information that is not also in
  the transcript or meters.
- The nav has `aria-label="Sections"`; the piano has `role="application"` + label; the
  velocity slider has a label.
- Keyboard users can play with `A S D F G H J K L ;`, shift layers with Z/X, sustain with
  Space and delete with Backspace. MIDI adds real keys.

### 10.2 Contrast rules

- Use the pairs in §3.1 only. **Never** small white text on red or small green text on ink.
- If you need a new coloured surface, compute the ink/paper pair first. Default to ink text.
- Decorative colour may fail contrast as long as it carries no information.
- Do not encode state in colour alone: pair the green MIDI dot with “MIDI LIVE”, the red LED
  with a label, the yellow key with its typed letter.

### 10.3 Focus & input

- **Known gap:** there are no `:focus-visible` styles in `style.css`. New interactive
  components MUST add a visible focus ring (3 px yellow + 2 px ink gap) until a global rule
  is added. This is the highest-priority accessibility fix.
- The custom cursor hides the native cursor on desktop. Interactive elements must remain
  reachable by keyboard and show focus.
- On touch/≤900 px, `cursor:auto` is restored and hover-only affordances are decorative.

### 10.4 Canvas & generated content

- Canvas visuals are decorative; the same information is present in text (transcript, note
  labels, WPM/velocity meters).
- Generated stickers/confetti contain only symbols and remove themselves after ~3 s. When
  touching them, add `aria-hidden="true"` to the container so screen readers never announce
  party debris, and never put essential content only in Canvas.

---

## 11. Content & metadata

### 11.1 Head

Every page MUST include:

- `charset`, responsive viewport, `title`, meta description, keywords, author, robots,
  theme-color `#17130c`, color-scheme light, canonical URL.
- Open Graph (`website`, site name, url, title, description, 1200×630 image with width,
  height, type, alt, locale) and Twitter `summary_large_image`.
- Icon family: `favicon.svg`, `favicon.ico`, `favicon-32.png`, `favicon-192.png`,
  `favicon-512.png`, `apple-touch-icon.png`; manifest; app title/name; iOS web-app meta.
- JSON-LD `@graph`: `WebSite` + `SoftwareApplication` with price 0, free flag, feature list,
  screenshot, repo `sameAs`. Update this when the real app ships.

### 11.2 Current values

| Field | Value |
|---|---|
| Canonical / OG URL | `https://abhinav-prabhakar.github.io/KeyChange/` |
| Title | `KeyChange — Type With Your Piano` |
| Description | “KeyChange turns any MIDI piano into a text instrument…” (~155 chars) |
| OG image | `assets/og.png`, 1200×630 |
| Theme / manifest theme | `#17130c` |
| Manifest background | `#f6edda` |
| Sitemap lastmod | `2026-11-11` |

### 11.3 URLs and deployment

- GitHub repo: `Abhinav-Prabhakar/KeyChange`; Pages from `main` root; `.nojekyll` present.
- GitHub lowercases the owner path: always use `abhinav-prabhakar.github.io`.
- New pages need their own canonical, OG tags, JSON-LD, sitemap entry and relative asset
  paths that work under the `/KeyChange/` base path.
- Never hard-code `localhost` in shipped metadata.

### 11.4 Copy slots

- Section kicker: `NN / NAME — SUBTITLE` (mono, uppercase).
- Section title: one declarative + one serif accent.
- Lede: one sentence, ≤ 22 words.
- Card: H3 ≤ 4 words, body ≤ 18 words.
- CTA: verb-first, uppercase (`GET TICKET ↓`, `HEAR IT TYPE`, `ENABLE WEB MIDI`).
- Fine print: mono caps, `·` separated (`MACOS 14+ · WINDOWS 11 · 4MB · NO ACCOUNT`).

---

## 12. Assets

### 12.1 Naming & formats

- Screenshots: `assets/01-hero.webp` … `10-mobile.webp`, 1440×1000 at DSF 2, WebP q88.
  Keep the two-digit prefix and descriptive slug.
- Social card: `assets/og.png` (1200×630), generated from `assets/og-card.html`; the source
  is committed and should stay in sync.
- Icons: SVG first, plus ICO and 32/180/192/512 PNG. Transparent corners; yellow tile with
  an ink eighth note.
- Audio: `assets/audio/piano/<note>.mp3` + `LICENSE.txt`. Note names use `s` for sharp
  (`Ds1`, `Fs1`) because the file system and map use that form.
- No raw design files, no unused variants, no multi-megabyte hero images.

### 12.2 Adding an image

1. Export at 2× the display size, crop to the intended ratio.
2. Convert to WebP q88 for README screenshots; keep PNG only for OG/icons.
3. Add width/height or an aspect-ratio box to prevent layout shift.
4. Write specific alt text: what is shown and why it matters, not “screenshot”.
5. Update the README gallery and this document if it is a new canonical shot.

---

## 13. Code conventions

### 13.1 File rules

- **One page, one CSS file, one JS file** until the product app lands. Do not add a build
  step, a framework, a package manager or a CSS preprocessor without an explicit decision.
- No external JS. Google Fonts is the only third-party runtime dependency; audio samples
  are self-hosted.
- Keep `index.html` semantic and readable; sections in page order; one blank line between
  major blocks.
- `design.md` and `README.md` are part of the product; update both when behaviour changes.

### 13.2 CSS

- Declarations are compressed onto one line per rule (`selector{prop:value;...}`), no
  nesting, no `!important` except the reduced-motion block.
- Tokens first in `:root`; base/reset; shared utilities; components in page order; legacy
  tail; reduced motion; responsive blocks at the bottom.
- Prefer classes over IDs for styling; IDs are for JS hooks and anchors.
- Use `clamp()` for display type, never fixed px for headlines.
- New components go next to their section, not in a separate file. Add new breakpoints to
  the bottom groups.
- Inline `style` is allowed only for per-instance custom properties (`--v`, `--h`, `--p`),
  canvas coordinates and generated confetti. Move anything reusable into CSS.

### 13.3 JavaScript

- `const $ = s => document.querySelector(s)` and `$$` for lists.
- Isolate one-off features in an IIFE; guard every query (`if (!el) return`) so a removed
  section cannot break the page.
- Wrap all audio in `try/catch`; a missing AudioContext must never throw into the UI.
- Bind events directly (`el.onclick = …`) as the page does; no synthetic framework.
- Keep state in module-level `let` variables; no persistence, no cookies, no network calls
  beyond fonts and samples.
- Canvas loops use `requestAnimationFrame`; skip drawing until the canvas has a width.
- New canvases must resize on `resize` and respect device pixel ratio only if it materially
  improves the look (current canvases are 1×).

### 13.4 HTML

- IDs in camelCase (`typedOutput`, `midiDot`); classes kebab-case (`pedalbox`, `vu-card`);
  data attributes kebab (`data-i`, `data-b`, `data-keys`).
- Buttons are `<button>`; links are `<a>`; do not put click handlers on `<div>` when a
  button is semantically correct.
- Decorative elements get `aria-hidden="true"`.
- Every image needs `alt`; every icon-only control needs an accessible name.

### 13.5 Performance

- Audio starts after `load`; core register first; 4 MB total.
- Fonts: preconnect to both Google hosts; only the families actually used.
- Images: WebP for screenshots; no image above the fold larger than necessary.
- No layout shift: reserve heights for canvases, the loader, key rows and the transcript.
- Measure with Lighthouse before shipping; do not regress a11y/BP/SEO below 100 or
  performance below the current 81 without a reason.

---

## 14. Definition of done

### 14.1 New component

- [ ] Uses documented tokens, fonts and spacing; no new hex without a token.
- [ ] Light/dark variant behaves correctly; text never ink-on-ink.
- [ ] Contrast passes for its size (see §3.1); no small white-on-red or green-on-ink.
- [ ] Hover, active, `:focus-visible`, disabled/off states exist.
- [ ] Keyboard operable; `aria-label`/semantics present; decorative layers hidden.
- [ ] Works at 320, 390, 768, 1024, 1440 px; no horizontal scroll or overlap.
- [ ] Reduced-motion version communicates the same state.
- [ ] No layout shift; canvases have reserved height.
- [ ] If below the fold, has `.reveal`; if it makes sound, routes through `master`.
- [ ] Documented in this file (anatomy + rules) and, if user-facing, in the README.

### 14.2 New section

- [ ] Follows the light/dark rhythm; does not put two darks back to back.
- [ ] Kicker + H2 + optional lede + one interactive object.
- [ ] Section number/anchor does not collide (see §5.4) and is added to the table.
- [ ] Has an `id` and appears in the top nav only if it is a primary destination.
- [ ] Mobile stack is designed, not just squeezed (check 320 px).
- [ ] Scroll reveal added; content readable with JS disabled (the current page has a known
  gap here — add a no-JS fallback when touching reveals; see §15).
- [ ] Metadata unchanged or updated (title/description/OG/JSON-LD) if the page meaning
  changed.

### 14.3 Release

- [ ] `node --check app.js` passes; no console errors on load or first interaction.
- [ ] Loader bar top is identical for all five words at 1440/800/390/320.
- [ ] Piano: 30 samples decode, a key press produces a note, sustain rings and damps, sound
  toggle works, no clipped peaks.
- [ ] MIDI connect and failure paths both show sensible status.
- [ ] Lighthouse: a11y/BP/SEO 100; performance not regressed.
- [ ] Live GitHub Pages build succeeds; `/assets/audio/piano/A4.mp3` returns 200.
- [ ] Commit message describes the user-visible change; push to `main`.

---

## 15. Known gaps & next decisions

These are real, current deviations — fix them deliberately, not accidentally.

1. **No global `:focus-visible` styles.** Add a 3 px yellow + 2 px ink focus ring for links,
   buttons, inputs, chords, piano keys and the ticket. Highest-priority a11y fix.
2. **Download is a demo.** `#dlBtn` plays a chord and confetti; no file exists. Replace the
   href and copy when the app ships, and update JSON-LD/OG accordingly.
3. **MIDI mapping is a demo.** `note % 15` maps any keyboard to the on-screen white keys; no
   real layout/preset engine yet.
4. **Quantize only handles `teh → the`.** The marketing claim is broader than the demo.
5. **Legacy/unused code:** `.nav`, `.stats`, `.stubs`, `.tickets`, `.faq`, `.finale`,
   `.btn.small`, `.btn.giant`, `.vu`, `.wkey/.bkey`, `--paper2`, `--line`, `qmap` are unused.
   Remove them when touching those areas; do not build on them.
6. **Unused fonts:** Archivo and Space Mono are still downloaded. Remove from the Google
   Fonts URL if not adopted.
7. **`.midi-dot.live` is 11 px green on ink (3.45:1)** and fails AA. Use paper text or a
   brighter green when the live state is refined.
8. **Performance:** mobile Lighthouse 81 with LCP 3.5 s, mostly font/hero cost. Future work:
   subset fonts, preload the hero font, consider system fallbacks.
9. **Inline styles:** the side strip, faders and generated stickers use inline styles. Move
   reusable values to CSS when those blocks are next edited.
10. **No automated test suite.** Verification is manual Playwright/Lighthouse; add a tiny
    smoke test if the app grows.
11. **Chord card audio is generic:** every chord card plays a C-major shape. Play the
    labelled chord when the real layout engine lands.
12. **The loader always dismisses at 3 s** even if fonts are still loading, by design. Do not
    reintroduce a blocking loader.
13. **Black-key MIDI values are a demo approximation.** The visual slots are correct, but
    `pressBlack`/`keyEls` use `61 + b × 2` (a chromatic run) instead of the true semitones
    for the engraved positions (`C#4, D#4, F#4, G#4, A#4, C#5, …`). The printed QWERTY hints
    on white keys are also decorative and do not fully match the `KeyA…Semicolon` handler.
    Fix both with a single `blackMidi` map and a single key-hint map when the real layout
    engine is built.
14. **`.btn:hover` is white-on-red at 12–15 px (3.71:1)** and fails AA. The resting state is
    fine; switch the hover text to ink (4.99:1) or reserve white for display-size red
    blocks. The red starburst stickers use white 13 px but are decorative and hidden on
    mobile; do not reuse that pairing for information.
15. **No-JS failure mode.** If `app.js` fails to load or JavaScript is disabled, the loader
    never receives `.done` and covers the page forever, and `.reveal` sections stay at
    opacity 0. Add a `<noscript>` block that hides `#loader` and forces `.reveal` visible;
    the toys can stay inert. Treat this as a launch blocker for the product app.

---

## 16. Document history

| Date | Change |
|---|---|
| 2026-11-11 | Initial design system: brand, tokens, components, piano, audio, motion, accessibility, metadata, checklists, known gaps. Verified against commit `4ec1da6`. |
| 2026-11-11 | Add ⑮ Huffman piano section (dark): word-macro chords the/and + interval bigrams, layer/velocity/timing chips, ticker updated to Huffman mappings, desktop Rust+GPUI mock lives in `app/`. |

When you change the design, add a row here with the date and a one-line summary.

---

## Appendix A — Starter snippets

### A.1 New light section

```html
<section class="panel" id="new-thing">
  <div class="kicker mono">NN / NEW THING — SHORT SUBTITLE</div>
  <h2 class="sec-title">A loud claim.<br><span class="serif italic">A human aside.</span></h2>
  <p class="lede">One sentence that sets up the object below.</p>
  <div class="mixer"><!-- three cards, or one full-width object --></div>
</section>
```

### A.2 New dark section

```html
<section class="panel dark" id="new-dark">
  <div class="kicker mono">NN / NEW DARK — SHORT SUBTITLE</div>
  <h2 class="sec-title">Another claim.<br><span class="serif italic">Another aside.</span></h2>
  <div class="roll-grid"><!-- paper cards inside the ink stage --></div>
</section>
```

### A.3 New paper card

```html
<div class="channel reveal">
  <div class="ch-top mono">CH-04 • LABEL</div>
  <div class="ch-icon">🎛</div>
  <h3>Short title</h3>
  <p>One sentence, 18 words maximum.</p>
  <div class="fader"><i style="--v:70%"></i></div>
  <div class="mono ch-val">VALUE</div>
</div>
```

### A.4 Action row

```html
<div class="hero-actions">
  <button class="btn big" id="primaryAction">▶ DO THE THING</button>
  <button class="btn ghost big" id="secondaryAction">⌁ SECONDARY</button>
</div>
```

### A.5 Token block (for a future app stylesheet)

```css
:root{
  --paper:#f6edda; --card:#fff8e8;
  --ink:#17130c; --red:#ff2e1f; --red-deep:#c81a0f;
  --blue:#2233ff; --yellow:#ffc700; --green:#0e7a4f;
  --shadow:6px 6px 0 var(--ink);
  --shadow-sm:4px 4px 0 var(--ink);
}
```

Copy these from `style.css`; never fork the palette.

### A.6 Applying this to the product app

Keep the palette, type families, hard borders, offset shadows, spot-colour states and the
voice. Drop the landing-page theatre: no grain, dust, starbursts, custom cursor, ticker,
scroll reveal or spinning records in the app chrome. Use a calmer density (8 px grid),
persistent navigation only where the app needs it, and the same contrast/focus/reduced-motion
rules. The app may feel like the same print shop; it must not feel like the same poster.
