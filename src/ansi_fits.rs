//! ANSI B4.1-1967 (R1987) preferred limits and fits for cylindrical parts, inch: the running
//! and sliding fits RC1 to RC9 and the locational clearance fits LC1 to LC11, for nominal sizes
//! up to 19.69 in. Used for a hole's or a boss's fit on an ASME sheet ([`crate::standard`]).
//!
//! Each row is a nominal size band, inches over the previous bound up to and including this
//! one, with the hole's upper limit (its lower limit is 0, basic hole) and the shaft's upper and
//! lower limits, thousandths of an inch, as the standard tabulates them. The rows were checked
//! against each other: every class's clearances follow from its limits, every hole of one grade
//! has one tolerance, and every shaft's tolerance is its grade's. The transition (LT),
//! interference (LN) and force (FN) fits are not tabulated here.

/// The size bands' upper bounds, inches.
pub const BANDS_IN: [f64; 13] = [
    0.12, 0.24, 0.40, 0.71, 1.19, 1.97, 3.15, 4.73, 7.09, 9.85, 12.41, 15.75, 19.69,
];

/// A class, its hole and shaft grades (ISO-style symbols as B4.1 gives them), and per band the
/// hole's upper limit and the shaft's upper and lower limits, thousandths of an inch.
pub struct Class {
    pub name: &'static str,
    pub hole: &'static str,
    pub shaft: &'static str,
    pub limits: [(f64, f64, f64); 13],
}

pub const CLASSES: [Class; 20] = [
    Class {
        name: "RC1",
        hole: "H5",
        shaft: "g4",
        limits: [
            (0.2, -0.1, -0.25),
            (0.2, -0.15, -0.3),
            (0.25, -0.2, -0.35),
            (0.3, -0.25, -0.45),
            (0.4, -0.3, -0.55),
            (0.4, -0.4, -0.7),
            (0.5, -0.4, -0.7),
            (0.6, -0.5, -0.9),
            (0.7, -0.6, -1.1),
            (0.8, -0.6, -1.2),
            (0.9, -0.8, -1.4),
            (1.0, -1.0, -1.7),
            (1.0, -1.2, -2.0),
        ],
    },
    Class {
        name: "RC2",
        hole: "H6",
        shaft: "g5",
        limits: [
            (0.25, -0.1, -0.3),
            (0.3, -0.15, -0.35),
            (0.4, -0.2, -0.45),
            (0.4, -0.25, -0.55),
            (0.5, -0.3, -0.7),
            (0.6, -0.4, -0.8),
            (0.7, -0.4, -0.9),
            (0.9, -0.5, -1.1),
            (1.0, -0.6, -1.3),
            (1.2, -0.6, -1.4),
            (1.2, -0.8, -1.7),
            (1.4, -1.0, -2.0),
            (1.6, -1.2, -2.2),
        ],
    },
    Class {
        name: "RC3",
        hole: "H7",
        shaft: "f6",
        limits: [
            (0.4, -0.3, -0.55),
            (0.5, -0.4, -0.7),
            (0.6, -0.5, -0.9),
            (0.7, -0.6, -1.0),
            (0.8, -0.8, -1.3),
            (1.0, -1.0, -1.6),
            (1.2, -1.2, -1.9),
            (1.4, -1.4, -2.3),
            (1.6, -1.6, -2.6),
            (1.8, -2.0, -3.2),
            (2.0, -2.5, -3.7),
            (2.2, -3.0, -4.4),
            (2.5, -4.0, -5.6),
        ],
    },
    Class {
        name: "RC4",
        hole: "H8",
        shaft: "f7",
        limits: [
            (0.6, -0.3, -0.7),
            (0.7, -0.4, -0.9),
            (0.9, -0.5, -1.1),
            (1.0, -0.6, -1.3),
            (1.2, -0.8, -1.6),
            (1.6, -1.0, -2.0),
            (1.8, -1.2, -2.4),
            (2.2, -1.4, -2.8),
            (2.5, -1.6, -3.2),
            (2.8, -2.0, -3.8),
            (3.0, -2.5, -4.5),
            (3.5, -3.0, -5.2),
            (4.0, -4.0, -6.5),
        ],
    },
    Class {
        name: "RC5",
        hole: "H8",
        shaft: "e7",
        limits: [
            (0.6, -0.6, -1.0),
            (0.7, -0.8, -1.3),
            (0.9, -1.0, -1.6),
            (1.0, -1.2, -1.9),
            (1.2, -1.6, -2.4),
            (1.6, -2.0, -3.0),
            (1.8, -2.5, -3.7),
            (2.2, -3.0, -4.4),
            (2.5, -3.5, -5.1),
            (2.8, -4.0, -5.8),
            (3.0, -5.0, -7.0),
            (3.5, -6.0, -8.2),
            (4.0, -8.0, -10.5),
        ],
    },
    Class {
        name: "RC6",
        hole: "H9",
        shaft: "e8",
        limits: [
            (1.0, -0.6, -1.2),
            (1.2, -0.8, -1.5),
            (1.4, -1.0, -1.9),
            (1.6, -1.2, -2.2),
            (2.0, -1.6, -2.8),
            (2.5, -2.0, -3.6),
            (3.0, -2.5, -4.3),
            (3.5, -3.0, -5.2),
            (4.0, -3.5, -6.0),
            (4.5, -4.0, -6.8),
            (5.0, -5.0, -8.0),
            (6.0, -6.0, -9.5),
            (6.0, -8.0, -12.0),
        ],
    },
    Class {
        name: "RC7",
        hole: "H9",
        shaft: "d8",
        limits: [
            (1.0, -1.0, -1.6),
            (1.2, -1.2, -1.9),
            (1.4, -1.6, -2.5),
            (1.6, -2.0, -3.0),
            (2.0, -2.5, -3.7),
            (2.5, -3.0, -4.6),
            (3.0, -4.0, -5.8),
            (3.5, -5.0, -7.2),
            (4.0, -6.0, -8.5),
            (4.5, -7.0, -9.8),
            (5.0, -8.0, -11.0),
            (6.0, -10.0, -13.5),
            (6.0, -12.0, -16.0),
        ],
    },
    Class {
        name: "RC8",
        hole: "H10",
        shaft: "c9",
        limits: [
            (1.6, -2.5, -3.5),
            (1.8, -2.8, -4.0),
            (2.2, -3.0, -4.4),
            (2.8, -3.5, -5.1),
            (3.5, -4.5, -6.5),
            (4.0, -5.0, -7.5),
            (4.5, -6.0, -9.0),
            (5.0, -7.0, -10.5),
            (6.0, -8.0, -12.0),
            (7.0, -10.0, -14.5),
            (8.0, -12.0, -17.0),
            (9.0, -14.0, -20.0),
            (10.0, -16.0, -22.0),
        ],
    },
    Class {
        name: "RC9",
        hole: "H11",
        shaft: "",
        limits: [
            (2.5, -4.0, -5.6),
            (3.0, -4.5, -6.0),
            (3.5, -5.0, -7.2),
            (4.0, -6.0, -8.8),
            (5.0, -7.0, -10.5),
            (6.0, -8.0, -12.0),
            (7.0, -9.0, -13.5),
            (9.0, -10.0, -15.0),
            (10.0, -12.0, -18.0),
            (12.0, -15.0, -22.0),
            (12.0, -18.0, -26.0),
            (14.0, -22.0, -31.0),
            (16.0, -25.0, -35.0),
        ],
    },
    Class {
        name: "LC1",
        hole: "H6",
        shaft: "h5",
        limits: [
            (0.25, 0.0, -0.2),
            (0.3, 0.0, -0.2),
            (0.4, 0.0, -0.25),
            (0.4, 0.0, -0.3),
            (0.5, 0.0, -0.4),
            (0.6, 0.0, -0.4),
            (0.7, 0.0, -0.5),
            (0.9, 0.0, -0.6),
            (1.0, 0.0, -0.7),
            (1.2, 0.0, -0.8),
            (1.2, 0.0, -0.9),
            (1.4, 0.0, -1.0),
            (1.6, 0.0, -1.0),
        ],
    },
    Class {
        name: "LC2",
        hole: "H7",
        shaft: "h6",
        limits: [
            (0.4, 0.0, -0.25),
            (0.5, 0.0, -0.3),
            (0.6, 0.0, -0.4),
            (0.7, 0.0, -0.4),
            (0.8, 0.0, -0.5),
            (1.0, 0.0, -0.6),
            (1.2, 0.0, -0.7),
            (1.4, 0.0, -0.9),
            (1.6, 0.0, -1.0),
            (1.8, 0.0, -1.2),
            (2.0, 0.0, -1.2),
            (2.2, 0.0, -1.4),
            (2.5, 0.0, -1.6),
        ],
    },
    Class {
        name: "LC3",
        hole: "H8",
        shaft: "h7",
        limits: [
            (0.6, 0.0, -0.4),
            (0.7, 0.0, -0.5),
            (0.9, 0.0, -0.6),
            (1.0, 0.0, -0.7),
            (1.2, 0.0, -0.8),
            (1.6, 0.0, -1.0),
            (1.8, 0.0, -1.2),
            (2.2, 0.0, -1.4),
            (2.5, 0.0, -1.6),
            (2.8, 0.0, -1.8),
            (3.0, 0.0, -2.0),
            (3.5, 0.0, -2.2),
            (4.0, 0.0, -2.5),
        ],
    },
    Class {
        name: "LC4",
        hole: "H10",
        shaft: "h9",
        limits: [
            (1.6, 0.0, -1.0),
            (1.8, 0.0, -1.2),
            (2.2, 0.0, -1.4),
            (2.8, 0.0, -1.6),
            (3.5, 0.0, -2.0),
            (4.0, 0.0, -2.5),
            (4.5, 0.0, -3.0),
            (5.0, 0.0, -3.5),
            (6.0, 0.0, -4.0),
            (7.0, 0.0, -4.5),
            (8.0, 0.0, -5.0),
            (9.0, 0.0, -6.0),
            (10.0, 0.0, -6.0),
        ],
    },
    Class {
        name: "LC5",
        hole: "H7",
        shaft: "g6",
        limits: [
            (0.4, -0.1, -0.35),
            (0.5, -0.15, -0.45),
            (0.6, -0.2, -0.6),
            (0.7, -0.25, -0.65),
            (0.8, -0.3, -0.8),
            (1.0, -0.4, -1.0),
            (1.2, -0.4, -1.1),
            (1.4, -0.5, -1.4),
            (1.6, -0.6, -1.6),
            (1.8, -0.6, -1.8),
            (2.0, -0.7, -1.9),
            (2.2, -0.7, -2.1),
            (2.5, -0.8, -2.4),
        ],
    },
    Class {
        name: "LC6",
        hole: "H9",
        shaft: "f8",
        limits: [
            (1.0, -0.3, -0.9),
            (1.2, -0.4, -1.1),
            (1.4, -0.5, -1.4),
            (1.6, -0.6, -1.6),
            (2.0, -0.8, -2.0),
            (2.5, -1.0, -2.6),
            (3.0, -1.2, -3.0),
            (3.5, -1.4, -3.6),
            (4.0, -1.6, -4.1),
            (4.5, -2.0, -4.8),
            (5.0, -2.2, -5.2),
            (6.0, -2.5, -6.0),
            (6.0, -2.8, -6.8),
        ],
    },
    Class {
        name: "LC7",
        hole: "H10",
        shaft: "e9",
        limits: [
            (1.6, -0.6, -1.6),
            (1.8, -0.8, -2.0),
            (2.2, -1.0, -2.4),
            (2.8, -1.2, -2.8),
            (3.5, -1.6, -3.6),
            (4.0, -2.0, -4.5),
            (4.5, -2.5, -5.5),
            (5.0, -3.0, -6.5),
            (6.0, -3.5, -7.5),
            (7.0, -4.0, -8.5),
            (8.0, -4.5, -9.5),
            (9.0, -5.0, -11.0),
            (10.0, -5.0, -11.0),
        ],
    },
    Class {
        name: "LC8",
        hole: "H10",
        shaft: "d9",
        limits: [
            (1.6, -1.0, -2.0),
            (1.8, -1.2, -2.4),
            (2.2, -1.6, -3.0),
            (2.8, -2.0, -3.6),
            (3.5, -2.5, -4.5),
            (4.0, -3.0, -5.5),
            (4.5, -4.0, -7.0),
            (5.0, -5.0, -8.5),
            (6.0, -6.0, -10.0),
            (7.0, -7.0, -11.5),
            (8.0, -7.0, -12.0),
            (9.0, -8.0, -14.0),
            (10.0, -9.0, -15.0),
        ],
    },
    Class {
        name: "LC9",
        hole: "H11",
        shaft: "c10",
        limits: [
            (2.5, -2.5, -4.1),
            (3.0, -2.8, -4.6),
            (3.5, -3.0, -5.2),
            (4.0, -3.5, -6.3),
            (5.0, -4.5, -8.0),
            (6.0, -5.0, -9.0),
            (7.0, -6.0, -10.5),
            (9.0, -7.0, -12.0),
            (10.0, -8.0, -14.0),
            (12.0, -10.0, -17.0),
            (12.0, -12.0, -20.0),
            (14.0, -14.0, -23.0),
            (16.0, -16.0, -26.0),
        ],
    },
    Class {
        name: "LC10",
        hole: "H12",
        shaft: "",
        limits: [
            (4.0, -4.0, -8.0),
            (5.0, -4.5, -9.5),
            (6.0, -5.0, -11.0),
            (7.0, -6.0, -13.0),
            (8.0, -7.0, -15.0),
            (10.0, -8.0, -18.0),
            (12.0, -10.0, -22.0),
            (14.0, -11.0, -25.0),
            (16.0, -12.0, -28.0),
            (18.0, -16.0, -34.0),
            (20.0, -20.0, -40.0),
            (22.0, -22.0, -44.0),
            (25.0, -25.0, -50.0),
        ],
    },
    Class {
        name: "LC11",
        hole: "H13",
        shaft: "",
        limits: [
            (6.0, -5.0, -11.0),
            (7.0, -6.0, -13.0),
            (9.0, -7.0, -16.0),
            (10.0, -8.0, -18.0),
            (12.0, -10.0, -22.0),
            (16.0, -12.0, -28.0),
            (18.0, -14.0, -32.0),
            (22.0, -16.0, -38.0),
            (25.0, -18.0, -43.0),
            (28.0, -22.0, -50.0),
            (30.0, -28.0, -58.0),
            (35.0, -30.0, -65.0),
            (40.0, -35.0, -75.0),
        ],
    },
];

/// The class named *code* (`RC4`, `lc2`), if it is one of the tabulated classes.
pub fn class(code: &str) -> Option<&'static Class> {
    let code = code.trim().to_ascii_uppercase();
    CLASSES.iter().find(|c| c.name == code)
}

/// The limits of class *code* at nominal diameter *nominal_mm*, for the hole (`hole`) or the
/// shaft, as (lower, upper) deviations in mm. None outside the table.
pub fn limits(code: &str, nominal_mm: f64, hole: bool) -> Option<(f64, f64)> {
    let c = class(code)?;
    let inches = nominal_mm / crate::standard::MM_PER_INCH;
    if inches <= 0.0 {
        return None;
    }
    // A nominal a hair over a bound (a rounding of the model) stays in the band it was meant for.
    let band = BANDS_IN.iter().position(|&b| inches <= b + 5e-5)?;
    let (hole_up, shaft_up, shaft_lo) = c.limits[band];
    let mm = |thou: f64| thou * crate::standard::MM_PER_INCH / 1000.0;
    Some(if hole {
        (0.0, mm(hole_up))
    } else {
        (mm(shaft_lo), mm(shaft_up))
    })
}

/// The tabulated classes that cover *nominal_mm*, in table order.
pub fn choices(nominal_mm: f64) -> Vec<String> {
    CLASSES
        .iter()
        .filter(|c| limits(c.name, nominal_mm, true).is_some())
        .map(|c| c.name.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One grade, one tolerance: every hole of a grade has the same limit in a band, and every
    /// shaft's tolerance equals its grade's hole tolerance.
    #[test]
    fn grades_are_consistent() {
        let mut grade: std::collections::BTreeMap<&str, [f64; 13]> = Default::default();
        for c in &CLASSES {
            let t: [f64; 13] = std::array::from_fn(|i| c.limits[i].0);
            if let Some(seen) = grade.insert(&c.hole[1..], t) {
                assert_eq!(seen, t, "{} {}", c.name, c.hole);
            }
        }
        for c in CLASSES.iter().filter(|c| !c.shaft.is_empty()) {
            let digits = c
                .shaft
                .trim_start_matches(|ch: char| ch.is_ascii_alphabetic());
            if let Some(t) = grade.get(digits) {
                for i in 0..13 {
                    let width = c.limits[i].1 - c.limits[i].2;
                    assert!((width - t[i]).abs() < 1e-9, "{} band {i}", c.name);
                }
            }
        }
    }

    #[test]
    fn limits_read_in_millimetres() {
        // RC5 at 2 in: hole +.0018/0, shaft -.0025/-.0037 (Engineers Edge's worked example).
        let (lo, hi) = limits("RC5", 50.8, true).unwrap();
        assert!(lo == 0.0 && (hi - 0.0018 * 25.4).abs() < 1e-9);
        let (lo, hi) = limits("rc5", 50.8, false).unwrap();
        assert!((hi + 0.0025 * 25.4).abs() < 1e-9 && (lo + 0.0037 * 25.4).abs() < 1e-9);
        assert!(limits("RC5", 600.0, true).is_none());
        assert!(class("H7").is_none());
        assert_eq!(choices(12.7).len(), 20);
    }
}
