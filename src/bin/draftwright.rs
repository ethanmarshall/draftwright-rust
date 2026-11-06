//! The command line over the document model (ADR R1), through [`draftwright::api`]. Each command
//! prints JSON.
//!
//! - `draftwright analyse part.step [--features r.json | --no-features]`: the Analysis
//! - `draftwright plan part.step [--features r.json | --no-features] [--spec s.json]`: the Plan;
//!   or `draftwright plan [part.step] --analysis a.json [--spec s.json]`, from an Analysis
//!   without analysing again
//! - `draftwright render part.step [--features r.json | --no-features | --analysis a.json]
//!   [--spec s.json] -o out [--formats svg,pdf,png] [--dpi 150]`: writes the files
//!   (`out.svg`, …; svg and pdf unless named, a PNG at `--dpi` pixels per inch; DXF export is
//!   disabled until it is fixed) and prints
//!   what was written, with the problems: the plan's, and what could not be drawn as planned
//! - `draftwright review part.step [--features r.json | --no-features | --analysis a.json]
//!   [--spec s.json]`: the drawing as SVG text, with the problems and lint's findings on the
//!   drawing
//! - `draftwright edit s.json ops.json`: the edited Spec
//! - `draftwright spec drawing.(svg|dxf|pdf|png)`: the Spec embedded in a rendered drawing
//! - `draftwright diff old.step new.step [--spec s.json] [--part K] -o dir [--title T]
//!   [--dpi 150]`: the drawing of *old* with the Spec, the Spec carried to *new* through
//!   quiddity's correspondence (ADR R3) and *new* drawn with it, and what changed
//!   ([`draftwright::revision`]). Writes `report.json`, `report.md` (a pull request comment),
//!   `spec.json` (the carried Spec), `changes.svg` and `changes.png` (the new drawing marked up)
//!   and `drawing.svg` into *dir*, and prints the verdict
//! - `draftwright features part.step [--units in|mm] [--analysis a.json]`: every recognised
//!   feature's sizes and its centre from the part's minimum corner ([`draftwright::inventory`])
//! - `draftwright check part.step --drawing d.pdf|d.txt [--units in|mm] [--analysis a.json]`:
//!   which of those sizes and locations a drawing's text does not state; `--fix punch.md`
//!   also writes a punch list with the callout each gap needs (and, for a drawing draftwright
//!   made, its review's coverage fixes as `spec_fixes`)
//! - `draftwright check-set parts/ --drawing set.pdf [--units in|mm]`: `check` for every STEP
//!   file in a folder against its page of a drawing set, the page found by the drawing number
//!   the file's name starts with (`SRT-100-118_finger.step` is drawing `SRT-100-118`)
//! - `draftwright parts part.step`: the file's distinct parts (`{parts: [{part, name,
//!   product_id, faces, instances}]}`), by which one part of an assembly is chosen
//! - `draftwright serve`: the JSON-lines service. One request per line on stdin, such as
//!   `{"id": 1, "op": "plan", "step": "part.step", "spec": {…}}`, and one reply per line on
//!   stdout: `{"id": 1, "ok": true, "result": …}` or `{"id": 1, "ok": false, "error": {"code",
//!   "message", "choices"?}}`. `{"op": "hello"}` gives the protocol version and the operations;
//!   the others, and their arguments, are those of [`draftwright::api::call`].
//!
//! The features come from Rust quiddity's recognition, run on the part
//! ([`draftwright::api::Features::Native`]), unless one of these says otherwise. A part whose
//! recognition document is refused is not drawn (exit 1, with the refusal), and the families
//! that recognition does not give are named in the plan's problems (`features_not_recognised`).
//! `--features` names Python quiddity's recognition JSON for the same file
//! (`tools/recognise.py`, read by [`draftwright::bridge`]), used instead. `--no-features`
//! analyses the part with no features (no feature dimensions, sections or lint), as the library
//! does when given none: the way to draw a part whose recognition is refused. `--analysis` names
//! an Analysis that `draftwright analyse` printed for the same STEP file (with its features),
//! used in place of analysing the file again; it is refused with `--features` or
//! `--no-features`, or when made from another file. `serve` takes Python's JSON or none, as
//! [`draftwright::api::call`] does.
//!
//! `analyse`, `plan`, `render` and `review` take `--part K`: part K of an assembly (as `parts`
//! lists them) drawn alone, in its own frame and with its PMI only. Without it the whole file is
//! drawn as one part, and an assembly's plan warns (`assembly_drawn_whole`), naming its parts;
//! when every body coincides with one part's (`coincident_bodies`), or lies within the largest
//! body's box (`assembly_principal_drawn`), that part is drawn alone.
//!
//! Every command takes `--standard iso|asme`: the drawing standard for this run, set over the
//! Spec's `sheet/standard`, so one Spec draws either way. A check then reports in the standard's
//! unit (inches for asme, millimetres for iso) unless `--units` says otherwise.
//!
//! `features`, `check` and `check-set` keep each part's Analysis in a cache
//! (`DRAFTWRIGHT_CACHE`, else `~/.cache/draftwright/analyses`), keyed by the STEP file's SHA-256
//! and this build, so checking an unchanged part again skips recognition; `--no-cache` analyses
//! afresh.
//!
//! Exit codes: 0 done; 2 the request is invalid (the JSON error says what would be accepted);
//! 1 something failed and nothing was written in its place.

use std::io::{BufRead, Write};
use std::process::ExitCode;

use draftwright::api::{self, Error, Features};
use draftwright::model::Spec;
use serde_json::{Value, json};

/// The service's protocol version, given by `hello`.
const PROTOCOL: u32 = 1;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() == 1 {
        match args[0].as_str() {
            "--help" | "-h" => {
                println!(
                    "draftwright — engineering drawings from STEP\n\nCommands:\n  analyse <step>\n  plan <step> [--spec spec.json]\n  render <step> [--spec spec.json] -o <stem> [--formats svg,pdf,png]\n  review <step> [--spec spec.json]\n  edit <spec.json> <ops.json>\n  spec <drawing>\n  parts <step>\n  features <step> [--units in|mm]\n  check <step> --drawing <drawing.pdf|.txt> [--units in|mm] [--fix punch.md]\n  check-set <step folder> --drawing <set.pdf> [--units in|mm] [--fix punch.md]\n  pack <step> -o <directory>\n  diff <old.step> <new.step> -o <directory>\n  spec_schema\n  item_options <part_ref> <id> [--spec spec.json]\n  serve\n  mcp-stdio | mcp-http\n\nAny command takes --standard iso|asme to draw (or check) to that standard.\n\nDiscovery: schemas/spec.json, docs/model.md and docs/mcp.md ship in the release.\nJSON-lines serve: {{\"op\":\"hello\"}} lists operations; spec_schema and item_options use the MCP argument shapes.\nUse --version for the release version."
                );
                return ExitCode::SUCCESS;
            }
            "--version" | "-V" => {
                println!("draftwright {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            _ => {}
        }
    }
    if args
        .first()
        .is_some_and(|s| ["mcp-stdio", "mcp-http"].contains(&s.as_str()))
    {
        return match draftwright::mcp::run(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        };
    }
    if args.first().map(String::as_str) == Some("serve") {
        return serve();
    }
    match run(&args) {
        Ok(out) => {
            println!("{}", serde_json::to_string_pretty(&out).expect("JSON"));
            ExitCode::SUCCESS
        }
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({ "ok": false, "error": error }))
                    .expect("JSON")
            );
            ExitCode::from(if error.code == "invalid" { 2 } else { 1 })
        }
    }
}

/// Where analyses are cached: `DRAFTWRIGHT_CACHE`, or `draftwright/analyses` under the user's
/// cache folder (`XDG_CACHE_HOME`, else `~/.cache`).
fn cache_dir() -> Option<std::path::PathBuf> {
    if let Some(dir) = std::env::var_os("DRAFTWRIGHT_CACHE") {
        return Some(std::path::PathBuf::from(dir));
    }
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".cache")))?;
    Some(base.join("draftwright").join("analyses"))
}

/// The part's Analysis with its native features, from the cache when the same file (by its
/// SHA-256) was analysed by this same build, else analysed and cached. A check of an unchanged
/// set then skips recognition. A cache that cannot be read or written is passed over.
fn cached_analysis(
    step: &std::path::Path,
    part: Option<usize>,
    use_cache: bool,
) -> Result<draftwright::model::Analysis, Error> {
    use sha2::Digest;
    let key = use_cache
        .then(|| {
            let bytes = std::fs::read(step).ok()?;
            let build = std::env::current_exe()
                .and_then(|e| e.metadata())
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs());
            let mut h = sha2::Sha256::new();
            h.update(&bytes);
            h.update(format!("|{part:?}|{}|{build}", env!("CARGO_PKG_VERSION")).as_bytes());
            let digest: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
            Some(cache_dir()?.join(format!("{digest}.json")))
        })
        .flatten();
    if let Some(path) = &key
        && let Ok(text) = std::fs::read_to_string(path)
        && let Ok(analysis) = serde_json::from_str(&text)
    {
        return Ok(analysis);
    }
    let analysis = api::analyse_part(step, part, Features::Native)?;
    if let Some(path) = key
        && let Some(dir) = path.parent()
        && std::fs::create_dir_all(dir).is_ok()
        && let Ok(text) = serde_json::to_string(&analysis)
    {
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
    Ok(analysis)
}

/// The value after flag *name*, if given.
fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn read_json(path: &str) -> Result<Value, Error> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| Error::invalid(format!("cannot read {path}: {e}")))?;
    serde_json::from_str(&text).map_err(|e| Error::invalid(format!("{path} is not valid: {e}")))
}

/// Runs the command, its arguments put as [`api::call`] takes them.
fn run(args: &[String]) -> Result<Value, Error> {
    let usage = "usage: draftwright (analyse|plan|render|review|features|check|check-set|pack|diff|edit|spec|parts|spec_schema|item_options|serve) …";
    let command = args.first().ok_or_else(|| Error::invalid(usage))?;
    let arg = |i: usize| {
        args.get(i)
            .map(String::as_str)
            .ok_or_else(|| Error::invalid(usage))
    };
    let mut spec = match flag(args, "--spec") {
        Some(path) => read_json(path)?,
        None => Value::Null,
    };
    // `--standard iso|asme` chooses the drawing standard for this run, over the Spec's own
    // `sheet/standard`; the same Spec can so be drawn either way.
    let standard = flag(args, "--standard")
        .map(|s| match s.to_ascii_lowercase().as_str() {
            "iso" => Ok("iso"),
            "asme" => Ok("asme"),
            other => Err(Error::invalid(format!(
                "--standard {other:?} is not a drawing standard; give iso or asme"
            ))),
        })
        .transpose()?;
    if let Some(standard) = standard {
        if spec.is_null() {
            spec = json!({});
        }
        let Some(object) = spec.as_object_mut() else {
            return Err(Error::invalid("the Spec is not a JSON object"));
        };
        if command == "pack" {
            // A pack Spec carries the standard for all its drawings.
            object.insert("standard".into(), json!(standard));
            object
                .entry("schema")
                .or_insert_with(|| json!(draftwright::pack::SPEC_SCHEMA));
        } else {
            let sheet = object.entry("sheet").or_insert_with(|| json!({}));
            let Some(sheet) = sheet.as_object_mut() else {
                return Err(Error::invalid("the Spec's sheet is not a JSON object"));
            };
            sheet.insert("standard".into(), json!(standard));
        }
    }
    // A check reports in the standard's unit unless --units says otherwise.
    let units = flag(args, "--units").or(match standard {
        Some("asme") => Some("in"),
        Some(_) => Some("mm"),
        None => None,
    });
    let features = flag(args, "--features");
    let analysis = match flag(args, "--analysis") {
        Some(path) => read_json(path)?,
        None => Value::Null,
    };
    let no_features = args.iter().any(|a| a == "--no-features");
    let no_cache = args.iter().any(|a| a == "--no-cache");
    let part = flag(args, "--part")
        .map(|k| {
            k.parse::<usize>().map_err(|_| {
                Error::invalid(format!(
                    "--part {k:?} is not a part's index; `draftwright parts <step>` lists them"
                ))
            })
        })
        .transpose()?;
    if no_features && (features.is_some() || !analysis.is_null()) {
        return Err(Error::invalid(
            "--no-features is not taken with --features or --analysis: it means no features at all.",
        ));
    }
    let output = || flag(args, "-o").ok_or_else(|| Error::invalid("render needs -o <output stem>"));
    let formats = || {
        flag(args, "--formats")
            .unwrap_or("svg,pdf")
            .split(',')
            .collect::<Vec<_>>()
    };
    let dpi = || {
        flag(args, "--dpi")
            .map(|d| {
                d.parse::<f64>()
                    .map_err(|_| Error::invalid(format!("--dpi {d:?} is not a number")))
            })
            .transpose()
    };
    if command == "parts" {
        return guarded(command, || {
            let parts = api::parts(std::path::Path::new(arg(1)?))?;
            let keys = draftwright::pack::part_keys(&parts);
            let parts: Vec<_> = parts
                .iter()
                .zip(keys)
                .map(|(part, key)| {
                    let mut value = serde_json::to_value(part).expect("part serialises");
                    value["key"] = json!(key);
                    value
                })
                .collect();
            Ok(json!({"parts": parts}))
        });
    }
    if command == "check-set" {
        let unit =
            draftwright::inventory::Unit::read(units).map_err(Error::invalid)?;
        let dir = std::path::Path::new(arg(1)?);
        let pdf = flag(args, "--drawing")
            .ok_or_else(|| Error::invalid("check-set needs --drawing <set.pdf>"))?;
        let pages = draftwright::inventory::drawing_pages(std::path::Path::new(pdf))
            .map_err(Error::invalid)?;
        let mut steps: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
            .map_err(|e| Error::invalid(format!("cannot read {}: {e}", dir.display())))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.extension().is_some_and(|x| {
                    x.eq_ignore_ascii_case("step") || x.eq_ignore_ascii_case("stp")
                })
            })
            .collect();
        steps.sort();
        let numbers: Vec<String> = steps
            .iter()
            .map(|p| draftwright::inventory::drawing_number(p))
            .collect();
        let texts: Vec<String> = pages.iter().map(|p| p.text.clone()).collect();
        let found = draftwright::inventory::match_pages(&numbers, &texts);
        let mut parts = Vec::new();
        for ((step, number), page) in steps.iter().zip(&numbers).zip(found) {
            let name = step
                .file_name()
                .map_or(String::new(), |n| n.to_string_lossy().into());
            let Some(found) = page else {
                parts.push(json!({ "step": name, "drawing": number, "pages": [],
                    "error": "no page of the set names this drawing number" }));
                continue;
            };
            let sheets: Vec<usize> = found.pages.iter().map(|i| i + 1).collect();
            let drawing =
                draftwright::inventory::Drawing::join(found.pages.iter().map(|&i| &pages[i].drawing));
            let result = guarded("check", || {
                let analysis = cached_analysis(step, None, !no_cache)?;
                Ok(draftwright::inventory::check_report(&analysis, &drawing, unit))
            });
            let note = (found.how == "mentioned").then_some(
                "no sheet carries this number in its title block; checked against the page that mentions it",
            );
            parts.push(match result {
                Ok(mut report) => {
                    report["step"] = json!(name);
                    report["drawing"] = json!(number);
                    report["pages"] = json!(sheets);
                    if let Some(note) = note {
                        report["page_note"] = json!(note);
                    }
                    report
                }
                Err(e) => json!({ "step": name, "drawing": number, "pages": sheets,
                    "error": e.message }),
            });
        }
        let tally = |key: &str| -> u64 {
            parts
                .iter()
                .map(|p| p[key]["not_stated"].as_u64().unwrap_or(0))
                .sum()
        };
        let punch = match flag(args, "--fix") {
            Some(path) => {
                let md = draftwright::inventory::punch_list(
                    &parts,
                    unit,
                    &format!("Drawing check punch list: {}", pdf),
                );
                std::fs::write(path, md)
                    .map_err(|e| Error::invalid(format!("cannot write {path}: {e}")))?;
                json!(path)
            }
            None => Value::Null,
        };
        return Ok(json!({
            "punch_list": punch,
            "units": match unit { draftwright::inventory::Unit::Inch => "in", _ => "mm" },
            "pages": pages.len(),
            "sizes_not_stated": tally("sizes"),
            "locations_not_stated": tally("locations"),
            "only_in_notes": parts
                .iter()
                .map(|p| p["only_in_notes"].as_array().map_or(0, Vec::len))
                .sum::<usize>(),
            "parts": parts,
        }));
    }
    if matches!(command.as_str(), "features" | "check") {
        let unit =
            draftwright::inventory::Unit::read(units).map_err(Error::invalid)?;
        let drawing = if command == "check" {
            let path = flag(args, "--drawing").ok_or_else(|| {
                Error::invalid("check needs --drawing <drawing.pdf | drawing.txt>")
            })?;
            Some(
                draftwright::inventory::drawing(std::path::Path::new(path))
                    .map_err(Error::invalid)?,
            )
        } else {
            None
        };
        let given: Option<draftwright::model::Analysis> = if analysis.is_null() {
            None
        } else {
            Some(
                serde_json::from_value(analysis)
                    .map_err(|e| Error::invalid(format!("the analysis is not valid: {e}")))?,
            )
        };
        let step = if given.is_none() { Some(arg(1)?) } else { None };
        return guarded(command, || {
            let analysis = match given {
                Some(a) => a,
                None => cached_analysis(
                    std::path::Path::new(step.expect("a STEP file without an analysis")),
                    part,
                    !no_cache,
                )?,
            };
            let Some(drawing) = &drawing else {
                return Ok(draftwright::inventory::report(&analysis, unit));
            };
            let mut report = draftwright::inventory::check_report(&analysis, drawing, unit);
            if let Some(path) = flag(args, "--fix") {
                // A drawing draftwright made carries its Spec: its review's own coverage
                // findings say which Spec edits would state what is missing.
                let source = flag(args, "--drawing").expect("check has --drawing");
                let embedded = std::fs::read(source)
                    .ok()
                    .and_then(|bytes| draftwright::model::embedded_spec(&bytes).ok().flatten());
                if let (Some(spec), Some(step)) = (embedded, step) {
                    let review = api::review_part(
                        std::path::Path::new(step),
                        part,
                        Features::Native,
                        &spec,
                    )?;
                    let fixes: Vec<Value> = review
                        .lint
                        .iter()
                        .filter(|i| i.code.contains("requirement"))
                        .map(|i| json!({ "code": i.code, "message": i.message, "fix": i.fix }))
                        .collect();
                    report["spec_fixes"] = json!(fixes);
                }
                if let Some(stem) = std::path::Path::new(source).file_stem() {
                    report["drawing"] = json!(stem.to_string_lossy());
                }
                let md = draftwright::inventory::punch_list(
                    std::slice::from_ref(&report),
                    unit,
                    &format!("Drawing check punch list: {source}"),
                );
                std::fs::write(path, md)
                    .map_err(|e| Error::invalid(format!("cannot write {path}: {e}")))?;
                report["punch_list"] = json!(path);
            }
            Ok(report)
        });
    }
    if command == "pack" {
        let step = std::path::Path::new(arg(1)?);
        let dir = std::path::Path::new(
            flag(args, "-o").ok_or_else(|| Error::invalid("pack needs -o <directory>"))?,
        );
        let spec = if spec.is_null() {
            draftwright::pack::PackSpec::default()
        } else {
            serde_json::from_value(spec)
                .map_err(|e| Error::invalid(format!("invalid pack Spec: {e}")))?
        };
        let formats: Vec<_> = flag(args, "--formats")
            .unwrap_or("pdf,csv,json")
            .split(',')
            .collect();
        return guarded(command, || {
            let (report, files, timings) =
                draftwright::pack::render_profiled(step, &spec, &formats)?;
            std::fs::create_dir_all(dir)
                .map_err(|e| Error::failed(format!("cannot create {}: {e}", dir.display())))?;
            let written = api::write(&dir.join("pack"), &files)?;
            Ok(json!({ "written": written, "report": report, "timings": timings }))
        });
    }
    if command == "diff" {
        let (old, new) = (std::path::Path::new(arg(1)?), std::path::Path::new(arg(2)?));
        let dir = std::path::PathBuf::from(
            flag(args, "-o").ok_or_else(|| Error::invalid("diff needs -o <output directory>"))?,
        );
        let spec = if spec.is_null() {
            json!({ "schema": draftwright::model::SPEC_SCHEMA, "sheet": {} })
        } else {
            spec
        };
        let dpi = dpi()?.unwrap_or(api::DPI);
        let title = flag(args, "--title").map(String::from).unwrap_or_else(|| {
            new.file_name()
                .map_or("part".into(), |n| n.to_string_lossy().into_owned())
        });
        return guarded(command, || {
            let revised = draftwright::revision::diff(old, new, part, &spec)?;
            let marked = revised.marked();
            let pretty = |v: &Value| serde_json::to_vec_pretty(v).expect("JSON serialises");
            let files: Vec<(&str, Vec<u8>)> = vec![
                ("report.json", pretty(&json_of(&revised.report))),
                (
                    "report.md",
                    revised.markdown(&title, Some("changes.png")).into_bytes(),
                ),
                ("spec.json", pretty(&revised.report.carried_spec)),
                ("changes.svg", marked.svg().into_bytes()),
                ("changes.png", marked.png(dpi).map_err(Error::failed)?),
                ("drawing.svg", revised.new.drawing().svg().into_bytes()),
            ];
            std::fs::create_dir_all(&dir)
                .map_err(|e| Error::failed(format!("cannot create {}: {e}", dir.display())))?;
            let mut written = Vec::new();
            for (name, bytes) in files {
                let path = dir.join(name);
                std::fs::write(&path, bytes)
                    .map_err(|e| Error::failed(format!("cannot write {}: {e}", path.display())))?;
                written.push(path.display().to_string());
            }
            Ok(json!({
                "written": written,
                "verdict": revised.report.verdict,
                "summary": revised.report.summary,
            }))
        });
    }
    if matches!(command.as_str(), "analyse" | "plan" | "render" | "review")
        && features.is_none()
        && analysis.is_null()
        && !no_features
    {
        let step = std::path::Path::new(arg(1)?);
        let spec: Spec = if spec.is_null() {
            Spec::default()
        } else {
            serde_json::from_value(spec)
                .map_err(|e| Error::invalid(format!("the spec is not valid: {e}")))?
        };
        let native = Features::Native;
        return guarded(command, || {
            Ok(match command.as_str() {
                "analyse" => json_of(&api::analyse_part(step, part, native)?),
                "plan" => json_of(&api::plan_part(step, part, native, &spec)?),
                "review" => json_of(&api::review_part(step, part, native, &spec)?),
                _ => {
                    let (output, dpi) = (output()?, dpi()?.unwrap_or(api::DPI));
                    let (plan, files) =
                        api::render_part(step, part, native, &spec, &formats(), dpi)?;
                    let written = api::write(std::path::Path::new(output), &files)?;
                    json!({ "written": written, "problems": plan.problems })
                }
            })
        });
    }
    let request = match command.as_str() {
        // From an Analysis, plan needs no STEP file: the first argument may be a flag.
        "plan" if !analysis.is_null() => json!({
            "step": args.get(1).filter(|a| !a.starts_with("--")),
            "features": features,
            "analysis": analysis,
            "spec": spec,
            "part": part,
        }),
        "analyse" | "plan" | "review" => json!({
            "step": arg(1)?,
            "features": features,
            "analysis": analysis,
            "spec": spec,
            "part": part,
        }),
        "render" => json!({
            "step": arg(1)?,
            "features": features,
            "analysis": analysis,
            "spec": spec,
            "part": part,
            "output": output()?,
            "formats": formats(),
            "dpi": dpi()?,
        }),
        "spec_schema" => json!({}),
        "item_options" => {
            let mut request = json!({ "part_ref": arg(1)?, "id": arg(2)? });
            if !spec.is_null() {
                request["spec"] = spec;
            }
            request
        }
        "edit" => json!({ "spec": read_json(arg(1)?)?, "ops": read_json(arg(2)?)? }),
        "spec" => json!({ "drawing": arg(1)? }),
        "parts" => json!({ "step": arg(1)? }),
        other => {
            return Err(Error::invalid(format!(
                "{other:?} is not a command. {usage}"
            )));
        }
    };
    api::call(command, &request)
}

/// *op*'s result, a panic inside it answered as `failed`, as [`api::call`] answers it.
fn guarded(op: &str, f: impl FnOnce() -> Result<Value, Error>) -> Result<Value, Error> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|panic| {
        let why = panic
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| panic.downcast_ref::<String>().cloned())
            .unwrap_or_default();
        Err(Error::failed(format!("{op} stopped: {why}")))
    })
}

fn json_of(document: &impl serde::Serialize) -> Value {
    serde_json::to_value(document).expect("a document serialises")
}

/// Answers requests, one JSON object per line, until stdin closes.
fn serve() -> ExitCode {
    let stdout = std::io::stdout();
    let mut service = api::Service::default();
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(request) => reply(&request, &mut service),
            Err(e) => json!({ "id": null, "ok": false, "error": Error::invalid(e.to_string()) }),
        };
        let mut out = stdout.lock();
        if writeln!(out, "{reply}").and_then(|_| out.flush()).is_err() {
            break;
        }
    }
    ExitCode::SUCCESS
}

/// A request's reply: its result, or a structured error.
fn reply(request: &Value, service: &mut api::Service) -> Value {
    let Some(fields) = request.as_object() else {
        let error = Error::invalid("a request is a JSON object");
        return json!({ "id": null, "ok": false, "error": error });
    };
    let id = fields.get("id").cloned().unwrap_or(Value::Null);
    let result = match fields.get("op").and_then(Value::as_str) {
        Some("hello") => Ok(
            json!({ "protocol": PROTOCOL, "ops": api::OPERATIONS.iter().chain(api::SESSION_OPERATIONS.iter()).chain(api::STATELESS_OPERATIONS.iter()).collect::<Vec<_>>() }),
        ),
        Some(op) => service.call(op, request),
        None => Err(Error::invalid("a request needs \"op\"")),
    };
    match result {
        Ok(result) => json!({ "id": id, "ok": true, "result": result }),
        Err(error) => json!({ "id": id, "ok": false, "error": error }),
    }
}
