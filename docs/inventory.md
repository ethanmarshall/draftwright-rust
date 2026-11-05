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
reports each size and each hole, boss and slot location that the text does not state as a
number. A number counts at one to four places, with or without a leading zero; a diameter is
also found as its radius and a radius as its diameter; a location is also found from the far
side or the centre; a hole on a bolt circle about the part's centre is located by the circle.

A location in `not_stated` is a prompt to look, not an error: a chain or baseline of dimensions
can fix it without printing it as one number. For a multi-sheet PDF, pass the part's own page
(`pdftotext -f N -l N -layout set.pdf part.txt`).
