"""draftwright on the web: upload STEP files, choose ISO or ASME, get drawings back; or check a
drawing set against its parts. A thin FastAPI layer over the `draftwright` command line.

Work is slow (tens of seconds a part) and CPU-bound, so each request becomes a job run one at a
time on a worker thread; the page polls the job and downloads its files.

Environment:
  DRAFTWRIGHT_BIN   the draftwright binary (default: draftwright on PATH)
  APP_PASSWORD      when set, every request needs HTTP Basic auth with this password
  MAX_UPLOAD_MB     total upload size per job (default 100)
  JOB_TIMEOUT       seconds one draftwright command may run (default 900)
  JOB_TTL           seconds a finished job's files are kept (default 7200)
  SOURCE_URL        this program's source, linked from the page (AGPL-3.0)
"""

from __future__ import annotations

import base64
import hmac
import io
import json
import os
import re
import secrets
import shutil
import subprocess
import tempfile
import threading
import time
import zipfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from fastapi import FastAPI, File, Form, HTTPException, Request, UploadFile
from fastapi.responses import FileResponse, JSONResponse, Response, StreamingResponse
from fastapi.staticfiles import StaticFiles

BIN = os.environ.get("DRAFTWRIGHT_BIN", "draftwright")
PASSWORD = os.environ.get("APP_PASSWORD", "")
MAX_UPLOAD = int(os.environ.get("MAX_UPLOAD_MB", "100")) * 1024 * 1024
TIMEOUT = int(os.environ.get("JOB_TIMEOUT", "900"))
TTL = int(os.environ.get("JOB_TTL", "7200"))
SOURCE_URL = os.environ.get("SOURCE_URL", "https://github.com/ethanmarshall/draftwright-rust")
ROOT = Path(os.environ.get("JOB_ROOT", tempfile.gettempdir())) / "draftwright-jobs"
ROOT.mkdir(parents=True, exist_ok=True)
HERE = Path(__file__).parent
GOV_SPEC = next(
    (p for p in (HERE / "asme-gov-spec.json", HERE.parent / "docs" / "asme-gov-spec.json") if p.exists()),
    HERE / "asme-gov-spec.json",
)

app = FastAPI(title="draftwright", docs_url=None, redoc_url=None)
worker = ThreadPoolExecutor(max_workers=1)
jobs: dict[str, dict] = {}
lock = threading.Lock()


@app.middleware("http")
async def basic_auth(request: Request, call_next):
    """With APP_PASSWORD set, every path but /healthz needs it (any user name)."""
    if PASSWORD and request.url.path != "/healthz":
        header = request.headers.get("authorization", "")
        ok = False
        if header.startswith("Basic "):
            try:
                _, _, given = base64.b64decode(header[6:]).decode().partition(":")
                ok = hmac.compare_digest(given, PASSWORD)
            except Exception:
                ok = False
        if not ok:
            return Response(
                "Sign in with the app password.",
                status_code=401,
                headers={"WWW-Authenticate": 'Basic realm="draftwright"'},
            )
    return await call_next(request)


@app.get("/healthz")
def healthz():
    return {"ok": True}


@app.get("/api/info")
def info():
    version = run([BIN, "--version"], timeout=30).stdout.strip()
    return {"version": version, "source": SOURCE_URL}


def run(cmd: list[str], timeout: int = TIMEOUT, cwd: Path | None = None):
    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout, cwd=cwd)


def safe_name(name: str) -> str:
    """A file name kept to its base and to characters a drawing number uses."""
    base = Path(name or "file").name
    base = re.sub(r"[^A-Za-z0-9._ -]+", "_", base).strip(" .") or "file"
    return base[:120]


def new_job(kind: str) -> tuple[str, Path]:
    jid = secrets.token_urlsafe(12)
    folder = ROOT / jid
    (folder / "in").mkdir(parents=True)
    (folder / "out").mkdir()
    with lock:
        jobs[jid] = {"id": jid, "kind": kind, "state": "queued", "created": time.time(),
                     "steps": [], "files": [], "error": None, "summary": None}
    return jid, folder


def update(jid: str, **fields):
    with lock:
        jobs[jid].update(fields)


def step_status(jid: str, name: str, state: str, detail: str | None = None):
    with lock:
        steps = jobs[jid]["steps"]
        for s in steps:
            if s["name"] == name:
                s.update(state=state, detail=detail)
                break
        else:
            steps.append({"name": name, "state": state, "detail": detail})


async def save_uploads(files: list[UploadFile], into: Path, suffixes: tuple[str, ...]) -> list[Path]:
    saved, total = [], 0
    for f in files:
        name = safe_name(f.filename)
        if not name.lower().endswith(suffixes):
            raise HTTPException(400, f"{name}: expected {' or '.join(suffixes)}")
        data = await f.read()
        total += len(data)
        if total > MAX_UPLOAD:
            raise HTTPException(413, f"uploads are limited to {MAX_UPLOAD // (1024 * 1024)} MB")
        path = into / name
        path.write_bytes(data)
        saved.append(path)
    return saved


def spec_for(standard: str, dual: bool, gov: bool, title: dict) -> dict:
    spec = json.loads(GOV_SPEC.read_text()) if gov and standard == "asme" else {}
    sheet = spec.setdefault("sheet", {})
    sheet["standard"] = standard
    if dual and standard == "asme":
        sheet["dual_dimensions"] = True
    fields = sheet.setdefault("title", {})
    for key, value in title.items():
        if value and (key != "cage" or standard == "asme"):
            fields[key] = value
    # Blank fields are no value: drop them so the drawing's own defaults (number, title) apply.
    sheet["title"] = {k: v for k, v in fields.items() if v}
    if not sheet["title"]:
        del sheet["title"]
    return spec


def render_job(jid: str, folder: Path, steps: list[Path], spec: dict, formats: str):
    update(jid, state="running")
    spec_path = folder / "spec.json"
    spec_path.write_text(json.dumps(spec, indent=2))
    out = folder / "out"
    done = 0
    for step in steps:
        stem = step.stem
        step_status(jid, step.name, "running")
        try:
            r = run([BIN, "render", str(step), "--spec", str(spec_path), "-o", str(out / stem),
                     "--formats", formats, "--dpi", "110"])
            result = json.loads(r.stdout or "{}")
        except subprocess.TimeoutExpired:
            step_status(jid, step.name, "failed", f"took longer than {TIMEOUT} s")
            continue
        except json.JSONDecodeError:
            step_status(jid, step.name, "failed", (r.stderr or "no output")[-300:])
            continue
        if r.returncode != 0 or result.get("ok") is False:
            message = (result.get("error") or {}).get("message") or r.stderr[-300:]
            step_status(jid, step.name, "failed", message)
            continue
        problems = result.get("problems") or []
        serious = [p for p in problems if p.get("severity") == "error"]
        detail = f"{len(serious)} errors, {len(problems) - len(serious)} notes" if problems else None
        step_status(jid, step.name, "done", detail)
        done += 1
    files = sorted(p.name for p in out.iterdir())
    update(jid, state="done", files=files,
           summary=f"{done} of {len(steps)} drawings made ({spec['sheet']['standard'].upper()})")


def check_job(jid: str, folder: Path, drawing: Path, standard: str):
    update(jid, state="running")
    step_status(jid, "check-set", "running")
    out = folder / "out"
    punch = out / "punch_list.md"
    try:
        r = run([BIN, "check-set", str(folder / "in"), "--drawing", str(drawing), "--standard",
                 standard, "--fix", str(punch)])
        report = json.loads(r.stdout or "{}")
    except subprocess.TimeoutExpired:
        step_status(jid, "check-set", "failed", f"took longer than {TIMEOUT} s")
        update(jid, state="failed", error="the check took too long")
        return
    except json.JSONDecodeError:
        step_status(jid, "check-set", "failed", r.stderr[-300:])
        update(jid, state="failed", error="the check failed")
        return
    if report.get("ok") is False:
        message = report["error"]["message"]
        step_status(jid, "check-set", "failed", message)
        update(jid, state="failed", error=message)
        return
    (out / "check_report.json").write_text(json.dumps(report, indent=2))
    step_status(jid, "check-set", "done")
    update(
        jid,
        state="done",
        files=sorted(p.name for p in out.iterdir()),
        summary=(f"{report.get('sizes_not_stated', 0)} sizes and "
                 f"{report.get('locations_not_stated', 0)} locations not stated; "
                 f"{report.get('only_in_notes', 0)} only in notes"),
        punch_list=punch.read_text() if punch.exists() else "",
    )


def guarded(fn, jid, *args):
    try:
        fn(jid, *args)
    except Exception as e:  # a job's failure is reported, never lost
        update(jid, state="failed", error=f"{type(e).__name__}: {e}")


@app.post("/api/render")
async def render(
    steps: list[UploadFile] = File(...),
    standard: str = Form("asme"),
    dual: bool = Form(False),
    gov: bool = Form(False),
    formats: str = Form("pdf,svg,png"),
    cage: str = Form(""),
    revision: str = Form(""),
    drawn_by: str = Form(""),
    company: str = Form(""),
):
    if standard not in ("iso", "asme"):
        raise HTTPException(400, "standard is iso or asme")
    formats = ",".join(f for f in formats.split(",") if f in ("pdf", "svg", "png")) or "pdf"
    jid, folder = new_job("render")
    saved = await save_uploads(steps, folder / "in", (".step", ".stp"))
    spec = spec_for(standard, dual, gov, {"cage": cage.strip(), "revision": revision.strip(),
                                          "drawn_by": drawn_by.strip(), "company": company.strip()})
    for s in saved:
        step_status(jid, s.name, "queued")
    worker.submit(guarded, render_job, jid, folder, saved, spec, formats)
    return {"id": jid}


@app.post("/api/check")
async def check(
    steps: list[UploadFile] = File(...),
    drawing: UploadFile = File(...),
    standard: str = Form("asme"),
):
    if standard not in ("iso", "asme"):
        raise HTTPException(400, "standard is iso or asme")
    jid, folder = new_job("check")
    await save_uploads(steps, folder / "in", (".step", ".stp"))
    [pdf] = await save_uploads([drawing], folder, (".pdf",))
    step_status(jid, "check-set", "queued")
    worker.submit(guarded, check_job, jid, folder, pdf, standard)
    return {"id": jid}


@app.get("/api/jobs/{jid}")
def job(jid: str):
    with lock:
        j = jobs.get(jid)
        if not j:
            raise HTTPException(404, "no such job (finished jobs are kept for a while, then removed)")
        queued = sum(1 for o in jobs.values() if o["state"] in ("queued", "running")
                     and o["created"] < j["created"])
        return {**j, "ahead": queued}


@app.get("/api/jobs/{jid}/files/{name}")
def job_file(jid: str, name: str):
    path = ROOT / safe_name(jid) / "out" / safe_name(name)
    if not path.is_file():
        raise HTTPException(404, "no such file")
    return FileResponse(path, filename=path.name)


@app.get("/api/jobs/{jid}/zip")
def job_zip(jid: str):
    out = ROOT / safe_name(jid) / "out"
    if not out.is_dir():
        raise HTTPException(404, "no such job")
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as z:
        for p in sorted(out.iterdir()):
            z.write(p, p.name)
    buf.seek(0)
    return StreamingResponse(buf, media_type="application/zip",
                             headers={"Content-Disposition": f'attachment; filename="drawings-{jid[:6]}.zip"'})


def sweep():
    """Remove finished jobs older than JOB_TTL."""
    while True:
        time.sleep(300)
        now = time.time()
        with lock:
            old = [j for j, v in jobs.items()
                   if v["state"] in ("done", "failed") and now - v["created"] > TTL]
            for j in old:
                jobs.pop(j, None)
        for j in old:
            shutil.rmtree(ROOT / j, ignore_errors=True)


threading.Thread(target=sweep, daemon=True).start()


@app.exception_handler(HTTPException)
async def http_error(_, exc: HTTPException):
    return JSONResponse({"error": exc.detail}, status_code=exc.status_code)


app.mount("/", StaticFiles(directory=HERE / "static", html=True), name="static")
