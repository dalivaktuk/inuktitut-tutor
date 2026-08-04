# inuktitut-tutor

A terminal typing tutor for learning Inuktitut syllabics on the Canadian
`ca(ike)` keyboard layout, built with [ratatui](https://ratatui.rs).

Home-row-first, `fff jjj`-style muscle-memory drills across fourteen graded
steps, from `ffff jjjj` up through real words and the Shift layer. Moving
from the home row to the top row, and from the bottom row onward, is gated
on proficiency (accuracy and wpm) — fall short and the app loops you back
through that row's lessons for another pass instead of pressing on.

## Running it

The app itself is the same on every platform — it's a plain terminal program
built on [ratatui](https://ratatui.rs)/[crossterm](https://github.com/crossterm-rs/crossterm),
which run natively on Linux, Windows, and macOS. What differs per platform is
**which OS keyboard layout to activate** so your keys actually produce
syllabics, since a terminal app can only see the character your OS layout
produces for a keypress — never the physical scancode. Pick your platform
below.

Prebuilt binaries for each platform are attached to the
[Releases](../../releases) page once one exists; otherwise (or to build from
source yourself) install Rust via [rustup.rs](https://rustup.rs) — a few
minutes on any of the three platforms — then from the project directory:

```sh
cargo build --release
```

The binary lands at `target/release/inuktitut-tutor` (`.exe` on Windows).

### Linux

Activate the layout so your keyboard produces syllabics directly (no dead
keys, no mode toggle — every key is a straight 1-key = 1-syllabic mapping,
Shift included):

```sh
setxkbmap ca ike
```

Run the app (`cargo run --release`, or the built binary directly). To go
back to your normal layout afterwards:

```sh
setxkbmap us   # or whatever layout you use normally
```

#### Long vowels: built in on Windows/macOS, custom layout on Linux

The long-vowel (doubled-vowel) syllabics — ᐆ ᖂ ᑰ ᑑ ᓲ ᒨ ᓅ ᓘ — live on a third
keyboard level, reached with right-Alt or Option. **Windows and macOS give
you that level by default. Linux is the only platform that needs extra
setup:**

| Platform | Long vowels | How |
| --- | --- | --- |
| Windows | Built in | `AltGr` (right Alt) + key, Caps Lock on |
| macOS | Built in *(expected — see the macOS section)* | `Option` (⌥) + key, Caps Lock on |
| Linux | Needs [`xkb/ike`](xkb/ike) from this repo | stock `ca(ike)` has only two levels per key |

The reason Linux is the exception: the stock `ca(ike)` layout shipped in
`xkeyboard-config` defines only **two levels per key** — base and Shift —
so there is simply nowhere for a third level to live. Confirm it yourself:

```sh
sed -n '/xkb_symbols "ike"/,/^};/p' /usr/share/X11/xkb/symbols/ca | grep 'key <AC'
# key <AC01>  {[  U1591,    U148d  ]};   <- two entries, no third level
```

The fix is a custom XKB layout adding right-Alt (`level3`) as a modifier.
**One is included in this repo at [`xkb/ike`](xkb/ike)** — originally
generated with AI assistance and used daily by this project's author. With
that third level in place, the home row gains its long-vowel forms:

(These codepoints are the same on all three platforms — only the way you
reach them differs.)

| Key | Base | Long vowel (`AltGr` / `Option`) |
| --- | --- | --- |
| `s` | ᐅ `U+1405` O   | ᐆ `U+1406` OO   |
| `d` | ᖁ `U+1581` QO  | ᖂ `U+1582` QOO  |
| `f` | ᑯ `U+146F` KO  | ᑰ `U+1470` KOO  |
| `g` | ᑐ `U+1450` TO  | ᑑ `U+1451` TOO  |
| `h` | ᓱ `U+14F1` SO  | ᓲ `U+14F2` SOO  |
| `j` | ᒧ `U+14A7` MO  | ᒨ `U+14A8` MOO  |
| `k` | ᓄ `U+14C4` NO  | ᓅ `U+14C5` NOO  |
| `l` | ᓗ `U+14D7` LO  | ᓘ `U+14D8` LOO  |

So the regular home row reads ᐅᖁᑯᑐᓱᒧᓄᓗ and the right-Alt layer reads
ᐆᖂᑰᑑᓲᒨᓅᓘ. (The pattern is consistent across the block: the long form is
always the base codepoint + 1.)

**Installing it.** Copy the file into your user XKB directory:

```sh
mkdir -p ~/.config/xkb/symbols
cp xkb/ike ~/.config/xkb/symbols/ike
```

On **Wayland** (GNOME, KDE, sway), libxkbcommon searches `~/.config/xkb`
automatically — add/select the layout named `ike` in your usual keyboard
settings, then log out and back in if it doesn't appear immediately.

On **X11**, point the include path at that directory and compile it into the
running server:

```sh
setxkbmap -I"$HOME/.config/xkb" ike -print | xkbcomp -I"$HOME/.config/xkb" - "$DISPLAY"
```

Right-Alt is the level-3 switch (the file ends with
`include "level3(ralt_switch)"`), so `AltGr` + a key gives the long vowel and
`AltGr` + `Shift` + a key gives the long form of the shifted glyph.

> **Safe to use with this tutor:** the layout's base and Shift levels were
> checked key-by-key against `src/layout.rs` and match on **all 42 keys** the
> app models, so the course plays identically whether you run stock
> `ca(ike)` or this custom layout. The extra long vowels sit on a third level
> the app never asks for.

> **Note:** the tutor itself does not currently teach the long-vowel layer —
> `src/layout.rs` models two levels per key (base + Shift), matching the
> stock layout, and no lesson targets a long vowel. The custom layout is
> therefore only needed for *general* Inuktitut typing outside the app, not
> to complete the course.

### Windows

Windows ships a matching layout out of the box: **Inuktitut - Naqittaut**.
Verified glyph-for-glyph against this app's key table — same base and Shift
mappings, including punctuation. Enable it once via *Settings → Time &
Language → Language & region → (your language) → Add a keyboard →
Inuktitut - Naqittaut*, then switch to it from the language/keyboard
indicator in the taskbar.

**Important difference from Linux:** Naqittaut is a dual-mode layout —
**Caps Lock toggles between normal Latin typing and the syllabics layer.**
Turn Caps Lock **on** before you start practicing, or every keypress will
produce a Latin letter instead of a syllabic (the app will show "layout not
active?" in the footer if this happens — that's the fix). Regular Shift still
works as expected for the app's Shift-layer lessons once Caps Lock is on.

**Long vowels work out of the box.** With Caps Lock on, `AltGr` (right Alt)
+ a key gives that key's long vowel — `AltGr`+`S` → ᐆ, `AltGr`+`F` → ᑰ, and
so on — and `AltGr`+`Shift`+ a key gives the long form of the shifted glyph.
No custom layout needed; this is confirmed against Microsoft's published
Naqittaut key chart, which lists the full OO-series on the AltGr level.
(Linux needs [a bundled custom layout](#long-vowels-built-in-on-windowsmacos-custom-layout-on-linux)
for the same thing.)

### macOS

macOS ships a built-in Inuktitut input source too, though named differently
than Windows' ("Inuktitut" / "Inuktitut – Nunavut" rather than "Naqittaut").
Enable it via *System Settings → Keyboard → Input Sources → Add → Inuktitut*,
then switch to it from the input-source menu in the menu bar. As on Windows,
turn **Caps Lock on** before practicing.

Unlike the Windows layout, this one hasn't been directly verified against the
app's key table from a published reference — before relying on it for real
practice, do a quick sanity check:

1. Open the input source's **Keyboard Viewer** (available from the
   input-source menu, or *System Settings → Keyboard → Show Input Sources in
   Menu Bar*, then Keyboard Viewer from that menu).
2. With Caps Lock on and the Inuktitut input source selected, press `F`, `J`,
   and `A` and confirm the Keyboard Viewer / a text field shows `ᑯ`, `ᒧ`, and
   `ᖑ` respectively (Step 1 and Step 5 in the app show the same glyphs for
   comparison).
3. If those match, you're good — the app's on-screen keyboard reflects
   reality. If they don't, stop and say so rather than practicing wrong key
   positions — the layout most likely differs enough to need a custom
   `.keylayout` file instead of the built-in one.
4. *(Optional, only if you want long vowels for general typing — the course
   never needs them.)* Still with Caps Lock on, press `Option` (⌥) + `S` and
   see whether `ᐆ` appears.

**Long vowels — expected to work, but unconfirmed.** macOS uses `Option` as
its level-3 modifier, so `Option` + a key *should* produce that key's long
vowel (`Option`+`S` → ᐆ), the same way `AltGr` does on Windows. **Nobody has
tested this on an actual Mac yet**, so treat it as expected rather than
established. Step 4 above is the check. If a plain `ᐅ` appears instead, this
layout most likely forms long vowels by pressing the vowel twice rather than
via `Option` — worth reporting back so this section can be corrected.

### Controls

- Type along with the exercise line to advance it.
- `[` / `]` or `Left` / `Right` — previous / next step
- `r` — restart the current step
- `h` — toggle the key-label captions on the exercise cells
- `Ctrl+S` — save progress and quit
- `q` / `Esc` — quit without saving

## Progress and the completion screen

Typing the very last cell of Step 14 correctly replaces the normal panels
with a course-complete recap screen: every step's title alongside the
wpm/accuracy it last recorded (or "not completed" for any step you skipped
past with `]` rather than finishing). From there `r` restarts Step 14,
`[` / `]` go back to browse any step, and `Ctrl+S` / `q` save/quit as usual.

`Ctrl+S` writes which steps are done, their last stats, and which step you
were on to a small progress file in your platform's standard per-user data
directory: `$XDG_DATA_HOME/inuktitut-tutor/progress.txt` (falling back to
`~/.local/share/...`) on Linux, `%APPDATA%\inuktitut-tutor\progress.txt` on
Windows, and `~/Library/Application Support/inuktitut-tutor/progress.txt` on
macOS. The next run reads that file back and resumes on the same step with
earlier steps still marked done — `q` alone quits without touching this
file, so you can back out of an in-progress attempt without saving it.

## Why an OS keyboard layout is required at all

This is a terminal app, so on every platform it can only see the *character*
your OS keyboard layout produces for a keypress (via crossterm) — never the
physical scancode. Every layout above is a direct 1-key = 1-syllabic mapping
(no dead keys or composition on Linux/Windows), so the app just compares the
incoming character against the target syllabic. This also covers the Shift
layer for free: `Shift+W` simply produces `ᐱ` as a single character.

If the layout isn't active (or, on Windows/macOS, Caps Lock isn't on), keys
will produce plain Latin letters instead of syllabics; the app detects this
and shows a hint in the footer rather than guessing at a translation.

The static keymap bundled in the app (`src/layout.rs`) is used purely for
*display* — drawing the on-screen keyboard and looking up which key/finger to
use for the current target glyph. It never participates in input matching.

## Font requirements

Your terminal font needs to cover Unified Canadian Aboriginal Syllabics
(U+1400–U+167F), or the syllabics will render as tofu/boxes.

- **Linux**: [Noto Sans Canadian Aboriginal](https://fonts.google.com/noto/specimen/Noto+Sans+Canadian+Aboriginal),
  or DejaVu Sans (bundled on most distros) as a fallback.
- **Windows**: install Noto Sans Canadian Aboriginal and set it as the font
  in Windows Terminal's profile settings — the default Cascadia font does
  not cover this Unicode block.
- **macOS**: Terminal.app and iTerm2 both fall back to Apple's system fonts,
  which already include Canadian Aboriginal Syllabics glyphs, so this
  usually works with no extra installation — if glyphs render as boxes
  anyway, install Noto Sans Canadian Aboriginal and pick it explicitly in
  the terminal's profile/font settings.

## Project layout

- `src/main.rs` — terminal setup/teardown and the event loop
- `src/app.rs` — state machine, input handling, and live stats
- `src/ui.rs` — rendering (keyboard, exercise strip, header, footer)
- `src/layout.rs` — the `ca(ike)` key table, `Finger` enum, and the
  glyph → key/finger reverse lookup used for display
- `src/lessons.rs` — the fourteen lesson steps, authored in key notation and
  translated to target glyph sequences at load time
- `src/save.rs` — plain-text progress persistence (`Ctrl+S`)
- `xkb/ike` — optional custom Linux XKB layout adding a right-Alt level for
  long vowels (see [Long vowels](#long-vowels-built-in-on-windowsmacos-custom-layout-on-linux))
