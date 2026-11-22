"""Check #130 at the approved resource/admission boundary."""
import csv
import gzip
import json
from pathlib import Path
import sys
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
ROWS = set(range(2451, 2501))
EVIDENCE = ROOT / "docs/evidence/issue-130"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
MEMBERSHIP = [
    ("priver", "VER"), ("élément", "NOM"), ("renverser", "VER"),
    ("cave", "NOM"), ("entourer", "VER"), ("courrier", "NOM"),
    ("virus", "NOM"), ("bagnole", "NOM"), ("concours", "NOM"),
    ("local", "ADJ"), ("terroriste", "NOM"), ("führer", "NOM"),
    ("canard", "NOM"), ("cérémonie", "NOM"), ("chic", "ADJ"),
    ("officiel", "ADJ"), ("annonce", "NOM"), ("écran", "NOM"),
    ("mardi", "NOM"), ("correspondre", "VER"), ("imposer", "VER"),
    ("piscine", "NOM"), ("discussion", "NOM"), ("piano", "NOM"),
    ("estomac", "NOM"), ("auteur", "NOM"), ("compliqué", "ADJ"),
    ("médecine", "NOM"), ("délicat", "ADJ"), ("grec", "ADJ"),
    ("grec", "NOM"), ("lesquelles", "PRO:int"), ("lesquelles", "PRO:rel"),
    ("foie", "NOM"), ("activité", "NOM"), ("bonbon", "NOM"),
    ("guerrier", "NOM"), ("sucer", "VER"), ("épreuve", "NOM"),
    ("aimable", "ADJ"), ("geler", "VER"), ("la plupart des", "ADJ:ind"),
    ("peser", "VER"), ("chagrin", "NOM"), ("communauté", "NOM"),
    ("dé", "NOM"), ("négatif", "ADJ"), ("patrie", "NOM"),
    ("éprouver", "VER"), ("pratique", "ADJ"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def retained_path(name):
    path = Path(name)
    if not path.is_absolute():
        path = ROOT / path
    if path.is_file():
        return path
    parts = name.replace("\\", "/").split("/")
    prefix = parts.index("issue-130")
    return Path(read("admission.json")["author_evidence_root"]).joinpath(*parts[prefix + 1:])


class AuthorPackage130Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_four_bound_proposals_and_honest_gates(self):
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2451, 2501)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        for group in groups:
            row = frozen[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"], group["subtitle_lemma_per_million"], group["subtitle_pos_per_million"]), tuple(row.values()))
            self.assertEqual(len(group["senses"]), 1)
            sense = group["senses"][0]
            self.assertTrue(sense["use"])
            self.assertIsNone(sense["cefr"])
            self.assertEqual(set(sense["fields"]), set(LANGUAGES))
            for name, field in sense["fields"].items():
                self.assertEqual(field["language"], LANGUAGES[name])
                self.assertTrue(field["text"].strip())
                self.assertTrue(field["modification"].strip())
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (0, 0))
        self.assertEqual((admission["pending_independent_review_fields"], admission["pending_independent_rights_fields"], admission["missing_admitted_fields"]), (200, 200, 200))
        self.assertEqual(admission["pending_independent_review_rows"], list(range(2451, 2501)))
        self.assertIsNone(admission["independent_reviewer"])
        self.assertFalse(read("field-review.json")["independent_acceptance"])
        self.assertEqual(read("field-review.json")["kind"], "author_prepared_for_separate_reviewer")
        self.assertEqual(read("reuse.json")["reused_fields"], 0)

    def test_fixed_sources_and_every_field_span_match_actual_captures(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        raw = {}
        self.assertEqual(len(captures["sources"]), 50)
        for capture in captures["sources"]:
            data = json.loads(retained_path(capture["api_path"]).read_text(encoding="utf-8-sig"))
            page = next(p for p in data["query"]["pages"] if p.get("pageid") == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            body = retained_path(capture["raw_path"]).read_bytes().decode("utf-8")
            self.assertEqual(body, revision["slots"]["main"]["content"])
            source = sources[capture["source_id"]]
            self.assertEqual((source["title"], source["revid"], source["timestamp"]), (page["title"], revision["revid"], revision["timestamp"]))
            self.assertIn("oldid=" + str(source["revid"]), source["fixed_url"])
            raw[source["id"]] = body.splitlines()
        lexical = captures["lexique_phonemic_source"]
        with gzip.open(ROOT / lexical["raw_path"], "rt", encoding="utf-8-sig") as handle:
            raw[lexical["source_id"]] = handle.read().splitlines()
        fields = {(f["sense_id"], f["field"]): f for f in read("source-excerpts.json")["fields"]}
        rights = {(f["sense_id"], f["field"]): f for f in read("rights.json")["fields"]}
        proposals = {(f["sense_id"], f["field"]): f for f in read("field-review.json")["fields"]}
        self.assertEqual(len(fields), 200)
        self.assertEqual(set(fields), set(rights))
        self.assertEqual(set(fields), set(proposals))
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                key = (sense["sense_id"], name)
                span = field["source_span"]
                self.assertEqual(span["raw_source_excerpt"], raw[field["source_id"]][span["line_1_based"] - 1])
                self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                self.assertEqual(rights[key]["exact_source_span"], span)
                self.assertTrue(rights[key]["author_rights_basis"])
                self.assertEqual(proposals[key]["proposed_text"], field["text"])
                self.assertEqual(fields[key]["source_id"], field["source_id"])
                self.assertEqual(proposals[key]["exact_source"]["source_id"], field["source_id"])
                self.assertEqual(proposals[key]["exact_source"]["revid"], sources[field["source_id"]]["revid"])
                if field["source_id"] != lexical["source_id"]:
                    lines = raw[field["source_id"]][:span["line_1_based"]]
                    language = next(l for l in reversed(lines) if l.startswith("== "))
                    self.assertEqual(language, "== {{langue|fr}} ==")
                    pos = next(l for l in reversed(lines) if l.startswith("=== {{S|"))
                    expected = {"NOM":"nom","VER":"verbe","ADJ":"adjectif","PRO:int":"pronom interrogatif","PRO:rel":"pronom relatif","ADJ:ind":"nom"}[group["lexique_cgram"]]
                    self.assertIn("{{S|" + expected + "|fr", pos)
                    if name == "ipa":
                        self.assertIn(field["text"].strip("/"), span["raw_source_excerpt"])
        for binding in read("source-excerpts.json")["supporting_evidence"]:
            line = raw[binding["source_id"]][binding["line_1_based"]-1]
            start = binding["column_1_based"]-1
            self.assertEqual(binding["reviewed_excerpt"], line[start:start+len(binding["reviewed_excerpt"])])
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(), (ROOT / "resources/core-pilot-79/COPYING").read_text(encoding="utf-8").rstrip())
        for authority in read("rights.json")["authority_captures"]:
            self.assertTrue(retained_path(authority["path"]).is_file())

    def test_contributor_pagination_is_complete_and_matches_article_credits(self):
        captures = read("source-captures.json")
        names, anonymous, continuation = {}, {}, {}
        for retained in captures["contributor_registry"]["parts"]:
            path = retained_path(retained)
            query = parse_qs(urlparse(path.with_suffix(".request.txt").read_text(encoding="utf-8")).query)
            for key, value in continuation.items():
                self.assertEqual(query[key], [str(value)])
            data = json.loads(path.read_text(encoding="utf-8"))
            self.assertNotIn("error", data)
            for page in data["query"]["pages"]:
                names.setdefault(page["title"],set()).update(p["name"] for p in page.get("contributors",[]))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
            continuation = data.get("continue",{})
        self.assertFalse(continuation)
        self.assertEqual(set(names), {p["title"] for p in captures["sources"]})
        contexts = {s["title"]:s for s in read("source-context.json")["current_content"]}
        for title in names:
            self.assertEqual(contexts[title]["current_contributors"], sorted(names[title]))
            self.assertEqual(contexts[title]["anonymous_contributors_count"], anonymous.get(title,0))

    def test_frozen_phrase_and_ambiguous_forms_keep_their_exact_sources(self):
        groups = {g["frozen_row"]:g for g in read("groups.json")}
        phrase = groups[2492]["senses"][0]
        self.assertEqual(phrase["fields"]["ipa"]["text"], "/la.ply.paʁ.dɛ/")
        lexical = read("lexique-phrase-source.json")
        self.assertEqual((lexical["original_record"]["ortho"], lexical["original_record"]["cgram"], lexical["original_record"]["phon"], lexical["original_record"]["syll"]), ("la plupart des", "ADJ:ind", "laplypaRdE", "la-ply-paR-dE"))
        manual = retained_path(lexical["manual_text_path"]).read_text(encoding="utf-8")
        self.assertIn("E paire, treize", manual)
        self.assertIn("R rue, venir", manual)
        self.assertIn("ADJ:ind Adjectif indéfini", manual)
        for row, fragment in ((2482, "Interrogative"),(2483,"Relative"),(2462,"pejorative"),(2481,"language name")):
            self.assertIn(fragment, groups[row]["senses"][0]["use"])
        self.assertTrue(all(r["alias_admission"]=="not_admitted" for r in read("source-context.json")["spelling_relations"]))


class Batch130Tests(unittest.TestCase):
    def test_frozen_groups_have_admitted_content(self):
        resource = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
        admitted = {g["frozen_row"] for g in resource["groups"]}
        self.assertEqual(ROWS - admitted, set(), "#130 frozen groups lack admitted batch content")


if __name__ == "__main__":
    author_only = "--author-only" in sys.argv
    if author_only:
        sys.argv.remove("--author-only")
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(AuthorPackage130Tests)
    if not author_only:
        suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(Batch130Tests))
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    sys.exit(0 if result.wasSuccessful() else 1)
