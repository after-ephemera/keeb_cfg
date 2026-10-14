# Evaluation of the Lily58 KMK config

Reviewed: `lily58/main.py` (keymap + modules) and `lily58/kb.py` (matrix/pins).
Overall this is a solid, working KMK config with some genuinely nice ergonomic
tricks. Below are the things worth your attention, ordered by severity.

## 🔴 Likely hardware-pin conflicts in `kb.py`

`GP29` is assigned three jobs at once:

```python
RotaryioEncoder(pin_a=board.GP28, pin_b=board.GP29, ...)  # encoder B
rgb_pixel_pin = board.GP29                                # RGB data (line 40)
encoder_b     = board.GP29                                # encoder B (line 45)
```

`rgb_pixel_pin` is also set twice — first to `board.TX` (line 37), then
overwritten to `board.GP29` (line 40) — and `i2c = board.I2C` is set twice.
A single GPIO can't simultaneously drive the WS2812 data line and read encoder
phase B. If your RGB or encoder behaves erratically, this is why. Pick one
function per pin (the stock Lily58 wires the underglow to a dedicated pin, not
the encoder pin).

## 🟠 Encoder is scanned but never mapped

`kb.py` registers a `RotaryioEncoder` in the scanner list, but `main.py` never
appends the `Encoder` module and no keymap positions correspond to encoder
events. Rotation currently does nothing useful. Either add
`from kmk.modules.encoder import EncoderHandler`, configure `encoder.map`, or
drop the encoder scanner if you're not using the knob.

## 🟠 `combos` module loaded but no combos defined

`Combos()` is appended to `keyboard.modules` but `keyboard.combos` is never set.
It's harmless but pure overhead — either define combos (a great fit for the
brackets/parens you currently spend keys on) or remove the module.

## 🟡 Case-sensitive import that only works by luck

```python
from kmk.extensions.peg_oled_Display import ...
```

The file in the repo is `peg_oled_display.py` (lowercase `d`). This resolves on
case-insensitive filesystems (macOS default, and the FAT CIRCUITPY volume), but
will `ImportError` on a case-sensitive one. Match the real filename.

## 🟡 Raise layer has leftover noise

```python
[ # raise
  KC.N2, KC.EXLM, KC.AT, ...        # top-left starts on "2" — looks accidental
  ...
  KC.N0, ..., KC.N1, KC.N2, KC.N3, KC.DOT, KC.N3   # trailing KC.N3 duplicated
]
```

The top-left `KC.N2` and the duplicated trailing `KC.N3` look like stray edits
rather than intent. Worth a pass to make them `KC.TRNS` or something deliberate.

## 🟡 OLED advertises layers that don't exist

The OLED `corner_three`/`corner_four` name layers `adjust`, `special`, etc., but
layers 3–7 are empty `KC.TRNS` placeholders. The display will show a name for a
layer you can never actually be on (nothing switches to 3+). Harmless, just
misleading. Trim the OLED tables to the 3 real layers or fill in the layers.

## 🟢 Nice things you're doing well

- **Layered tap-dance / hold-tap.** `LGUIPP`, `LCTLPP`, `F`, `LBRC/RBRC` are
  clever — tap-for-symbol / hold-for-mod / double-tap-for-app-switch packs a lot
  into few keys.
- **`prefer_hold=False` on the `F` cmd key** (per your commit history) is the
  right call for a home-key hold-tap to avoid accidental holds while typing.
- **Sticky GUI** (`KC.SK`) on a thumb is a good ergonomic choice.
- **Consistent 200 ms** `tap_time` across hold-tap and tap-dance keeps behavior
  predictable.

## Minor consistency nits

- `F` is a tap-dance wrapping a hold-tap (`⌘`/app-switch) while `J` is a plain
  `KC.HT(KC.J, KC.RGUI)`. If you want symmetric home-key mods, consider making
  them mirror each other.
- `LOWER`/`RAISE` use `KC.TO()` (switch-and-stay) with a `BASE = KC.TO(0)` to get
  back — a deliberate "sticky layers" design (matches your commit message). It
  works; just note that a mispress strands you until you hit `BASE`. Momentary
  `KC.MO()` is the common alternative if you ever find that annoying.
- `diode_orientation = DiodeOrientation.COLUMNS` (class attr) vs.
  `columns_to_anodes=DiodeOrientation.COL2ROW` (scanner arg) are redundant; the
  scanner arg is what's used.

None of the 🟢/🟡 items block anything — the 🔴 pin conflict is the one I'd fix
first if RGB/encoder ever misbehave.
