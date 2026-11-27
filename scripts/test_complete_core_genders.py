import unittest
import io
import json
import tempfile
import hashlib
from pathlib import Path
from unittest.mock import patch

from complete_core_genders import ROOT, apply_human_review, fetch_missing_revisions, main, review_report, selected_header


class HeaderCaptureTests(unittest.TestCase):
    def test_review_policy_replays_exact_columns_and_retains_extra_exceptions(self):
        rows = [{"sense_id": str(i), "lexique_gender": lexique, "candidate_gender": candidate,
                 "gender": None, "review_status": "pending_independent_human_review"}
                for i, (lexique, candidate) in enumerate((("m", "mf"), (None, "mf"), (None, None)))]
        review = {"reviewer": "LMY", "reviewed_on": "2026-11-27", "policy": "lexique_then_candidate_else_omit",
                  "decisions": [{key: row[key] for key in ("sense_id", "lexique_gender", "candidate_gender")} for row in rows]}
        apply_human_review(rows, review)
        self.assertEqual([row["gender"] for row in rows], ["m", "mf", None])
        self.assertTrue(all(row["review_status"] == "human_reviewed" for row in rows))
        rows[0]["candidate_gender"] = "f"
        with self.assertRaisesRegex(ValueError, "reviewed columns"):
            apply_human_review(rows, review)
        row = {**rows[1], "queue": "auto_accepted", "frozen_row": 2, "headword": "nom",
               "definition_fr": "Nom.", "usage": "nom", "revid": 1, "line_1_based": 1,
               "fixed_url": "https://fr.wiktionary.org/w/index.php?oldid=1", "raw_header": "{{mf}}", "rationale": "Usage."}
        self.assertIn("2 / 1", review_report([row], review))

    def test_generated_review_files_and_pinned_input_use_lf_bytes(self):
        source = {"id": "fixed", "revid": 1, "fixed_url": "https://fr.wiktionary.org/w/index.php?oldid=1"}
        items = [{"frozen_row": index + 1, "headword": "nom", "source_headword": "nom", "sense_id": str(index),
                  "source_id": "fixed", "wikt_gender": "m", "accepted_gender": "m",
                  "ipa_span": {"raw_source_excerpt": "'''nom''' {{m}}", "line_1_based": 2},
                  "def_span": {"raw_source_excerpt": "# Un nom."}, "use_hint": "m", "use": "m",
                  "lexique_gender": "m", "definition_fr": "Un nom.",
                  "queue": "auto_accepted" if index < 1947 else "conflict"} for index in range(2134)]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            resource = root / "resources/core-pilot-79"
            resource.mkdir(parents=True)
            (resource / "curated-inputs.json").write_text(json.dumps({"sources": [source]}), encoding="utf-8")
            output = root / "output"
            with patch("complete_core_genders.ROOT", root), patch("complete_core_genders.load_lexique_noun_genders", return_value={}), patch("complete_core_genders.triage_core_noun_genders", return_value={"auto_accepted": items[:1947], "residuals": items[1947:]}), patch("sys.argv", ["complete_core_genders.py", "--output", str(output)]):
                main()
            for path in output.iterdir():
                if path.is_file():
                    self.assertNotIn(b"\r", path.read_bytes(), path.name)
        resource = ROOT / "resources/core-pilot-79"
        checksum = json.loads((resource / "input-checksums.json").read_text(encoding="utf-8"))["noun-genders.json"]
        canonical = (resource / "noun-genders.json").read_bytes().replace(b"\r\n", b"\n")
        self.assertEqual(hashlib.sha256(canonical).hexdigest(), checksum)

    def test_api_batches_deduplicate_and_verify_fixed_revision_ids(self):
        items = [{"source_id": str(i)} for i in range(51)] + [{"source_id": "0"}]
        sources = {str(i): {"id": str(i), "revid": i + 1, "fixed_url": f"https://fr.wiktionary.org/w/index.php?oldid={i + 1}"} for i in range(51)}
        def response(request, timeout):
            from urllib.parse import parse_qs, urlsplit
            ids = map(int, parse_qs(urlsplit(request.full_url).query)["revids"][0].split("|"))
            return io.BytesIO(json.dumps({"query": {"pages": [{"revisions": [
                {"revid": i, "slots": {"main": {"content": f"revision {i}"}}} for i in ids]}]}}).encode())
        with tempfile.TemporaryDirectory() as directory, patch("complete_core_genders.urllib.request.urlopen", side_effect=response) as fetch:
            cache = Path(directory)
            fetch_missing_revisions(items, sources, cache)
            self.assertEqual(fetch.call_count, 2)
            self.assertEqual((cache / "51.wiki").read_text(), "revision 51")
            fetch_missing_revisions(items, sources, cache)
            self.assertEqual(fetch.call_count, 2)
        with tempfile.TemporaryDirectory() as directory, patch("complete_core_genders.urllib.request.urlopen", return_value=io.BytesIO(b'{"query":{"pages":[]}}')):
            with self.assertRaisesRegex(ValueError, "Missing fixed revisions"):
                fetch_missing_revisions(items[:1], sources, Path(directory))
            self.assertEqual(list(Path(directory).iterdir()), [])

    def test_header_belongs_to_the_replayed_selected_definition(self):
        raw = "=== {{S|nom|fr}} ===\n'''livre''' {{m}}\n# Un livre imprimé.\n=== {{S|nom|fr|num=2}} ===\n'''livre''' {{f}}\n# Un poids.\n* {{pron|livʁ|fr}}\n"
        item = {"source_headword": "livre", "def_span": {"line_1_based": 6, "raw_source_excerpt": "# Un poids."}, "ipa_span": {"line_1_based": 7}}
        self.assertEqual(selected_header(raw, item, "* {{pron|livʁ|fr}}"), ("'''livre''' {{f}}", 5))
        item["def_span"]["raw_source_excerpt"] = "# Another revision."
        self.assertEqual(selected_header(raw, item, "* {{pron|livʁ|fr}}"), ("* {{pron|livʁ|fr}}", 7))


if __name__ == "__main__":
    unittest.main()
