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
//! many are fixed by chain or baseline dimensions the text cannot show as one number. A location
//! is also found through a chain (the distance from an already located feature is printed) or a
//! shared centreline (the same coordinate as a located feature), repeated until no more are.

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

/// The texts a location of *value* may be printed as: three or four places (with or without a
/// leading zero), and one or two only when *value* is exact at that precision; never a bare whole
/// number.
pub fn exact_spellings(value: f64) -> Vec<String> {
    spellings(value)
        .into_iter()
        .filter(|s| {
            let places = s.split_once('.').map_or(0, |(_, f)| f.len());
            // A bare whole number (`2`) is a count or a sheet number as often as a length.
            places >= 3
                || (places > 0
                    && s.parse::<f64>()
                        .is_ok_and(|n| (n - value.abs()).abs() < 5e-5))
        })
        .collect()
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
    /// How a location was found when not as one number from an edge: `chain from <feature>`
    /// (the distance to an already located feature is printed) or `aligned with <feature>`
    /// (on a located feature's centreline).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub how: Option<String>,
}

/// Each size and location of *items* against *text*. Locations at the part's edge (zero from
/// the minimum or maximum) are skipped: an edge needs no dimension.
pub fn check(analysis: &Analysis, items: &[Item], text: &str, unit: Unit) -> Vec<Finding> {
    let printed = tokens(text);
    let states = |v: f64| spellings(v).iter().any(|s| printed.contains(s));
    // A location's number must be what the value prints as: three or four places, or fewer only
    // when the value is exact at them (`2.15` for 2.150, never `.3` for .297). Loose matching
    // would find a location in any page dense with numbers.
    let exact = |v: f64| exact_spellings(v).iter().any(|s| printed.contains(s));
    let [lo, hi] = analysis.envelope;
    let span: [f64; 3] = std::array::from_fn(|k| unit.of(hi[k] - lo[k]));
    let mut out = Vec::new();
    // Coordinates located without a number: on an edge, the centre plane or a bolt circle.
    let mut fixed: Vec<(usize, f64, String)> = Vec::new();
    for item in items {
        for (name, v) in &item.sizes {
            let found = states(*v)
                || (name.starts_with("half_") && states(v * 2.0))
                || (name == "diameter" && states(v / 2.0))
                || (name == "radius" && states(v * 2.0));
            out.push(Finding {
                feature: item.id.clone(),
                kind: item.kind.clone(),
                what: format!("size:{name}"),
                value: *v,
                found,
                how: None,
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
                if r > 1e-3 && (exact(2.0 * r) || exact(r)) {
                    out.push(Finding {
                        feature: item.id.clone(),
                        kind: item.kind.clone(),
                        what: "location:circle".into(),
                        value: round(2.0 * r),
                        found: true,
                        how: None,
                    });
                    for k in (0..3).filter(|&k| ax[k].abs() <= 0.9) {
                        fixed.push((k, at[k], item.id.clone()));
                    }
                    continue;
                }
            }
            // A radial hole (its axis runs through the part's centre across two axes, as a
            // wheel's rim holes do) is placed by its angle about that centre, not by the two
            // coordinates in that plane.
            let radial: Vec<usize> = item
                .axis
                .map(|ax| {
                    let comps: Vec<usize> = (0..3).filter(|&k| ax[k].abs() > 1e-3).collect();
                    let off: Vec<f64> = comps.iter().map(|&k| at[k] - span[k] / 2.0).collect();
                    let cross = comps.len() == 2
                        && (off[0] * ax[comps[1]] - off[1] * ax[comps[0]]).abs() < 1e-3
                        && off.iter().any(|o| o.abs() > 1e-3);
                    if cross { comps } else { Vec::new() }
                })
                .unwrap_or_default();
            if !radial.is_empty() {
                out.push(Finding {
                    feature: item.id.clone(),
                    kind: item.kind.clone(),
                    what: "location:angle".into(),
                    value: round(
                        (at[radial[1]] - span[radial[1]] / 2.0)
                            .atan2(at[radial[0]] - span[radial[0]] / 2.0)
                            .to_degrees(),
                    ),
                    found: true,
                    how: Some("radial: placed by its angle about the part's centre".into()),
                });
            }
            for k in 0..3 {
                if item.axis.is_some_and(|ax| ax[k].abs() > 0.9) || radial.contains(&k) {
                    continue;
                }
                let (a, b) = (at[k], span[k] - at[k]);
                // On an edge or on the centre plane: fixed by the outline or by symmetry.
                if a.abs() < 1e-3 || b.abs() < 1e-3 || (a - span[k] / 2.0).abs() < 1e-3 {
                    fixed.push((k, a, item.id.clone()));
                    continue;
                }
                let found = exact(a) || exact(b) || exact((a - span[k] / 2.0).abs());
                out.push(Finding {
                    feature: item.id.clone(),
                    kind: item.kind.clone(),
                    what: format!("location:{}", ["x", "y", "z"][k]),
                    value: a,
                    found,
                    how: None,
                });
            }
        }
    }
    // The part's own walls as reference planes, when the text locates them from an edge: a plate's
    // faces, a riser (a step's wall), a level. Drawings dimension holes from such a face.
    for (k, p, id) in planes(analysis) {
        let a = unit.of(p - lo[k]);
        let b = span[k] - a;
        if a.abs() > 1e-3 && b.abs() > 1e-3 && (exact(a) || exact(b)) {
            fixed.push((k, a, format!("{id} ({} from the minimum)", round(a))));
        }
    }
    // Chains and arcs feed each other: run both until neither finds more.
    loop {
        let before = out.iter().filter(|f| f.found).count();
        chains(&mut out, &fixed, &exact);
        arcs(items, &mut out, &span, &exact);
        if out.iter().filter(|f| f.found).count() == before {
            break;
        }
    }
    out
}

/// Holes and bosses on an arc about another point: a corner, an edge's midpoint, the centre or
/// another feature's axis. When the text states the arc's radius (or diameter), the feature is
/// located on both axes of its plane (its angle is read by eye).
fn arcs(items: &[Item], out: &mut [Finding], span: &[f64; 3], states: &dyn Fn(f64) -> bool) {
    for item in items.iter().filter(|i| LOCATED.contains(&i.kind.as_str())) {
        let (Some(at), Some(ax)) = (item.from_min, item.axis) else {
            continue;
        };
        let plane: Vec<usize> = (0..3).filter(|&k| ax[k].abs() <= 0.9).collect();
        if plane.len() != 2 {
            continue;
        }
        let open = |out: &[Finding]| {
            out.iter()
                .any(|f| f.feature == item.id && !f.found && f.what.starts_with("location:"))
        };
        if !open(out) {
            continue;
        }
        let (u, v) = (plane[0], plane[1]);
        let mut centres: Vec<([f64; 2], String)> = Vec::new();
        for (cu, nu) in [(0.0, "min"), (span[u] / 2.0, "mid"), (span[u], "max")] {
            for (cv, nv) in [(0.0, "min"), (span[v] / 2.0, "mid"), (span[v], "max")] {
                let name = format!(
                    "the point {nu} {}, {nv} {}",
                    ["x", "y", "z"][u],
                    ["x", "y", "z"][v]
                );
                centres.push(([cu, cv], name));
            }
        }
        // Another feature is a centre only once it is located itself (or needs no location).
        let located = |id: &str| {
            out.iter()
                .filter(|f| f.feature == id && f.what.starts_with("location:"))
                .all(|f| f.found)
        };
        for other in items.iter().filter(|o| o.id != item.id && located(&o.id)) {
            if let (Some(p), Some(oax)) = (other.from_min, other.axis)
                && (0..3).all(|k| (oax[k].abs() - ax[k].abs()).abs() < 1e-3)
            {
                centres.push(([p[u], p[v]], other.id.clone()));
            }
        }
        let hit = centres.iter().find_map(|(c, name)| {
            let r = ((at[u] - c[0]).powi(2) + (at[v] - c[1]).powi(2)).sqrt();
            (r > 1e-3 && (states(r) || states(2.0 * r)))
                .then(|| format!("on R{} about {name}", round(r)))
        });
        if let Some(how) = hit {
            for f in out
                .iter_mut()
                .filter(|f| f.feature == item.id && !f.found && f.what.starts_with("location:"))
            {
                f.found = true;
                f.how = Some(how.clone());
            }
        }
    }
}

/// The part's planar walls along each axis, mm in the part's frame: each plate's two faces,
/// each riser's positions and each support level (Z).
fn planes(analysis: &Analysis) -> Vec<(usize, f64, String)> {
    let axis = |a: &str| ["x", "y", "z"].iter().position(|x| *x == a);
    let mut out = Vec::new();
    for f in analysis.features.iter().filter(|f| f.kind == "plate") {
        let Some(k) = f
            .parameters
            .get("axis")
            .and_then(Value::as_str)
            .and_then(axis)
        else {
            continue;
        };
        for side in ["lo", "hi"] {
            if let Some(v) = f.parameters.get(side).and_then(Value::as_f64) {
                out.push((k, v, format!("{} {side} face", f.id)));
            }
        }
    }
    for r in &analysis.risers {
        if let Some(k) = axis(&r.axis) {
            out.extend(
                r.positions
                    .iter()
                    .map(|&v| (k, v, format!("the wall across {}", r.axis))),
            );
        }
    }
    out.extend(
        analysis
            .levels
            .iter()
            .map(|&v| (2, v, "a level face".to_string())),
    );
    out
}

/// Locations found through other features: a chain dimension (the distance from an already
/// located feature is printed) or a shared centreline (the same coordinate as one). Repeats
/// until nothing more is found, so a chain of any length resolves from its located end.
/// *fixed* holds the coordinates located by the outline, by symmetry or by a bolt circle.
fn chains(out: &mut [Finding], fixed: &[(usize, f64, String)], states: &dyn Fn(f64) -> bool) {
    let axis = |f: &Finding| {
        f.what
            .strip_prefix("location:")
            .and_then(|a| ["x", "y", "z"].iter().position(|x| *x == a))
    };
    loop {
        let mut anchors: Vec<(usize, f64, String)> = fixed.to_vec();
        anchors.extend(
            out.iter()
                .filter(|f| f.found)
                .filter_map(|f| Some((axis(f)?, f.value, f.feature.clone()))),
        );
        let mut changed = false;
        for f in out.iter_mut().filter(|f| !f.found) {
            let Some(k) = axis(f) else { continue };
            let hit = anchors
                .iter()
                .filter(|(ak, _, id)| *ak == k && *id != f.feature)
                .find_map(|(_, b, id)| {
                    let d = (f.value - b).abs();
                    if d < 1e-3 {
                        Some(format!("aligned with {id}"))
                    } else if states(d) {
                        Some(format!("chain from {id}"))
                    } else {
                        None
                    }
                });
            if let Some(how) = hit {
                f.found = true;
                f.how = Some(how);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
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
        "note": "Locations are found from an edge, the centre, a bolt circle, a chain from another located feature, or a shared centreline. What is left is not stated by any of those; check it by eye.",
        "not_stated": missing,
        "located_through": findings.iter().filter(|f| f.how.is_some()).collect::<Vec<_>>(),
    })
}

/// A drawing set's pages' text: a PDF's pages through `pdftotext -layout` (which ends each page
/// with a form feed), or a text file's form-feed-separated pages.
pub fn drawing_pages(path: &std::path::Path) -> Result<Vec<String>, String> {
    let text = drawing_text(path)?;
    let mut pages: Vec<String> = text.split('\u{c}').map(String::from).collect();
    while pages.last().is_some_and(|p| p.trim().is_empty()) {
        pages.pop();
    }
    Ok(pages)
}

/// The drawing number a STEP file's name gives: its stem up to the first `_` or space
/// (`SRT-100-118_pressure_finger.step` is `SRT-100-118`).
pub fn drawing_number(step: &std::path::Path) -> String {
    let stem = step
        .file_stem()
        .map_or(String::new(), |s| s.to_string_lossy().into_owned());
    stem.split(['_', ' ']).next().unwrap_or(&stem).to_string()
}

/// How a drawing's pages were found ([`match_pages`]).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Pages {
    /// Indices into the set's pages.
    pub pages: Vec<usize>,
    /// `title block` when the pages carry the number in their title block, `mentioned` when no
    /// page does and the page that mentions it with the fewest other numbers stands in.
    pub how: &'static str,
}

/// Where *number* last occurs in *text* as a whole word.
fn last_named(text: &str, number: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    text.match_indices(number)
        .filter(|(i, _)| {
            let before = i.checked_sub(1).map(|j| bytes[j]);
            let after = bytes.get(i + number.len()).copied();
            let edge = |b: Option<u8>| b.is_none_or(|b| !(b.is_ascii_alphanumeric() || b == b'-'));
            edge(before) && edge(after)
        })
        .map(|(i, _)| i)
        .last()
}

/// Whether *text* names drawing number *number* as a whole word (`SRT-100-11` is not in
/// `SRT-100-118`).
fn names(text: &str, number: &str) -> bool {
    last_named(text, number).is_some()
}

/// For each drawing number, its sheets. A page belongs to the number it prints last: the title
/// block is at the foot of the sheet, so its number comes after any the views or notes mention.
/// A drawing of several sheets gets them all. A number no page belongs to falls back to the
/// page that mentions it with the fewest other numbers (an index or an assembly names many),
/// the last such on a tie; none when no page mentions it.
pub fn match_pages(numbers: &[String], pages: &[String]) -> Vec<Option<Pages>> {
    let owner: Vec<Option<&String>> = pages
        .iter()
        .map(|p| {
            numbers
                .iter()
                .filter_map(|n| last_named(p, n).map(|at| (at, n)))
                .max_by_key(|&(at, n)| (at, n.len()))
                .map(|(_, n)| n)
        })
        .collect();
    numbers
        .iter()
        .map(|n| {
            let own: Vec<usize> = (0..pages.len()).filter(|&i| owner[i] == Some(n)).collect();
            if !own.is_empty() {
                return Some(Pages {
                    pages: own,
                    how: "title block",
                });
            }
            pages
                .iter()
                .enumerate()
                .filter(|(_, p)| names(p, n))
                .map(|(i, p)| (numbers.iter().filter(|m| *m != n && names(p, m)).count(), i))
                .min_by_key(|&(others, i)| (others, std::cmp::Reverse(i)))
                .map(|(_, i)| Pages {
                    pages: vec![i],
                    how: "mentioned",
                })
        })
        .collect()
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
    fn pages_match_by_drawing_number() {
        let numbers = vec!["SRT-100-101".to_string(), "SRT-100-118".to_string()];
        let pages = vec![
            "INDEX SRT-100-101 SRT-100-118".to_string(),
            "BASE  DWG SRT-100-101".to_string(),
            "FINGER SRT-100-118 SEE SRT-100-1180".to_string(),
        ];
        let found = match_pages(&numbers, &pages);
        assert_eq!(found[0].as_ref().unwrap().pages, [1]);
        assert_eq!(found[1].as_ref().unwrap().pages, [2]);
        // A drawing of two sheets, and a part no title block carries.
        let pages = vec![
            "views SRT-100-118 ... SRT-100-101 SHEET 1".to_string(),
            "notes ... SRT-100-101 SHEET 2".to_string(),
            "assembly lists SRT-100-118 and SRT-100-777".to_string(),
        ];
        let numbers = vec!["SRT-100-101".to_string(), "SRT-100-118".to_string()];
        let found = match_pages(&numbers, &pages);
        assert_eq!(
            found[0],
            Some(Pages {
                pages: vec![0, 1],
                how: "title block"
            })
        );
        assert_eq!(found[1].as_ref().unwrap().how, "title block");
        assert_eq!(
            drawing_number(std::path::Path::new("x/SRT-100-118_pressure_finger.step")),
            "SRT-100-118"
        );
    }

    #[test]
    fn locations_need_their_printed_precision() {
        let s = exact_spellings(0.297);
        assert!(s.contains(&".297".to_string()) && !s.contains(&".3".to_string()));
        assert!(exact_spellings(2.15).contains(&"2.15".to_string()));
        assert!(!exact_spellings(2.0).contains(&"2".to_string()));
    }

    #[test]
    fn tokens_read_leading_points() {
        let t = tokens("4× ⌀.250 THRU  R1.50 12.");
        assert!(t.contains(".250") && t.contains("1.50") && t.contains("12"));
    }
}
