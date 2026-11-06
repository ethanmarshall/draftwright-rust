# draftwright web app

A small FastAPI app over the `draftwright` command line, for using it from a browser (Edge,
Chrome, any) without installing anything:

- **Make drawings**: upload STEP files, choose **ASME (inches)** or **ISO (mm)**, optionally dual
  dimensions, a revision block and distribution statement, the CAGE code and revision; get PDF,
  SVG and PNG back, one drawing per file. Or upload one assembly STEP for the whole package:
  general assembly with parts list and balloons, then a drawing per made part.
- **Check a drawing set**: upload the parts and the set's PDF (from any CAD program); get the
  `check-set` report and its punch list.

Run locally (needs `pdftotext` from poppler-utils and a built `draftwright`):

```
pip install -r web/requirements.txt
cd web && DRAFTWRIGHT_BIN=../target/release/draftwright uvicorn app:app --port 8000
```

Or with Docker, from the repository root: `docker build -t draftwright-web .` and
`docker run -p 8000:8000 -e APP_PASSWORD=choose-one draftwright-web`.

Set `APP_PASSWORD` on any deployment reachable from the internet: every page then asks for it
(any user name). A part takes about 45 s and 650 MB of memory, so host it on at least 1 CPU and
2 GB. Jobs run one at a time; finished jobs' files are removed after two hours (`JOB_TTL`).

The program is AGPL-3.0: the page links its source (`SOURCE_URL`), as the licence asks of a
modified version offered over a network.
