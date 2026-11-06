#!/usr/bin/env python3
"""Export every part of an Onshape document as its own STEP file, named by part number, and
optionally check them against a drawing set with `draftwright check-set`.

    export ONSHAPE_ACCESS_KEY=...  ONSHAPE_SECRET_KEY=...
    python3 tools/onshape_export.py "https://cad.onshape.com/documents/<did>/w/<wid>/e/<eid>" \
        -o parts/ [--drawing set.pdf --fix punch.md]

The keys are an Onshape API key pair (Onshape: My account > Developer > API keys, or the
developer portal). They are read from the environment and never written anywhere. Every request
counts against the account's Onshape API allocation; a document of N parts costs about 2N + 2
calls.

Each file is named `<part number>_<part name>.step`, so `check-set` matches it to the drawing
whose title block carries that number. A part with no part number is named by its name alone
and reported, since check-set then finds its drawing only if the name starts with the number.

With no element in the URL, every Part Studio in the workspace or version is exported. Only the
standard library is used, so it runs wherever Python 3.8+ does.
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

BASE = os.environ.get("ONSHAPE_BASE", "https://cad.onshape.com").rstrip("/")


class Onshape:
    """A small Onshape REST client: Basic auth with an API key pair, JSON in and out, and a wait
    on HTTP 429 as Onshape's rate limit asks."""

    def __init__(self, access: str, secret: str, base: str = BASE, opener=None):
        token = base64.b64encode(f"{access}:{secret}".encode()).decode()
        self.auth = f"Basic {token}"
        self.base = base
        self.open = opener or urllib.request.urlopen
        self.calls = 0

    def request(self, method: str, path: str, body=None, raw: bool = False):
        data = None if body is None else json.dumps(body).encode()
        for attempt in range(6):
            req = urllib.request.Request(self.base + path, data=data, method=method)
            req.add_header("Authorization", self.auth)
            req.add_header("Accept", "application/octet-stream" if raw else "application/json")
            if data is not None:
                req.add_header("Content-Type", "application/json")
            self.calls += 1
            try:
                with self.open(req) as resp:
                    payload = resp.read()
                return payload if raw else json.loads(payload or b"null")
            except urllib.error.HTTPError as e:
                if e.code == 429 and attempt < 5:
                    time.sleep(float(e.headers.get("Retry-After") or 2 ** attempt))
                    continue
                detail = e.read().decode(errors="replace")[:300]
                raise SystemExit(f"Onshape {method} {path}: HTTP {e.code} {detail}") from None
        raise SystemExit(f"Onshape {method} {path}: still rate limited")


def parse_url(url: str):
    """(did, 'w'|'v', wvid, eid or None) from an Onshape document URL."""
    m = re.search(r"/documents/([0-9a-f]{24})/([wv])/([0-9a-f]{24})(?:/e/([0-9a-f]{24}))?", url)
    if not m:
        raise SystemExit(
            "not an Onshape document URL: expected .../documents/<did>/w/<wid>[/e/<eid>]"
        )
    return m.group(1), m.group(2), m.group(3), m.group(4)


def slug(text: str) -> str:
    return re.sub(r"[^A-Za-z0-9.-]+", "_", text).strip("_") or "part"


def file_name(part: dict) -> tuple[str, bool]:
    """The STEP file's name, and whether it carries a part number."""
    number = (part.get("partNumber") or "").strip()
    name = (part.get("name") or part.get("partId") or "part").strip()
    if number:
        rest = name[len(number):] if name.startswith(number) else name
        rest = slug(rest)
        return (f"{slug(number)}_{rest}.step" if rest else f"{slug(number)}.step"), True
    return f"{slug(name)}.step", False


def parts(api: Onshape, did: str, wv: str, wvid: str, eid: str | None) -> list[dict]:
    """The solid parts of the document (or of one Part Studio)."""
    path = f"/api/v6/parts/d/{did}/{wv}/{wvid}"
    if eid:
        path += f"/e/{eid}"
    found = api.request("GET", path + "?withThumbnails=false") or []
    return [p for p in found if p.get("bodyType", "solid") == "solid"]


def export_step(api: Onshape, did: str, wv: str, wvid: str, part: dict, poll: float = 1.0) -> bytes:
    """One part as STEP, through a translation that is polled until done."""
    eid = part["elementId"]
    started = api.request(
        "POST",
        f"/api/v6/partstudios/d/{did}/{wv}/{wvid}/e/{eid}/translations",
        {
            "formatName": "STEP",
            "partIds": part["partId"],
            "storeInDocument": False,
            "translate": True,
            "stepUnit": "MILLIMETER",
            "flattenAssemblies": False,
        },
    )
    tid = started["id"]
    wait = poll
    while True:
        state = api.request("GET", f"/api/v6/translations/{tid}")
        if state.get("requestState") == "DONE":
            break
        if state.get("requestState") == "FAILED":
            raise SystemExit(f"STEP export of {part.get('name')} failed: {state.get('failureReason')}")
        time.sleep(wait)
        wait = min(wait * 1.5, 10.0)
    fid = state["resultExternalDataIds"][0]
    return api.request("GET", f"/api/v6/documents/d/{did}/externaldata/{fid}", raw=True)


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("url", help="Onshape document URL (a Part Studio tab, or the document)")
    ap.add_argument("-o", "--out", default="onshape_parts", help="folder for the STEP files")
    ap.add_argument("--drawing", help="a drawing set PDF: run draftwright check-set on the export")
    ap.add_argument("--fix", help="with --drawing: write the punch list here")
    ap.add_argument("--standard", choices=["iso", "asme"], help="with --drawing: the check's standard")
    ap.add_argument("--draftwright", default="draftwright", help="the draftwright binary")
    args = ap.parse_args(argv)

    access, secret = os.environ.get("ONSHAPE_ACCESS_KEY"), os.environ.get("ONSHAPE_SECRET_KEY")
    if not access or not secret:
        print("Set ONSHAPE_ACCESS_KEY and ONSHAPE_SECRET_KEY (an Onshape API key pair).", file=sys.stderr)
        return 2
    did, wv, wvid, eid = parse_url(args.url)
    api = Onshape(access, secret)
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    unnumbered = []
    found = parts(api, did, wv, wvid, eid)
    if not found:
        print("No solid parts found.", file=sys.stderr)
        return 1
    used: set[str] = set()
    for part in found:
        name, numbered = file_name(part)
        if name in used:  # two parts of one number: keep both
            name = name[:-5] + f"_{slug(part['partId'])}.step"
        used.add(name)
        (out / name).write_bytes(export_step(api, did, wv, wvid, part))
        print(f"exported {name}")
        if not numbered:
            unnumbered.append(name)
    if unnumbered:
        print(
            "No part number (set Part number in each part's properties to match its drawing): "
            + ", ".join(unnumbered),
            file=sys.stderr,
        )
    print(f"{len(found)} parts, {api.calls} Onshape API calls", file=sys.stderr)
    if args.drawing:
        cmd = [args.draftwright, "check-set", str(out), "--drawing", args.drawing]
        if args.fix:
            cmd += ["--fix", args.fix]
        if args.standard:
            cmd += ["--standard", args.standard]
        return subprocess.call(cmd)
    return 0


if __name__ == "__main__":
    sys.exit(main())
