//! The fourteen graded lesson steps. Lines are authored in *key notation*
//! (see `layout::translate_line`) and translated into the glyph sequence at
//! load time -- the key-notation strings below are the single source of
//! truth.
//!
//! Line-length policy: keep every line at or under 36 key-notation
//! characters (including spaces). At the app's `MIN_WIDTH` floor the
//! exercise strip wraps at 12 cells per row and the terminal-height gate is
//! sized for at most 3 wrapped rows -- see `ui.rs`'s `KEYBOARD_HEIGHT`/
//! `exercise_row_count` derivation. A longer line still renders, just by
//! requiring a taller terminal than the stated minimum.

use crate::layout::translate_line;

/// One cell of an exercise line: the key label shown on top, and the target
/// syllabic (or space) the learner must produce underneath.
#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub key_label: char,
    pub glyph: char,
}

pub struct Lesson {
    pub title: &'static str,
    pub hint: &'static str,
    pub lines: Vec<Vec<Cell>>,
}

impl Lesson {
    fn new(title: &'static str, hint: &'static str, raw_lines: &[&'static str]) -> Self {
        let lines = raw_lines
            .iter()
            .map(|line| {
                let cells: Vec<Cell> = translate_line(line)
                    .into_iter()
                    .map(|(key_label, glyph)| Cell { key_label, glyph })
                    .collect();
                debug_assert_eq!(
                    cells.len(),
                    line.chars().count(),
                    "lesson {title:?}: line {line:?} dropped a character -- \
                     an unrecognized key-notation char (e.g. a stray '?' or \
                     an uppercase letter with no shift glyph) is silently \
                     filtered out by translate_line instead of erroring",
                );
                cells
            })
            .collect();
        Lesson { title, hint, lines }
    }

    /// Total number of typed cells across every line of this lesson.
    pub fn total_chars(&self) -> usize {
        self.lines.iter().map(Vec::len).sum()
    }
}

pub fn all_lessons() -> Vec<Lesson> {
    vec![
        Lesson::new(
            "STEP 1 — INDEX FINGERS",
            "F J · the home-row anchors",
            &[
                "ffff jjjj ffff jjjj",
                "ffjj jjff fjfj jfjf",
                "fff jjj fjf jfj ffjj",
                "ffjj jjff fjfj jfjf ffjj jjff",
                "fjfj jfjf ffjj jjff fjjf jffj",
            ],
        ),
        Lesson::new(
            "STEP 2 — INDEX REACHES",
            "G H · reach in from F and J",
            &[
                "gggg hhhh gghh hhgg",
                "fghj jhgf fgfg hjhj",
                "gh gh hg hg gghh hhgg ghgh hghg",
                "fghj jhgf ghfj jfhg fgjh hjfg",
            ],
        ),
        Lesson::new(
            "STEP 3 — MIDDLE FINGERS",
            "D K",
            &[
                "dddd kkkk ddkk kkdd",
                "dkdk fjdk kjfd ddfk",
                "dk dk kd kd ddkk kkdd dkdk kdkd",
                "fjdk kjfd dkfj jkdf fdjk kdjf",
            ],
        ),
        Lesson::new(
            "STEP 4 — RING FINGERS",
            "S L",
            &[
                "ssss llll ssll llss",
                "slsl dksl asdf jkl",
                "sl sl ls ls ssll llss slsl lsls",
                "dksl slkd asdf jkl fjsl lskj",
            ],
        ),
        Lesson::new(
            "STEP 5 — WHOLE HOME ROW",
            "the u-series · a s d f g h j k l",
            &[
                "aaaa asdf ghjkl",
                "asdfghjkl",
                "lkjhgfdsa",
                "asdf jkl asdf jkl ghjk fdsa",
                "asdfghjkl lkjhgfdsa asdfghjkl",
                "fghj jhgf asdfjkl lkjfdsagh",
            ],
        ),
        Lesson::new(
            "STEP 6 — HOME ROW SHIFT",
            "hold Shift · the u-series again, one octave up",
            &[
                "FF JJ FF JJ FJFJ JFJF",
                "AA SS DD FF JJ KK LL",
                "ASDF JKL ASDF JKL",
                "FJ FJ FJ AS DK JL",
            ],
        ),
        Lesson::new(
            "STEP 7 — TOP ROW (i-series)",
            "reach up from home",
            &[
                "rrrr uuuu ffrr jjuu",
                "tttt yyyy eeee iiii",
                "wwww oooo qqqq pppp",
                "qwert yuiop",
            ],
        ),
        Lesson::new(
            "STEP 8 — BOTTOM ROW (a-series)",
            "reach down from home",
            &[
                "vvvv mmmm ffvv jjmm",
                "bbbb nnnn cccc xxxx",
                "zzzz zxcvb nm",
                "zxcvbnm",
            ],
        ),
        Lesson::new(
            "STEP 9 — NUMBER ROW (finals)",
            "naniit · these end syllables & words · = is ᕝ",
            &[
                "4444 7777 2222 5555",
                "6666 8888 1111 3333 9999 0000",
                // ᕝ (F final) also lives on this row, two keys right of 0.
                "==== ==== ==== ====",
                "4= 7= 2= 5= 6= 8= 1=",
            ],
        ),
        Lesson::new(
            "STEP 10 — PUNCTUATION",
            "comma · period · Shift+V = ?",
            &[
                ",,,, .... ,,,, ....",
                ",. ., ,. ., ,,.. ..,,",
                "vv VV vv VV vVVv VvvV",
                "zx, cv. bn, m.",
                "v, V. v, V. vV Vv",
            ],
        ),
        Lesson::new(
            "STEP 11 — COLUMNS = CONSONANTS",
            "three vowels i / u / a per consonant · last row needs Shift",
            &[
                // Seven consonants really are one finger, three rows.
                "qaz wsx edc rfv",
                "tgb yhn ujm",
                // l, n and y break the pattern: their third vowel sits on
                // the Shift layer of a right-index key, not a third key in
                // the same column. Drilled together so the series still
                // completes -- ᓕᓗᓚ, ᓂᓄᓇ, ᔨᔪᔭ.
                "olM ikN pJ/",
                "olM olM ikN ikN pJ/ pJ/",
            ],
        ),
        Lesson::new(
            "STEP 12 — REAL WORDS",
            "all on base keys — no Shift",
            &[
                "wk4 wkw5 w[l yf",
                "wu6 vu4 c/6 sux6",
                "xuh5 wclw5 tr wu6",
            ],
        ),
        Lesson::new(
            "STEP 13 — SHIFT LAYER",
            "hold Shift · W/S/X · Q/A/Z · E/D/C · F/K/|",
            &["WSX WSX", "QAZ QAZ", "EDC EDC", "FK| FK|"],
        ),
        // Everything on the Shift layer the earlier steps never reach: the
        // ᙱᙳᙵᖖ (nng) and ᖠᖢᖤᖦ (lh) series, plus the ᔅ and ᕼ finals. With
        // this step the course targets every glyph in `layout::KEYS` --
        // enforced by `every_layout_glyph_is_drilled` below.
        Lesson::new(
            "STEP 14 — THE REST OF THE SHIFT LAYER",
            "ᙱᙳᙵᖖ nng · ᖠᖢᖤᖦ lh · ᔅ and ᕼ finals",
            &[
                "TT YY UU RR",
                "TYU TYU RRRR",
                "OO LL II PP",
                "OLI OLI PPPP",
                "{{ BB {{ BB",
                "TYUR OLIP {B",
            ],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::locate_glyph;

    fn glyphs(cells: &[Cell]) -> String {
        cells.iter().map(|c| c.glyph).collect()
    }

    fn lesson_by_title_prefix<'a>(lessons: &'a [Lesson], prefix: &str) -> &'a Lesson {
        lessons
            .iter()
            .find(|l| l.title.starts_with(prefix))
            .unwrap_or_else(|| panic!("no lesson titled {prefix:?}"))
    }

    #[test]
    fn step1_line1_is_ku_mu() {
        let lessons = all_lessons();
        let line = &lesson_by_title_prefix(&lessons, "STEP 1 —").lines[0];
        assert_eq!(glyphs(line), "ᑯᑯᑯᑯ ᒧᒧᒧᒧ ᑯᑯᑯᑯ ᒧᒧᒧᒧ");
        let labels: String = line.iter().map(|c| c.key_label).collect();
        assert_eq!(labels, "ffff jjjj ffff jjjj");
    }

    #[test]
    fn home_row_is_u_series() {
        let lessons = all_lessons();
        let line = &lesson_by_title_prefix(&lessons, "STEP 5 —").lines[1]; // "asdfghjkl"
        assert_eq!(glyphs(line), "ᖑᐅᖁᑯᑐᓱᒧᓄᓗ");
    }

    #[test]
    fn step13_line1_targets_shift_glyphs() {
        let lessons = all_lessons();
        let line = &lesson_by_title_prefix(&lessons, "STEP 13 —").lines[0]; // "WSX WSX"
        assert_eq!(glyphs(line), "ᐱᐳᐸ ᐱᐳᐸ");
        for cell in line {
            if cell.glyph == ' ' {
                continue;
            }
            let loc = locate_glyph(cell.glyph).expect("glyph should be on the keyboard");
            assert!(loc.shifted, "expected {:?} to require Shift", cell.glyph);
        }
    }

    #[test]
    fn punctuation_lesson_targets_real_keys() {
        let lessons = all_lessons();
        let lesson = lesson_by_title_prefix(&lessons, "STEP 10 —");
        // Line 1 (",,,, .... ,,,, ....") is a literal comma/period drill.
        assert_eq!(glyphs(&lesson.lines[0]), ",,,, .... ,,,, ....");
        // Line 3 ("vv VV vv VV vVVv VvvV") alternates ᑲ (v) and literal ? (V).
        assert_eq!(glyphs(&lesson.lines[2]), "ᑲᑲ ?? ᑲᑲ ?? ᑲ??ᑲ ?ᑲᑲ?");
    }

    #[test]
    fn all_lesson_glyphs_are_locatable_or_space() {
        for lesson in all_lessons() {
            for line in &lesson.lines {
                for cell in line {
                    if cell.glyph != ' ' {
                        assert!(
                            locate_glyph(cell.glyph).is_some(),
                            "unlocatable glyph {:?} in {}",
                            cell.glyph,
                            lesson.title
                        );
                    }
                }
            }
        }
    }

    /// Every *syllabic* the layout can produce -- base or Shift, any key --
    /// must be targeted by at least one lesson. Plain-ASCII levels are exempt
    /// (`-`/`_`, Shift+`=`, and the punctuation keys' own characters): they
    /// carry no syllabic to teach. Without this, glyphs silently go untaught
    /// -- ᓇ (Shift+N) and ᓚ (Shift+M) were missing for exactly this reason,
    /// and ᕝ was unreachable because its key wasn't modelled at all.
    #[test]
    fn every_syllabic_in_the_layout_is_drilled() {
        use crate::layout::KEYS;

        // Unified Canadian Aboriginal Syllabics.
        fn is_syllabic(c: char) -> bool {
            ('\u{1400}'..='\u{167f}').contains(&c)
        }

        let drilled: std::collections::HashSet<char> = all_lessons()
            .iter()
            .flat_map(|l| l.lines.iter())
            .flatten()
            .map(|c| c.glyph)
            .collect();

        let mut missing = Vec::new();
        for &(key, base, shift, ..) in KEYS {
            if is_syllabic(base) && !drilled.contains(&base) {
                missing.push(format!("{base} (base of {key:?})"));
            }
            if let Some(s) = shift {
                if is_syllabic(s) && !drilled.contains(&s) {
                    missing.push(format!("{s} (Shift+{})", key.to_ascii_uppercase()));
                }
            }
        }
        assert!(
            missing.is_empty(),
            "{} syllabic(s) never appear in any lesson: {}",
            missing.len(),
            missing.join(", ")
        );
    }

    /// See the module-level line-length policy comment: lines longer than
    /// this force a taller-than-`MIN_WIDTH` terminal to avoid wrapping past
    /// what the keyboard/footer height math assumes.
    #[test]
    fn no_line_exceeds_the_authoring_budget() {
        const MAX_LINE_LEN: usize = 36;
        for lesson in all_lessons() {
            for line in &lesson.lines {
                assert!(
                    line.len() <= MAX_LINE_LEN,
                    "{}: a line has {} cells, over the {MAX_LINE_LEN}-char budget",
                    lesson.title,
                    line.len()
                );
            }
        }
    }
}
