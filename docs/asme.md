# The ASME inch version

This fork (`ethanmarshall/draftwright-rust`) draws to ASME Y14.5 in inches as well as to ISO in
millimetres. Everything upstream draftwright does still works the same way; the ASME behaviour
only switches on when a Spec asks for it. This page lists what is different. The details live in
[`model.md`](model.md) ("Drawing standard: ISO or ASME") and [`inventory.md`](inventory.md).

## Turning it on

```json
{"sheet": {"standard": "asme"}}
```

Without `sheet/standard`, or with `"iso"`, a drawing is exactly what upstream would make.

## What changes on an ASME sheet

The part is still read, planned and laid out in millimetres. Only what is printed changes
([`src/standard.rs`](../src/standard.rs)).

| | ISO (upstream) | ASME (this fork) |
|---|---|---|
| Lengths | `6.35`, `12` | `.250`, `.472` (three places, no leading zero) |
| Paper | A4 to A0 | ANSI A to E (`A4` to `A0` in a Spec map to the same ranks) |
| Scales | 2:1, 1:2, 1:5 … | 4:1, 2:1, 1:1, 1:2, 1:4, 1:8 … |
| Title block | `mm`, ISO general tolerance | `IN`, ANSI size, `.XX ±.01 .XXX ±.005 ±0.5° ASME Y14.5-2018` |
| Threads | `M6` | `1/4-20 UNC`; notes take Unified designations (`#10-32 UNF-2B`) |
| Fits | ISO 286 (`H7`, `g6`) | ISO 286, plus ANSI B4.1 `RC1`-`RC9` and `LC1`-`LC11` |
| Tolerance limits | `+0.012` | `+.0005`, a zero limit keeps its sign (`-.0000`) |
| Chamfers | `C1` | `.040 X 45°` |
| Review findings | lengths in mm | lengths in inches (finding IDs unchanged) |

An ANSI fit on a hole takes the class's hole limits, and on a boss its shaft limits, and the label
always prints them: `⌀.132 LC2 +.0005 -.0000`. The table runs to 19.69 in and is checked for
consistency by a unit test in [`src/ansi_fits.rs`](../src/ansi_fits.rs). An ISO sheet refuses
these classes.

## A whole assembly

`draftwright pack assembly.step -o out --standard asme` makes the drawing package for an
assembly: a general assembly with its parts list and item balloons, then one drawing for each
distinct made part (identical parts are merged and counted; screws, nuts, washers, bearings and
the like are listed as bought and not drawn), as one `pack.pdf` plus `pack.csv` and `pack.json`.
The pack Spec's `standard` (`"standard": "asme"`, which `--standard` sets) draws every sheet to
it; a part's own `drawing` Spec can still set its own. See "Drawing packs" in `model.md`.

## Government drawing fields

For a drawing package that goes to a government customer, an ASME sheet carries:

- **CAGE CODE**, a cell in the title block's top row, blank unless `sheet/title/cage` gives it.
  An ISO sheet has no such cell and refuses the key.
- **Dual dimensions**: `sheet/dual_dimensions: true` prints each length's millimetres after its
  inches, `.250 [6.35]`. Tolerance limits stay in inches. Refused on an ISO sheet.
- **Revision block and distribution statement** as corner tables (`tables`, see `model.md`).
  [`asme-gov-spec.json`](asme-gov-spec.json) is a starting Spec: the title block fields, a
  revision block top right and a distribution statement bottom left, all blank. Fill them from
  the contract. Distribution statement A reads `DISTRIBUTION STATEMENT A. APPROVED FOR PUBLIC
  RELEASE; DISTRIBUTION IS UNLIMITED.`; B to F carry the reason, date and controlling office the
  contract names (DoDI 5230.24), each line its own table row.

```
draftwright render part.step --spec docs/asme-gov-spec.json -o out
```

## Commands added

```
draftwright features part.step --units in              # every recognised feature, sizes and centre
draftwright check part.step --drawing part.pdf         # what a drawing does not state
draftwright check-set parts/ --drawing set.pdf         # the same for a whole drawing set
```

`check` reads any drawing, not only draftwright's: it lists the sizes and the hole, boss and slot
locations the drawing's text does not state. A location counts when it is printed at its own
precision, measured from an edge, a wall the drawing locates, another located feature (a chain, a
shared centreline, an arc or bolt circle) or, for a hole in a rim, by angle.

`check-set` takes a folder of STEP files named by drawing number (`SRT-100-116_holder.step`) and
a PDF of the set. It gives each part the pages whose title block carries its number, so a drawing
of several sheets is checked as one. A cover or index page is not counted as anyone's sheet. A part
no title block carries is checked against the page that mentions it and is reported that way.

## Straight from Onshape

`tools/onshape_export.py` exports every solid part of an Onshape document as its own STEP file,
named `<part number>_<name>.step`, and can run `check-set` on them:

```
export ONSHAPE_ACCESS_KEY=...  ONSHAPE_SECRET_KEY=...
python3 tools/onshape_export.py "<Onshape URL of the document or a Part Studio>" -o parts \
    --drawing set.pdf --fix punch.md --standard asme
```

The keys are an Onshape API key pair, read from the environment only. Each part needs its Part
number set in Onshape so its file matches its drawing; parts without one are listed. Each part
costs about two Onshape API calls plus the translation's status checks.

## Other fixes

- Recognition refusals (such as `prismatic_pocket_refusals`) are kept as refusals instead of
  failing the whole part.
- Tests that need the sample corpus skip with a message when it is not there, and still fail when
  `DRAFTWRIGHT_CORPUS_REQUIRED=1`.

## Limits

- Transition, interference and force fits (ANSI `LT`, `LN`, `FN`) are not tabulated.
- `check` works from the drawing's text. It cannot tell which view a number sits in, so a value
  that happens to appear elsewhere on the sheet counts as stated. Read its findings as a list of
  things to look at, not a verdict.

## Releases

Tags are named after the upstream version they are built on: `v0.2.10-asme.1` is upstream 0.2.10
with this page's changes.
