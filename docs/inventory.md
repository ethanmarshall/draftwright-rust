# Feature inventory and drawing check

Two commands that do not plan or draw anything. Both take `--units in` (the default) or
`--units mm`, `--part K` for one part of an assembly, and `--analysis a.json` in place of
analysing the STEP file again.

## `draftwright features part.step`

Every recognised feature with its named sizes (`diameter`, `radius`, `width`, `depth`, …) and
its centre measured from the part's minimum corner (`from_min`, X Y Z), as a hole table measures
it. Use it to see what recognition found and to read hole positions off the model.

```
draftwright features SRT-100-118_pressure_finger.step --units in
```

## `draftwright check part.step --drawing drawing.pdf`

Reads the drawing's text (a `.pdf` through `pdftotext` from poppler-utils, or a `.txt`) and
reports each size and each hole, boss and slot location that the drawing does not state.

**Where a number sits.** From a PDF, each word's position is read (`pdftotext -bbox`), and:

- the title block (the region right of and below its labels: DWG, SCALE, SHEET, TOLERANCES …)
  and the zone border are left out, so `±.005` in a tolerance block or a zone number never states
  a size;
- prose lines are notes: a run of words with lower-case letters (drawing callouts are capitals),
  or six or more words mostly of letters. A number found only in a note still counts as stated,
  and is listed in `only_in_notes`: a dimension given only in a notes block or in modelling
  instructions is worth a look. Everything else is a view.

A `.txt` drawing has no positions, so all of it counts as a view (`read_layout: false`).

**Sizes** count when the text holds the number at one to four places, with or without a
leading zero. A diameter is also found as its radius and a radius as its diameter. A THRU hole's
depth and a countersink's depth are not asked for.

**Locations** must be printed at the precision they are (`.297` or `.2970`, never `.3`), and a
bare whole number never counts, so a page dense with numbers does not locate a part by accident.
A coordinate along the feature's own axis, on an edge or on the centre plane needs no number. A
location is found when it is stated:

- from an edge or the centre;
- from one of the part's own walls (a plate's face, a step's wall, a level) that the drawing
  locates from an edge, as a hole measured from a flange's inner face;
- as a chain from another located feature (the distance between them is printed), or on the
  same centreline as one;
- as the radius or diameter of a circle about the centre, a corner, an edge's midpoint or a
  located feature (a bolt circle, holes on an arc); its angle is read by eye;
- by its angle, for a radial hole whose axis runs through the part's centre (a wheel's rim holes).

`located_through` lists the locations found by a chain, a wall, a circle or an angle, with how.
`not_stated` lists the rest: check those by eye.

## `draftwright check-set parts/ --drawing set.pdf`

`check` for every STEP file in a folder against its sheets of a drawing set, in one run. A
STEP file's drawing number is its name up to the first `_` (`SRT-100-118_pressure_finger.step` is
`SRT-100-118`). A sheet belongs to the number its title block prints, the number printed last on
the page; a drawing of several sheets is checked against all of them. A part no title block
carries is checked against the page that mentions it with the fewest other numbers, and says so
(`page_note`). The result totals what is not stated across the set (and what is stated only in
notes) and lists each part's report.
