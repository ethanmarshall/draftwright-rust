"""tools/onshape_export.py against a fake Onshape: the calls it makes, the files it names, and
the rate-limit wait. Run: python3 tools/test_onshape_export.py"""

import io
import json
import sys
import tempfile
import unittest
import urllib.error
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import onshape_export as ox  # noqa: E402

DID, WID, EID = "a" * 24, "b" * 24, "c" * 24


class Fake:
    """Answers the export's requests; the first translation poll is still ACTIVE, and the very
    first call is rate limited once."""

    def __init__(self):
        self.seen = []
        self.polls = 0
        self.limited = False

    def __call__(self, req):
        method, url = req.get_method(), req.full_url
        self.seen.append((method, url, req.data and json.loads(req.data)))
        assert req.get_header("Authorization").startswith("Basic ")
        if not self.limited:
            self.limited = True
            raise urllib.error.HTTPError(url, 429, "slow down", {"Retry-After": "0"}, io.BytesIO())
        if "/parts/" in url:
            body = [
                {"partId": "JHD", "name": "SRT-100-116 Brush Holder", "partNumber": "SRT-100-116",
                 "elementId": EID, "bodyType": "solid"},
                {"partId": "JKD", "name": "Spacer", "partNumber": "", "elementId": EID,
                 "bodyType": "solid"},
                {"partId": "JLD", "name": "Sketch surface", "elementId": EID, "bodyType": "sheet"},
            ]
        elif url.endswith("/translations"):
            body = {"id": "t1"}
        elif "/translations/t1" in url:
            self.polls += 1
            body = {"requestState": "ACTIVE"} if self.polls == 1 else {
                "requestState": "DONE", "resultExternalDataIds": ["x1"]}
        elif "/externaldata/x1" in url:
            return io.BytesIO(b"ISO-10303-21;")
        else:
            raise AssertionError(url)
        return io.BytesIO(json.dumps(body).encode())


class ExportTest(unittest.TestCase):
    def test_url_and_names(self):
        self.assertEqual(
            ox.parse_url(f"https://cad.onshape.com/documents/{DID}/w/{WID}/e/{EID}"),
            (DID, "w", WID, EID),
        )
        self.assertEqual(ox.parse_url(f"https://cad.onshape.com/documents/{DID}/v/{WID}")[3], None)
        self.assertEqual(
            ox.file_name({"partNumber": "SRT-100-116", "name": "SRT-100-116 Brush Holder"}),
            ("SRT-100-116_Brush_Holder.step", True),
        )
        self.assertEqual(ox.file_name({"partNumber": "", "name": "Spacer"}), ("Spacer.step", False))

    def test_exports_each_solid_part(self):
        fake = Fake()
        ox.time.sleep = lambda s: None
        ox.Onshape.__init__.__defaults__ = (ox.BASE, fake)
        with tempfile.TemporaryDirectory() as out:
            import os
            os.environ["ONSHAPE_ACCESS_KEY"], os.environ["ONSHAPE_SECRET_KEY"] = "k", "s"
            code = ox.main([f"https://cad.onshape.com/documents/{DID}/w/{WID}/e/{EID}", "-o", out])
            self.assertEqual(code, 0)
            self.assertEqual(sorted(p.name for p in Path(out).iterdir()),
                             ["SRT-100-116_Brush_Holder.step", "Spacer.step"])
        posts = [body for method, url, body in fake.seen if method == "POST"]
        self.assertEqual([b["partIds"] for b in posts], ["JHD", "JKD"])
        self.assertTrue(all(b["formatName"] == "STEP" and not b["storeInDocument"] for b in posts))


if __name__ == "__main__":
    unittest.main()
