//! The part's features as a plain list of sizes and locations (`draftwright features`), and a
//! check of that list against the text of a drawing someone else made (`draftwright check`).
//!
//! Neither plans nor draws anything. `features` answers "what did recognition find, how big is
//! it and where is it"; `check` answers "does this drawing state every one of those numbers".
//! The drawing's text is read as it is: a `.txt` file, or a `.pdf` through `pdftotext` (from
//! poppler) when that is installed.
//!
//! Sizes are the feature record's own named lengths (`diameter`, `radius`, `width`, `depth`,
//! …). A location is the feature's centre (its `location`, `at` or `center` parameter, else the
//! centre of its faces' box) measured from the part's minimum corner along X, Y and Z, as a
//! hole table measures it; `check` asks for the locations of holes, bosses and slots only. A drawing states a number when its text holds that number at any
//! precision from one to four places that it rounds to (`.250`, `.25`, `6.35`, `6.4`), in the
//! chosen unit. A diameter is also found as its radius and a radius as its diameter. A location
//! is also found measured from the part's maximum corner or its centre, because drawings
//! dimension from either side; one on the part's centre plane or an edge needs no number. A location the text does not state is reported, never an error:
//! many are fixed by chain or baseline dimensions the text cannot show as one number.

use crate::model::Analysis;
use serde::Serialize;
use serde_json::Value;

/// The size parameters a feature record may name, as lengths in mm.
pub const SIZE_KEYS: [&str; 14] = [
    "diameter",
    "radius",
    "width",
    "depth",
    "height",
    "length",
    "across_flats",
    "thickness",
    "size",
    "major",
    "minor",
    "od",
    "id",
    "pitch",
];

/// The kinds that are a grouping of other features or the stock itself: listed, never checked
/// for a location of their own.
const GROUPS: [&str; 6] = [
    "slot_pattern",
    "hole_pattern",
    "plate",
    "section_recess",
    "turned_body",
    "prismatic_body",
];

/// The kinds whose centre a drawing must locate: holes, bosses and slots. Other features' centres
/// (a flat's, a pad's) are where their faces happen to sit and are fixed by the outline.
const LOCATED: [&str; 5] = ["hole", "boss", "slot", "threaded_hole", "counterbore"];

/// One feature's sizes and location.
#[derive(Clone, Debug, Serialize)]
pub struct Item {
    pub id: String,
    pub kind: String,
    /// Named sizes in the report's unit.
    pub sizes: std::collections::BTreeMap<String, f64>,
    /// Centre from the part's minimum corner, X, Y, Z, in the report's unit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_min: Option<[f64; 3]>,
    /// The feature's axis, when it has one: its centre along that axis is not a location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axis: Option<[f64; 3]>,
    /// How many like features, when recognition grouped a pattern on this one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
}

/// The report's unit: inches or millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Unit {
    Inch,
    Mm,
}

impl Unit {
    pub fn read(text: Option<&str>) -> Result<Unit, String> {
        match text.map(str::to_ascii_lowercase).as_deref() {
            None | Some("in") | Some("inch") | Some("inches") => Ok(Unit::Inch),
            Some("mm") => Ok(Unit::Mm),
            Some(other) => Err(format!("--units {other:?}: use in or mm")),
        }
    }
    fn of(self, mm: f64) -> f64 {
        match self {
            Unit::Inch => mm / crate::standard::MM_PER_INCH,
            Unit::Mm => mm,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Unit::Inch => "in",
            Unit::Mm => "mm",
        }
    }
}

fn point(v: Option<&Value>) -> Option<[f64; 3]> {
    let a = v?.as_array()?;
    if a.len() != 3 {
        return None;
    }
    Some([a[0].as_f64()?, a[1].as_f64()?, a[2].as_f64()?])
}

/// Every recognised feature, in recognition's order.
pub fn items(analysis: &Analysis, unit: Unit) -> Vec<Item> {
    let lo = analysis.envelope[0];
    analysis
        .features
        .iter()
        .map(|f| {
            let mut sizes: std::collections::BTreeMap<String, f64> = SIZE_KEYS
                .iter()
                .filter_map(|k| {
                    let v = f.parameters.get(*k)?.as_f64()?;
                    (v > 0.0).then(|| ((*k).to_string(), round(unit.of(v))))
                })
                .collect();
            for (k, v) in &f.fingerprint.sizes {
                // Fingerprint sizes are lengths, except a chamfer's or a pattern's angle.
                let angle = k.contains("angle");
                if *v > 0.0 && !angle && !sizes.contains_key(k) {
                    sizes.insert(k.clone(), round(unit.of(*v)));
                }
            }
            let at = ["location", "at", "center", "centre"]
                .iter()
                .find_map(|k| point(f.parameters.get(*k)))
                .unwrap_or(f.fingerprint.position);
            let from_min = (!GROUPS.contains(&f.kind.as_str()))
                .then(|| std::array::from_fn(|k| round(unit.of(at[k] - lo[k]))));
            // A THRU hole's depth is the wall it goes through, stated by THRU; a countersink is
            // called by its diameter and angle, its depth follows.
            if f.parameters.get("bottom").and_then(Value::as_str) == Some("through") {
                sizes.remove("depth");
            }
            if f.kind == "countersink" {
                sizes.remove("depth");
            }
            sizes.remove("csink.depth");
            Item {
                id: f.id.clone(),
                kind: f.kind.clone(),
                axis: f.fingerprint.axis,
                sizes,
                from_min,
                count: f
                    .fingerprint
                    .pattern
                    .as_ref()
                    .and_then(|p| serde_json::to_value(p).ok())
                    .and_then(|p| p.get("count").and_then(Value::as_u64)),
            }
        })
        .collect()
}

fn round(v: f64) -> f64 {
    (v * 1e4).round() / 1e4
}

/// The texts *value* may be printed as: one to four places, and without a leading zero.
pub fn spellings(value: f64) -> Vec<String> {
    let mut out = Vec::new();
    for places in 1..=4 {
        let t = format!("{:.*}", places, value.abs());
        out.push(t.clone());
        if let Some(rest) = t.strip_prefix("0.") {
            out.push(format!(".{rest}"));
        }
    }
    // A whole number may print bare.
    if (value - value.round()).abs() < 1e-4 {
        out.push(format!("{}", value.abs().round()));
    }
    out.sort();
    out.dedup();
    out
}

/// The numbers *text* prints, as they are written.
fn tokens(text: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    let mut cur = String::new();
    for c in text.chars().chain(std::iter::once(' ')) {
        if c.is_ascii_digit() || (c == '.' && !cur.contains('.')) {
            cur.push(c);
        } else {
            let t = cur.trim_end_matches('.').to_string();
            if t.chars().any(|c| c.is_ascii_digit()) {
                out.insert(t);
            }
            cur.clear();
            if c == '.' {
                cur.push(c);
            }
        }
    }
    out
}

/// What a check found for one number.
#[derive(Clone, Debug, Serialize)]
pub struct Finding {
    pub feature: String,
    pub kind: String,
    /// `size:<name>` or `location:<axis>`.
    pub what: String,
    pub value: f64,
    pub found: bool,
}

/// Each size and location of *items* against *text*. Locations at the part's edge (zero from
/// the minimum or maximum) are skipped: an edge needs no dimension.
pub fn check(analysis: &Analysis, items: &[Item], text: &str, unit: Unit) -> Vec<Finding> {
    let printed = tokens(text);
    let states = |v: f64| spellings(v).iter().any(|s| printed.contains(s));
    let [lo, hi] = analysis.envelope;
    let span: [f64; 3] = std::array::from_fn(|k| unit.of(hi[k] - lo[k]));
    let mut out = Vec::new();
    for item in items {
        for (name, v) in &item.sizes {
            let found = states(*v)
                || (name == "diameter" && states(v / 2.0))
                || (name == "radius" && states(v * 2.0));
            out.push(Finding {
                feature: item.id.clone(),
                kind: item.kind.clone(),
                what: format!("size:{name}"),
                value: *v,
                found,
            });
        }
        if let Some(at) = item
            .from_min
            .filter(|_| LOCATED.contains(&item.kind.as_str()))
        {
            // On a bolt circle about the part's centre: located by the circle (and an angle).
            if let Some(ax) = item.axis {
                let r2: f64 = (0..3)
                    .filter(|&k| ax[k].abs() <= 0.9)
                    .map(|k| (at[k] - span[k] / 2.0).powi(2))
                    .sum();
                let r = r2.sqrt();
                if r > 1e-3 && (states(2.0 * r) || states(r)) {
                    out.push(Finding {
                        feature: item.id.clone(),
                        kind: item.kind.clone(),
                        what: "location:circle".into(),
                        value: round(2.0 * r),
                        found: true,
                    });
                    continue;
                }
            }
            for k in 0..3 {
                if item.axis.is_some_and(|ax| ax[k].abs() > 0.9) {
                    continue;
                }
                let (a, b) = (at[k], span[k] - at[k]);
                // On an edge or on the centre plane: fixed by the outline or by symmetry.
                if a.abs() < 1e-3 || b.abs() < 1e-3 || (a - span[k] / 2.0).abs() < 1e-3 {
                    continue;
                }
                let found = states(a) || states(b) || states((a - span[k] / 2.0).abs());
                out.push(Finding {
                    feature: item.id.clone(),
                    kind: item.kind.clone(),
                    what: format!("location:{}", ["x", "y", "z"][k]),
                    value: a,
                    found,
                });
            }
        }
    }
    out
}

/// The `features` command's result.
pub fn report(analysis: &Analysis, unit: Unit) -> Value {
    let [lo, hi] = analysis.envelope;
    serde_json::json!({
        "units": unit.name(),
        "envelope": std::array::from_fn::<f64, 3, _>(|k| round(unit.of(hi[k] - lo[k]))),
        "origin": "the part's minimum corner",
        "features": items(analysis, unit),
    })
}

/// The `check` command's result: the numbers *text* does not state, and a tally.
pub fn check_report(analysis: &Analysis, text: &str, unit: Unit) -> Value {
    let items = items(analysis, unit);
    let findings = check(analysis, &items, text, unit);
    let missing: Vec<&Finding> = findings.iter().filter(|f| !f.found).collect();
    let sizes = |found: bool| {
        findings
            .iter()
            .filter(|f| f.what.starts_with("size:") && (found || !f.found))
            .count()
    };
    let locations = |found: bool| {
        findings
            .iter()
            .filter(|f| f.what.starts_with("location:") && (found || !f.found))
            .count()
    };
    serde_json::json!({
        "units": unit.name(),
        "sizes": { "checked": sizes(true), "not_stated": sizes(false) },
        "locations": { "checked": locations(true), "not_stated": locations(false) },
        "note": "A location not stated as one number may still be fixed by a chain or baseline of dimensions; read those by eye.",
        "not_stated": missing,
    })
}

/// The text of a drawing: a text file as it is, a PDF through `pdftotext -layout`.
pub fn drawing_text(path: &std::path::Path) -> Result<String, String> {
    let pdf = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
    if !pdf {
        return std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()));
    }
    let out = std::process::Command::new("pdftotext")
        .arg("-layout")
        .arg(path)
        .arg("-")
        .output()
        .map_err(|e| {
            format!("reading a PDF needs pdftotext (poppler-utils) installed ({e}); or pass the drawing's text as a .txt file")
        })?;
    if !out.status.success() {
        return Err(format!(
            "pdftotext could not read {}: {}",
            path.display(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spellings_cover_asme_and_metric_forms() {
        let s = spellings(0.25);
        assert!(s.contains(&".250".to_string()) && s.contains(&".25".to_string()));
        let s = spellings(6.35);
        assert!(s.contains(&"6.35".to_string()) && s.contains(&"6.350".to_string()));
        assert!(spellings(12.0).contains(&"12".to_string()));
    }

    #[test]
    fn tokens_read_leading_points() {
        let t = tokens("4× ⌀.250 THRU  R1.50 12.");
        assert!(t.contains(".250") && t.contains("1.50") && t.contains("12"));
    }
}
