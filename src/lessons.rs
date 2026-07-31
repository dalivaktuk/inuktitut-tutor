//! The thirteen graded lesson steps. Lines are authored in *key notation*
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
            "naniit · these end syllables & words",
            &["4444 7777 2222 5555", "6666 8888 1111 3333 9999 0000"],
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
            "same finger, three vowels: i / u / a",
            &["qaz wsx edc rfv", "tgb yhn ujm ol", "ik p/"],
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
