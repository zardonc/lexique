"""Check ticket #142 at the approved resource/admission boundary."""
import json
import csv
from pathlib import Path
import sys
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
ROWS = set(range(3051, 3101))
EVIDENCE = ROOT / "docs/evidence/issue-142"
ADMITTED = "--admitted" in sys.argv
if ADMITTED:
    sys.argv.remove("--admitted")
MEMBERSHIP = [
    ("catastrophe", "NOM"), ("extraterrestre", "NOM"), ("jungle", "NOM"),
    ("cachet", "NOM"), ("bienvenu", "NOM"), ("chaussette", "NOM"),
    ("hommage", "NOM"), ("merder", "VER"), ("nucléaire", "ADJ"),
    ("bavarder", "VER"), ("serviteur", "NOM"), ("envisager", "VER"),
    ("lent", "ADJ"), ("pipe", "NOM"), ("découverte", "NOM"),
    ("flotte", "NOM"), ("timide", "ADJ"), ("ténèbres", "NOM"),
    ("bâtard", "NOM"), ("vitre", "NOM"), ("beurre", "NOM"),
    ("disparition", "NOM"), ("troubler", "VER"), ("conscient", "ADJ"),
    ("médaille", "NOM"), ("reproduire", "VER"), ("toast", "NOM"),
    ("tonne", "NOM"), ("réchauffer", "VER"), ("panique", "NOM"),
    ("sympathique", "ADJ"), ("flipper", "VER"), ("pouce", "NOM"),
    ("adjoint", "NOM"), ("procédure", "NOM"), ("tricher", "VER"),
    ("confondre", "VER"), ("remède", "NOM"), ("agneau", "NOM"),
    ("marrer", "VER"), ("témoignage", "NOM"), ("pose", "NOM"),
    ("affecter", "VER"), ("par-dessus", "PRE"), ("ô", "ONO"),
    ("confesser", "VER"), ("nain", "NOM"), ("damoiseau", "NOM"),
    ("carton", "NOM"), ("humilier", "VER"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def proposal(name):
    original = EVIDENCE / "authored" / name
    return json.loads(original.read_text(encoding="utf-8")) if original.exists() else read(name)


def retained(name):
    archive = Path(read("source-captures.json")["capture_root_fallback"])
    archived = archive.joinpath(*Path(name).parts[2:])
    return archived if archived.exists() else ROOT / name


class Batch142AdmissionTests(unittest.TestCase):
    @unittest.skipUnless(ADMITTED, "Canonical admission is owned by integration; use --admitted")
    def test_all_frozen_groups_have_admitted_four_field_content(self):
        resource = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
        groups = [group for group in resource["groups"] if group["frozen_row"] in ROWS]
        self.assertEqual({group["frozen_row"] for group in groups}, ROWS,
                         "Ticket #142 lacks admitted content at the resource boundary")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        for group in groups:
            for sense in group["senses"]:
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                self.assertTrue(all(field["text"].strip() for field in sense["fields"].values()))

    def test_author_handoff_preserves_every_exact_group_and_four_proposals(self):
        groups = proposal("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3051, 3101)))
        for group in groups:
            self.assertEqual(len(group["senses"]), 1)
            sense = group["senses"][0]
            self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
            self.assertEqual(sense["independent_review"], "pending")
            for field in sense["fields"].values():
                self.assertTrue(field["text"].strip())
                self.assertEqual(field["independent_review"], "pending")
        review = proposal("field-review.json")
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        self.assertEqual(len(review["fields"]), 200)
        self.assertEqual(proposal("admission.json")["scope_denominator_groups"], 4087)
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig") as handle:
            scope = list(csv.DictReader(handle))
        self.assertEqual(len(scope), 4087)
        for group in groups:
            row = scope[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"],
                              group["subtitle_lemma_per_million"], group["subtitle_pos_per_million"]), tuple(row.values()))

    def test_unresolved_modern_use_remains_four_missing_ordinary_fields(self):
        admission = proposal("admission.json")
        self.assertEqual(admission["author_unresolved_content_rows"], [3098])
        self.assertEqual(admission["annotation_exception_groups"], 0)
        self.assertEqual(admission["specialist_exception_groups"], 0)
        self.assertEqual(admission["independently_accepted_new_fields"], 0)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["pending_independent_review_fields"], 200)
        blockers = admission["unresolved_field_blockers"]
        self.assertEqual({(r["frozen_row"], r["field"]) for r in blockers},
                         {(3098, name) for name in ("french_definition", "ipa", "english", "chinese")})
        senses = {g["frozen_row"]: g["senses"][0] for g in proposal("groups.json")}
        self.assertIn("dated", senses[3098]["fields"]["english"]["text"])
        self.assertIn("vieilli", senses[3098]["fields"]["french_definition"]["text"])
        self.assertIn("se marrer", senses[3090]["fields"]["english"]["text"])
        self.assertIn("le bienvenu", senses[3055]["fields"]["english"]["text"])
        self.assertEqual(senses[3082]["fields"]["ipa"]["text"], "/fli.pe/")
        self.assertEqual(senses[3095]["fields"]["ipa"]["text"], "/o/")

    def test_all_field_spans_match_fixed_raw_api_language_pos_and_ipa(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        raw_lines = {}
        for capture in captures["sources"]:
            api = json.loads(retained(capture["api_path"]).read_text(encoding="utf-8"))
            page = next(p for p in api["query"]["pages"] if p["pageid"] == capture["pageid"])
            rev = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            raw = retained(capture["raw_path"]).read_bytes()
            self.assertEqual(raw, rev["slots"]["main"]["content"].encode("utf-8"))
            self.assertEqual(rev["timestamp"], capture["timestamp"])
            self.assertEqual(page["title"], capture["title"])
            raw_lines[capture["source_id"]] = raw.decode("utf-8").splitlines()
            self.assertEqual(sources[capture["source_id"]]["revid"], rev["revid"])
        excerpts = read("source-excerpts.json")
        selected = {(r["sense_id"], r["field"]): r for r in excerpts["fields"]}
        self.assertEqual(len(selected), 200)
        for record in excerpts["fields"] + excerpts["supporting_evidence"]:
            line = raw_lines[record["source_id"]][record["line_1_based"] - 1]
            column = record["column_1_based"] - 1
            self.assertEqual(line[column:column + len(record["reviewed_excerpt"])], record["reviewed_excerpt"])
            self.assertEqual(record["revid"], sources[record["source_id"]]["revid"])
        pos_tags = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "PRE": "préposition", "ONO": "particule"}
        for group in proposal("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                span = selected[sense["sense_id"], name]
                self.assertEqual(field["source_id"], span["source_id"])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], span["reviewed_excerpt"])
                before = raw_lines[field["source_id"]][:span["line_1_based"]]
                language = next(line for line in reversed(before) if line.startswith("== "))
                self.assertEqual(language.replace(" ", ""), "=={{langue|fr}}==")
                frame = next(line for line in reversed(before) if line.startswith("=== {{S|")
                             and not any(tag in line for tag in ("étymologie", "prononciation", "références")))
                self.assertIn("{{S|" + pos_tags[group["lexique_cgram"]] + "|fr", frame)
                if name == "ipa":
                    self.assertEqual(field["text"].strip("/"), span["ipa_variant_excerpt"])
                self.assertEqual(field["content_license"], "CC-BY-SA-4.0")

    def test_contributor_chain_and_selected_rights_are_complete_author_proposals(self):
        captures = read("source-captures.json")
        parts = captures["contributor_registry"]["parts"]
        names, anonymous = {}, {}
        for index, path in enumerate(parts):
            data = json.loads(retained(path).read_text(encoding="utf-8"))
            for page in data["query"]["pages"]:
                names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
            if index == len(parts) - 1:
                self.assertFalse(data.get("continue"))
            else:
                query = parse_qs(urlparse(retained(parts[index + 1]).with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                for key, value in data["continue"].items():
                    self.assertEqual(query[key], [str(value)])
        self.assertEqual(set(names), {s["title"] for s in captures["sources"]})
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        rights = proposal("rights.json")
        self.assertIsNone(rights["reviewer"])
        self.assertEqual(len(rights["fields"]), 200)
        for field in rights["fields"]:
            self.assertEqual(field["independent_review"], "pending")
            self.assertIsNone(field["reviewer"])
            self.assertTrue(field["modification"])
        carton = next(s for s in rights["sources"] if s["title"] == "carton")
        cfc = next(m for m in carton["reference_markers"] if "Import:CFC" in m["raw_reference"])
        self.assertEqual(cfc["line_1_based"], 53)
        self.assertEqual(cfc["author_rights_proposal"], "excluded_from_selected_content")
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())


if __name__ == "__main__":
    unittest.main()
