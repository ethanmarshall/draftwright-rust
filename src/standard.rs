//! The drawing standard a sheet is drawn to: ISO (the default, millimetres) or ASME Y14.5
//! (inches). `sheet.standard` in the Spec selects it; [`scope`] makes it current for planning,
//! rendering and review on this thread, the same way [`crate::layout::with_gutter`] scopes the
//! frame gutter.
//!
//! Under ASME the part is still read and planned in millimetres and the paper is still laid out
//! in millimetres. Only what is printed changes:
//! - every length a label prints is converted to inches and written ASME style: three decimals,
//!   no leading zero (`.250`, `1.500`) ([`crate::nominal::format`]);
//! - angles print as before ([`crate::nominal::format_angle`]);
//! - paper is ANSI A to E and the scale series is the inch one (4:1, 2:1, 1:1, 1:2, 1:4, 1:8)
//!   ([`crate::layout::pages`], [`crate::layout::scales`]). Internally the five sheets keep the
//!   ISO names `A4` … `A0` as ranks; [`sheet_name`] prints the ANSI name;
//! - the title block states inches and, when no general tolerance is given, the usual ASME
//!   block tolerance;
//! - modelled threads are named as Unified (UNC/UNF) threads when one matches
//!   ([`unified_thread`]).

use std::cell::Cell;

thread_local! {
    static ASME: Cell<bool> = const { Cell::new(false) };
    static DUAL: Cell<bool> = const { Cell::new(false) };
}

/// The Spec's standards (`sheet/standard`).
pub const STANDARDS: [&str; 2] = ["iso", "asme"];

/// Whether the current sheet is drawn to ASME Y14.5 in inches.
pub fn asme() -> bool {
    ASME.get()
}

/// Whether an ASME sheet prints each length's millimetres after its inches (`.250 [6.35]`).
pub fn dual() -> bool {
    ASME.get() && DUAL.get()
}

/// Restores the previous standard (and dual dimensioning) when dropped.
pub struct Guard(bool, bool);

impl Drop for Guard {
    fn drop(&mut self) {
        ASME.set(self.0);
        DUAL.set(self.1);
    }
}

/// Makes *spec*'s standard current until the guard is dropped.
pub fn scope(spec: &crate::model::Spec) -> Guard {
    let guard = set(spec.sheet.standard.as_deref() == Some("asme"));
    DUAL.set(spec.sheet.dual_dimensions == Some(true));
    guard
}

/// Makes ASME current (true) or ISO (false) until the guard is dropped; dual dimensioning is
/// left as it is.
pub fn set(asme: bool) -> Guard {
    Guard(ASME.replace(asme), DUAL.get())
}

/// Runs *run* with *spec*'s standard current.
pub fn with<T>(spec: &crate::model::Spec, run: impl FnOnce() -> T) -> T {
    let _guard = scope(spec);
    run()
}

/// Millimetres per inch.
pub const MM_PER_INCH: f64 = 25.4;

/// ANSI Y14.1 sheets, smallest first, landscape, mm, under the internal rank names.
pub const ANSI_PAGES: [(&str, f64, f64); 5] = [
    ("A4", 279.4, 215.9),
    ("A3", 431.8, 279.4),
    ("A2", 558.8, 431.8),
    ("A1", 863.6, 558.8),
    ("A0", 1117.6, 863.6),
];

/// The printed name of the sheet of internal rank *name* (`A3` is `ANSI B` under ASME).
pub fn sheet_name(name: &str) -> String {
    if !asme() {
        return name.into();
    }
    match name {
        "A4" => "ANSI A",
        "A3" => "ANSI B",
        "A2" => "ANSI C",
        "A1" => "ANSI D",
        "A0" => "ANSI E",
        other => other,
    }
    .into()
}

/// The internal rank of a sheet name as a Spec may give it: `A4` … `A0`, or ANSI `A` … `E`
/// (also `ANSI B`, `B-size`). None when it is neither.
pub fn sheet_rank(name: &str) -> Option<&'static str> {
    let n = name.trim().to_ascii_uppercase();
    let n = n
        .trim_start_matches("ANSI")
        .trim()
        .trim_end_matches("-SIZE")
        .trim_end_matches(" SIZE")
        .trim();
    Some(match n {
        "A4" | "A" => "A4",
        "A3" | "B" => "A3",
        "A2" | "C" => "A2",
        "A1" | "D" => "A1",
        "A0" | "E" => "A0",
        _ => return None,
    })
}

/// ASME's usual block tolerance, for a title block given none.
pub const ASME_GENERAL_TOLERANCE: &str = ".XX ±.01  .XXX ±.005  ±0.5°";

/// The standard the drawing is interpreted to, as the title block states it.
pub const ASME_INTERPRET: &str = "ASME Y14.5-2018";

/// A length in mm, printed in inches ASME style: three decimals, no leading zero.
pub fn inch(mm: f64) -> String {
    let v = mm / MM_PER_INCH;
    let v = (v * 1e9).round() / 1e9;
    let text = format!("{v:.3}");
    let text = if let Some(rest) = text.strip_prefix("-0.") {
        format!("-.{rest}")
    } else if let Some(rest) = text.strip_prefix("0.") {
        format!(".{rest}")
    } else {
        text
    };
    if text == "-.000" { ".000".into() } else { text }
}

/// A length in mm in the unit the sheet prints: inches under ASME, else unchanged.
pub fn printed(mm: f64) -> f64 {
    if asme() { mm / MM_PER_INCH } else { mm }
}

/// A finding's text with its millimetre lengths restated in inches, for an ASME sheet's review.
/// A length is a decimal number (`1.524`, `-173.863`) or a number after a diameter or radius
/// sign (`ø8`, `R2`); an angle (a number before `°`), a count, an ID's digits (`f:hole:12`, `e43`)
/// and a scale (`1:2`) are left as they are. `mm` after a converted length becomes `in`. Under ISO
/// the text is returned unchanged.
pub fn inch_text(text: &str) -> String {
    if !asme() {
        return text.to_string();
    }
    let c: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < c.len() {
        let starts = c[i].is_ascii_digit()
            || (c[i] == '-' && c.get(i + 1).is_some_and(|d| d.is_ascii_digit()));
        // A radius sign is an `R` that starts its word (`R2.54`, not `e43`'s or `M8`'s letter).
        let radius = i > 0 && c[i - 1] == 'R' && (i == 1 || !c[i - 2].is_ascii_alphanumeric());
        let joined = i > 0
            && !radius
            && (c[i - 1].is_ascii_alphanumeric() || matches!(c[i - 1], ':' | '_' | '.' | '/'));
        if !starts || joined {
            out.push(c[i]);
            i += 1;
            continue;
        }
        let start = i;
        if c[i] == '-' {
            i += 1;
        }
        while i < c.len() && c[i].is_ascii_digit() {
            i += 1;
        }
        let mut decimal = false;
        if i + 1 < c.len() && c[i] == '.' && c[i + 1].is_ascii_digit() {
            decimal = true;
            i += 1;
            while i < c.len() && c[i].is_ascii_digit() {
                i += 1;
            }
        }
        let token: String = c[start..i].iter().collect();
        let sized = start > 0 && matches!(c[start - 1], 'ø' | 'Ø' | '⌀' | 'R');
        let angle = c.get(i) == Some(&'°');
        let ratio = c.get(i) == Some(&':') || (start > 0 && c[start - 1] == ':');
        let glued = c
            .get(i)
            .is_some_and(|n| n.is_ascii_alphabetic() || *n == '_');
        let length = (decimal || sized) && !angle && !ratio && !glued;
        match token.parse::<f64>() {
            Ok(mm) if length => {
                out += &inch(mm);
                // `0.305 mm deep` reads `.012 in deep`.
                let rest: String = c[i..c.len().min(i + 3)].iter().collect();
                if rest == " mm" && c.get(i + 3).is_none_or(|n| !n.is_ascii_alphanumeric()) {
                    out += " in";
                    i += 3;
                }
            }
            _ => out += &token,
        }
    }
    out
}

/// A limit deviation in mm, printed in inches to four decimals with its sign (`+.0006`), an
/// exact zero as `0`.
pub fn inch_deviation(mm: f64) -> String {
    let v = mm / MM_PER_INCH;
    if v.abs() < 5e-5 {
        return "0".into();
    }
    let text = format!("{v:+.4}");
    text.replacen("+0.", "+.", 1).replacen("-0.", "-.", 1)
}

/// Unified threads (ASME B1.1): designation, basic major diameter in inches, threads per inch.
/// Coarse (UNC) first, then fine (UNF), smallest first.
pub const UNIFIED: [(&str, f64, f64); 34] = [
    ("#2-56 UNC", 0.0860, 56.0),
    ("#4-40 UNC", 0.1120, 40.0),
    ("#5-40 UNC", 0.1250, 40.0),
    ("#6-32 UNC", 0.1380, 32.0),
    ("#8-32 UNC", 0.1640, 32.0),
    ("#10-24 UNC", 0.1900, 24.0),
    ("#12-24 UNC", 0.2160, 24.0),
    ("1/4-20 UNC", 0.2500, 20.0),
    ("5/16-18 UNC", 0.3125, 18.0),
    ("3/8-16 UNC", 0.3750, 16.0),
    ("7/16-14 UNC", 0.4375, 14.0),
    ("1/2-13 UNC", 0.5000, 13.0),
    ("9/16-12 UNC", 0.5625, 12.0),
    ("5/8-11 UNC", 0.6250, 11.0),
    ("3/4-10 UNC", 0.7500, 10.0),
    ("7/8-9 UNC", 0.8750, 9.0),
    ("1-8 UNC", 1.0000, 8.0),
    ("1-1/4-7 UNC", 1.2500, 7.0),
    ("1-1/2-6 UNC", 1.5000, 6.0),
    ("#4-48 UNF", 0.1120, 48.0),
    ("#6-40 UNF", 0.1380, 40.0),
    ("#8-36 UNF", 0.1640, 36.0),
    ("#10-32 UNF", 0.1900, 32.0),
    ("1/4-28 UNF", 0.2500, 28.0),
    ("5/16-24 UNF", 0.3125, 24.0),
    ("3/8-24 UNF", 0.3750, 24.0),
    ("7/16-20 UNF", 0.4375, 20.0),
    ("1/2-20 UNF", 0.5000, 20.0),
    ("9/16-18 UNF", 0.5625, 18.0),
    ("5/8-18 UNF", 0.6250, 18.0),
    ("3/4-16 UNF", 0.7500, 16.0),
    ("7/8-14 UNF", 0.8750, 14.0),
    ("1-12 UNF", 1.0000, 12.0),
    ("1-14 UNS", 1.0000, 14.0),
];

/// The Unified thread whose basic major diameter and pitch match *major_mm* and *pitch_mm*
/// (within 0.05 mm and 2%), or with no pitch the coarse thread of that diameter.
pub fn unified_thread(major_mm: f64, pitch_mm: Option<f64>) -> Option<&'static str> {
    UNIFIED
        .iter()
        .filter(|(_, d, _)| (d * MM_PER_INCH - major_mm).abs() <= 0.05)
        .find(|(_, _, tpi)| pitch_mm.is_none_or(|p| (MM_PER_INCH / tpi - p).abs() <= 0.02 * p))
        .map(|(name, _, _)| *name)
}

/// A Unified designation as a Spec may give it (`1/4-20 UNC-2B`, `#10-32 UNF`): its basic major
/// diameter and pitch, mm, and the class, if any. None for any other text.
pub fn parse_unified(text: &str) -> Option<(f64, f64, Option<String>)> {
    let t = text.trim().to_ascii_uppercase();
    let (size, rest) = t.split_once('-')?;
    let rest = rest.trim();
    let (tpi_text, tail) = rest.split_once(' ').unwrap_or((rest, ""));
    let tpi: f64 = tpi_text.parse().ok()?;
    let series_class = tail.trim();
    if !series_class.is_empty() && !series_class.starts_with("UN") {
        return None;
    }
    let class = series_class
        .split_once('-')
        .map(|(_, c)| c.trim().to_string())
        .filter(|c| !c.is_empty());
    let major = UNIFIED
        .iter()
        .find(|(n, _, t)| n.split('-').next() == Some(size) && (*t - tpi).abs() < 1e-9)
        .map(|(_, d, _)| *d)
        .or_else(|| fraction(size))?;
    Some((major * MM_PER_INCH, MM_PER_INCH / tpi, class))
}

/// `1/4`, `1-1/4`, `0.25` or `#10` as inches.
fn fraction(size: &str) -> Option<f64> {
    if let Some(n) = size.strip_prefix('#') {
        let n: f64 = n.parse().ok()?;
        return Some(0.060 + 0.013 * n);
    }
    if let Some((whole, frac)) = size.split_once(' ') {
        return Some(whole.parse::<f64>().ok()? + fraction(frac)?);
    }
    match size.split_once('/') {
        Some((a, b)) => Some(a.parse::<f64>().ok()? / b.parse::<f64>().ok()?),
        None => size.parse().ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inches_print_asme_style() {
        assert_eq!(inch(6.35), ".250");
        assert_eq!(inch(38.1), "1.500");
        assert_eq!(inch(-6.35), "-.250");
        assert_eq!(inch(0.0), ".000");
        assert_eq!(inch_deviation(0.0127), "+.0005");
        assert_eq!(inch_deviation(-0.0254), "-.0010");
        assert_eq!(inch_deviation(0.0), "0");
    }

    #[test]
    fn finding_text_restates_lengths_in_inches() {
        let _g = set(true);
        assert_eq!(
            inch_text("open channel f:section_recess:1 1.524 wide, at (-173.863, 0, 7.938)"),
            "open channel f:section_recess:1 .060 wide, at (-6.845, 0, .313)"
        );
        assert_eq!(
            inch_text("recess 0.305 mm deep (1 of 5)"),
            "recess .012 in deep (1 of 5)"
        );
        assert_eq!(
            inch_text("requires 122.6° and ø8 R2.54 at 1:2"),
            "requires 122.6° and ø.315 R.100 at 1:2"
        );
        assert_eq!(
            inch_text("corner f37:e43:e47, 4 hole(s)"),
            "corner f37:e43:e47, 4 hole(s)"
        );
        drop(_g);
        assert_eq!(inch_text("1.524 wide"), "1.524 wide");
    }

    #[test]
    fn unified_threads_are_found_and_read() {
        assert_eq!(unified_thread(6.35, Some(1.27)), Some("1/4-20 UNC"));
        assert_eq!(unified_thread(4.826, Some(25.4 / 32.0)), Some("#10-32 UNF"));
        assert_eq!(unified_thread(19.05, None), Some("3/4-10 UNC"));
        assert_eq!(unified_thread(8.0, Some(1.25)), None);
        let (d, p, c) = parse_unified("1/4-20 UNC-2B").unwrap();
        assert!((d - 6.35).abs() < 1e-9 && (p - 1.27).abs() < 1e-9);
        assert_eq!(c.as_deref(), Some("2B"));
        assert!(parse_unified("#10-32 UNF").is_some());
        assert!(parse_unified("M8x1.25").is_none());
    }

    #[test]
    fn sheet_names_map_both_ways() {
        assert_eq!(sheet_rank("B"), Some("A3"));
        assert_eq!(sheet_rank("ANSI D"), Some("A1"));
        assert_eq!(sheet_rank("A2"), Some("A2"));
        assert_eq!(sheet_rank("F"), None);
        let _g = set(true);
        assert_eq!(sheet_name("A3"), "ANSI B");
        drop(_g);
        assert_eq!(sheet_name("A3"), "A3");
    }
}
