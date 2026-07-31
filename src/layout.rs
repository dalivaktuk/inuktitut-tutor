//! The `ca(ike)` keyboard layout: physical keys, their base/shift syllabics,
//! which finger types them, and whether they sit on the home row.
//!
//! This table is the single source of truth. Both the on-screen keyboard and
//! the lesson-line translation (key notation -> glyph sequence) derive from it.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finger {
    LPinky,
    LRing,
    LMiddle,
    LIndex,
    RIndex,
    RMiddle,
    RRing,
    RPinky,
}

impl Finger {
    pub fn label(self) -> &'static str {
        match self {
            Finger::LPinky => "left pinky",
            Finger::LRing => "left ring",
            Finger::LMiddle => "left middle",
            Finger::LIndex => "left index",
            Finger::RIndex => "right index",
            Finger::RMiddle => "right middle",
            Finger::RRing => "right ring",
            Finger::RPinky => "right pinky",
        }
    }
}

/// One physical key: `(key, base_glyph, shift_glyph, finger, is_home_row)`.
pub type KeyDef = (char, char, Option<char>, Finger, bool);

// Rows in physical order: number, top (QWERTY + [), home (A..L),
// bottom (Z..M, comma, period, /). The home row (is_home = true) is the
// whole u-series. Comma/period are plain unshifted ASCII on the real
// ca(ike) layout (verified against xkeyboard-config's `ike` symbols) --
// not syllabics -- so their shift slot is None and their base glyph is
// just themselves.
pub const KEYS: &[KeyDef] = &[
    ('1', '\u{1595}', None, Finger::LPinky, false), // ᖕ
    ('2', '\u{1449}', None, Finger::LRing, false),  // ᑉ
    ('3', '\u{1550}', None, Finger::LMiddle, false), // ᕐ
    ('4', '\u{1483}', None, Finger::LIndex, false), // ᒃ
    ('5', '\u{1466}', None, Finger::LIndex, false), // ᑦ
    ('6', '\u{1585}', None, Finger::RIndex, false), // ᖅ
    ('7', '\u{14bb}', None, Finger::RIndex, false), // ᒻ
    ('8', '\u{14d0}', None, Finger::RMiddle, false), // ᓐ
    ('9', '\u{14ea}', None, Finger::RRing, false),  // ᓪ
    ('0', '\u{153e}', None, Finger::RPinky, false), // ᔾ
    ('q', '\u{158f}', Some('\u{148b}'), Finger::LPinky, false), // ᖏ  ⇧ᒋ
    ('w', '\u{1403}', Some('\u{1431}'), Finger::LRing, false), // ᐃ  ⇧ᐱ
    ('e', '\u{157f}', Some('\u{1546}'), Finger::LMiddle, false), // ᕿ  ⇧ᕆ
    ('r', '\u{146d}', Some('\u{1596}'), Finger::LIndex, false), // ᑭ  ⇧ᖖ
    ('t', '\u{144e}', Some('\u{1671}'), Finger::LIndex, false), // ᑎ  ⇧ᙱ
    ('y', '\u{14ef}', Some('\u{1673}'), Finger::RIndex, false), // ᓯ  ⇧ᙳ
    ('u', '\u{14a5}', Some('\u{1675}'), Finger::RIndex, false), // ᒥ  ⇧ᙵ
    ('i', '\u{14c2}', Some('\u{15a4}'), Finger::RMiddle, false), // ᓂ  ⇧ᖤ
    ('o', '\u{14d5}', Some('\u{15a0}'), Finger::RRing, false), // ᓕ  ⇧ᖠ
    ('p', '\u{1528}', Some('\u{15a6}'), Finger::RPinky, false), // ᔨ  ⇧ᖦ
    ('[', '\u{14a1}', Some('\u{1505}'), Finger::RPinky, false), // ᒡ  ⇧ᔅ
    ('a', '\u{1591}', Some('\u{148d}'), Finger::LPinky, true), // ᖑ  ⇧ᒍ
    ('s', '\u{1405}', Some('\u{1433}'), Finger::LRing, true), // ᐅ  ⇧ᐳ
    ('d', '\u{1581}', Some('\u{1548}'), Finger::LMiddle, true), // ᖁ  ⇧ᕈ
    ('f', '\u{146f}', Some('\u{1555}'), Finger::LIndex, true), // ᑯ  ⇧ᕕ
    ('g', '\u{1450}', None, Finger::LIndex, true),  // ᑐ
    ('h', '\u{14f1}', None, Finger::RIndex, true),  // ᓱ
    ('j', '\u{14a7}', Some('\u{152a}'), Finger::RIndex, true), // ᒧ  ⇧ᔪ
    ('k', '\u{14c4}', Some('\u{1557}'), Finger::RMiddle, true), // ᓄ  ⇧ᕗ
    ('l', '\u{14d7}', Some('\u{15a2}'), Finger::RRing, true), // ᓗ  ⇧ᖢ
    ('z', '\u{1593}', Some('\u{1490}'), Finger::LPinky, false), // ᖓ  ⇧ᒐ
    ('x', '\u{140a}', Some('\u{1438}'), Finger::LRing, false), // ᐊ  ⇧ᐸ
    ('c', '\u{1583}', Some('\u{154b}'), Finger::LMiddle, false), // ᖃ  ⇧ᕋ
    ('v', '\u{1472}', Some('?'), Finger::LIndex, false), // ᑲ  ⇧?
    ('b', '\u{1455}', Some('\u{157c}'), Finger::LIndex, false), // ᑕ  ⇧ᕼ
    ('n', '\u{14f4}', Some('\u{14c7}'), Finger::RIndex, false), // ᓴ  ⇧ᓇ
    ('m', '\u{14aa}', Some('\u{14da}'), Finger::RIndex, false), // ᒪ  ⇧ᓚ
    (',', ',', None, Finger::RMiddle, false), // literal comma, no shift drilled
    ('.', '.', None, Finger::RRing, false), // literal period, no shift drilled
    ('/', '\u{152d}', Some('\u{1559}'), Finger::RPinky, false), // ᔭ  ⇧ᕙ
];

/// Where a glyph lives on the keyboard: which key, whether Shift is needed,
/// and which finger to use.
#[derive(Debug, Clone, Copy)]
pub struct GlyphLocation {
    pub key: char,
    pub shifted: bool,
    pub finger: Finger,
}

/// Reverse lookup: given a target syllabic (or space), find which key produces
/// it and whether Shift is required. Used only for display (keyboard
/// highlighting, finger hints) -- never for input matching.
pub fn locate_glyph(glyph: char) -> Option<GlyphLocation> {
    if glyph == ' ' {
        return None;
    }
    for &(key, base, shift, finger, _) in KEYS {
        if base == glyph {
            return Some(GlyphLocation {
                key,
                shifted: false,
                finger,
            });
        }
        if shift == Some(glyph) {
            return Some(GlyphLocation {
                key,
                shifted: true,
                finger,
            });
        }
    }
    None
}

fn key_def(key: char) -> Option<KeyDef> {
    KEYS.iter().copied().find(|&(k, ..)| k == key)
}

/// Translate one character of *key notation* into the glyph it produces, and
/// whether pressing it requires Shift (display only):
///
/// - lowercase letter / digit / `[` / `/` / `,` / `.` -> that key's base glyph
/// - uppercase letter -> that key's shift glyph (this is how a real `?`,
///   Shift+V, is authored -- write `V`)
/// - `|` -> Shift + `/` (the one shifted-punctuation alias needed, since `/`
///   has no uppercase form to piggyback on the rule above)
/// - space -> a literal space
pub fn key_notation_to_glyph(c: char) -> Option<(char, bool)> {
    if c == ' ' {
        return Some((' ', false));
    }
    if c == '|' {
        let (_, _, shift, ..) = key_def('/')?;
        return Some((shift?, true));
    }
    if c.is_ascii_uppercase() {
        let lower = c.to_ascii_lowercase();
        let (_, _, shift, ..) = key_def(lower)?;
        return Some((shift?, true));
    }
    let (_, base, ..) = key_def(c)?;
    Some((base, false))
}

/// Translate a whole line written in key notation into `(key_label,
/// target_glyph)` pairs, one per cell the learner will see: the key label as
/// written (case preserved, so Shift needs are visible) alongside the glyph
/// it produces.
pub fn translate_line(line: &str) -> Vec<(char, char)> {
    line.chars()
        .filter_map(|label| key_notation_to_glyph(label).map(|(glyph, _)| (label, glyph)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn question_mark_is_retired_as_a_notation_alias() {
        // There is no physical key literally labeled '?' -- it's reached via
        // Shift+V, authored as uppercase 'V'.
        assert_eq!(key_notation_to_glyph('?'), None);
    }

    #[test]
    fn pipe_is_the_shift_slash_alias() {
        assert_eq!(key_notation_to_glyph('|'), Some(('\u{1559}', true)));
    }

    #[test]
    fn shift_v_produces_a_real_question_mark() {
        assert_eq!(key_notation_to_glyph('V'), Some(('?', true)));
        let loc = locate_glyph('?').expect("? should be locatable");
        assert_eq!(loc.key, 'v');
        assert!(loc.shifted);
    }

    #[test]
    fn comma_and_period_are_plain_ascii_passthrough() {
        assert_eq!(key_notation_to_glyph(','), Some((',', false)));
        assert_eq!(key_notation_to_glyph('.'), Some(('.', false)));
    }

    #[test]
    fn punctuation_line_round_trips_with_no_dropped_characters() {
        let line = ",.V ,.V";
        let translated = translate_line(line);
        assert_eq!(translated.len(), line.chars().count());
        let glyphs: String = translated.iter().map(|(_, g)| *g).collect();
        assert_eq!(glyphs, ",.? ,.?");
    }
}
