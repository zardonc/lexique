"""Check #139 at the approved resource/admission seam."""
import csv
import json
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-139"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
# Exact word/subclass membership from ticket #139, rows 2901–2950.
MEMBERSHIP = [
    ("dealer", "NOM"), ("portefeuille", "NOM"), ("appliquer", "VER"),
    ("bonté", "NOM"), ("dresser", "VER"), ("elle-même", "PRO:per"),
    ("redevenir", "VER"), ("angoisse", "NOM"), ("bouffe", "NOM"),
    ("casier", "NOM"), ("bible", "NOM"), ("démissionner", "VER"),
    ("envier", "VER"), ("pension", "NOM"), ("plaisanterie", "NOM"),
    ("vertu", "NOM"), ("pasteur", "NOM"), ("stylo", "NOM"),
    ("candidat", "NOM"), ("choper", "VER"), ("capturer", "VER"),
    ("muet", "ADJ"), ("apparence", "NOM"), ("musicien", "NOM"),
    ("bol", "NOM"), ("évacuer", "VER"), ("technologie", "NOM"),
    ("suffisant", "ADJ"), ("saleté", "NOM"), ("guide", "NOM"),
    ("international", "ADJ"), ("milliard", "NOM"), ("poing", "NOM"),
    ("citer", "VER"), ("salir", "VER"), ("soumettre", "VER"),
    ("célébrer", "VER"), ("gagnant", "NOM"), ("exécution", "NOM"),
    ("gang", "NOM"), ("ministère", "NOM"), ("réception", "NOM"),
    ("injuste", "ADJ"), ("sommet", "NOM"), ("préoccuper", "VER"),
    ("trouille", "NOM"), ("ralentir", "VER"), ("tube", "NOM"),
    ("envahir", "VER"), ("peste", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch139Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_complete_resource_records(self):
        if (EVIDENCE / "groups.json").exists():
            groups = read("groups.json")
        else:
            canonical = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
            groups = [g for g in canonical["groups"] if 2901 <= g["frozen_row"] <= 2950]
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP,
                         "The frozen #139 resource slice lacks complete batch content")
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2901, 2951)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group in groups:
            with self.subTest(row=group["frozen_row"]):
                row = frozen[group["frozen_row"] - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]), (row["word"], row["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], row[frequency])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertTrue(sense["use"].strip())
                self.assertIsNone(sense["cefr"])
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"].strip())

    def test_fields_and_constructions_bind_to_retained_french_sources(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        raw = {}
        capture_root = Path(captures["capture_root"])
        self.assertEqual(captures["capture_root_fallback"], str(capture_root))
        self.assertEqual(str(capture_root), read("admission.json")["author_evidence_root"])
        self.assertEqual(capture_root.name, "issue-139")
        self.assertEqual(capture_root.parent.name, "author-evidence")

        def retained(name):
            return capture_root.joinpath(*Path(name).parts[2:])

        self.assertEqual(len(captures["sources"]), 50)
        for capture in captures["sources"]:
            data = json.loads(retained(capture["api_path"]).read_text(encoding="utf-8-sig"))
            page = next(p for p in data["query"]["pages"] if p["pageid"] == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            body = revision["slots"]["main"]["content"]
            self.assertEqual(page["title"], capture["title"])
            self.assertEqual(revision["timestamp"], capture["timestamp"])
            self.assertEqual(body, retained(capture["raw_path"]).read_text(encoding="utf-8"))
            source = next(s for s in sources.values() if s["pageid"] == capture["pageid"])
            self.assertEqual(source["revid"], capture["revid"])
            self.assertEqual(parse_qs(urlparse(source["fixed_url"]).query)["oldid"], [str(capture["revid"])])
            raw[source["id"]] = body.splitlines()
        excerpts = read("source-excerpts.json")
        self.assertEqual(len(excerpts["fields"]), 200)
        for record in [*excerpts["fields"], *excerpts["supporting_evidence"]]:
            line = raw[record["source_id"]][record["line_1_based"] - 1]
            column = record["column_1_based"] - 1
            self.assertEqual(line[column:column + len(record["reviewed_excerpt"])], record["reviewed_excerpt"])
            self.assertFalse(record["reviewed_excerpt"].lstrip().startswith("#*"))
        by_field = {(r["sense_id"], r["field"]): r for r in excerpts["fields"]}
        self.assertEqual(len(by_field), 200)
        pos_frames = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "PRO:per": "pronom personnel"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            support = [r for r in excerpts["supporting_evidence"] if r["sense_id"] == sense["sense_id"]]
            self.assertTrue(any(r["reviewed_excerpt"] == "== {{langue|fr}} ==" for r in support))
            self.assertTrue(any("{{S|" + pos_frames[group["lexique_cgram"]] + "|fr" in r["reviewed_excerpt"] for r in support))
            for name, field in sense["fields"].items():
                exact = by_field[sense["sense_id"], name]
                self.assertEqual(field["source_id"], exact["source_id"])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], exact["reviewed_excerpt"])
                self.assertEqual(field["source_span"]["line_1_based"], exact["line_1_based"])
                self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                if name == "ipa":
                    self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
        contributor_names = {}
        anonymous_counts = {}
        chains = {}
        for name in captures["contributor_registry"]["parts"]:
            path = retained(name)
            data = json.loads(path.read_text(encoding="utf-8"))
            prefix, part = path.name.removesuffix(".api.json").rsplit("-", 1)
            chains.setdefault(prefix, {})[int(part)] = (path, data)
            for page in data["query"]["pages"]:
                contributor_names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous_counts[page["title"]] = page["anoncontributors"]
        self.assertEqual(set(chains), {"contributors"})
        for parts in chains.values():
            self.assertEqual(sorted(parts), list(range(1, len(parts) + 1)))
            for part, (path, data) in parts.items():
                continuation = data.get("continue")
                if part == len(parts):
                    self.assertFalse(continuation)
                else:
                    query = parse_qs(urlparse(parts[part + 1][0].with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                    for key, value in continuation.items():
                        self.assertEqual(query[key], [str(value)])
        for context in contexts.values():
            self.assertEqual(context["current_contributors"], sorted(contributor_names[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(contributor_names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous_counts.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])

    def test_original_proposals_keep_review_and_admission_pending(self):
        admission = read("authored/admission.json")
        review = read("authored/field-review.json")
        rights = read("authored/rights.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (0, 0))
        self.assertEqual((admission["pending_independent_review_fields"], admission["pending_independent_rights_fields"]), (200, 200))
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual((admission["reused_groups"], admission["reused_fields"]), (0, 0))
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        self.assertEqual(len(review["fields"]), 200)
        self.assertEqual(len(review["usage"]), 50)
        self.assertEqual(len(review["spelling_relations"]), 50)
        self.assertEqual(len(rights["fields"]), 200)
        for field in review["fields"]:
            self.assertEqual(field["accepted_text_quality_status"], "pending")
            self.assertIsNone(field["reviewer"])
        for field in rights["fields"]:
            self.assertEqual(field["independent_review"], "pending")
            self.assertIsNone(field["reviewer"])
        for relation in review["spelling_relations"]:
            self.assertEqual(relation["alias_admission"], "not_admitted")

    def test_selected_pos_register_spelling_and_original_rights_boundaries(self):
        senses = {g["frozen_row"]: g["senses"][0] for g in read("groups.json")}
        self.assertEqual(senses[2901]["fields"]["ipa"]["text"], "/di.lœʁ/")
        self.assertEqual(senses[2906]["source_headword"], "elle-même")
        self.assertEqual(senses[2906]["fields"]["ipa"]["text"], "/ɛl.mɛm/")
        self.assertIn("emphatic", senses[2906]["fields"]["english"]["text"])
        self.assertIn("printed copy", senses[2911]["fields"]["english"]["text"])
        self.assertIn("1,000,000,000", senses[2932]["fields"]["english"]["text"])
        self.assertIn("十亿", senses[2932]["fields"]["chinese"]["text"])
        for row, restriction in [(2901, "slang"), (2909, "informal"), (2920, "France"),
                                 (2922, "temporarily"), (2940, "masculine"), (2946, "informal")]:
            self.assertIn(restriction, senses[row]["fields"]["english"]["text"])
        wallet = next(s for s in read("rights.json")["sources"] if s["title"] == "portefeuille")
        cfc = next(m for m in wallet["reference_markers"] if "Import:CFC" in m["raw_reference"])
        self.assertEqual(cfc["line_1_based"], 62)
        self.assertEqual(cfc["author_rights_proposal"], "excluded_from_selected_content")
        self.assertFalse(any(r["line_1_based"] == 62 for r in wallet["selected_locations"]))
        self.assertNotIn("Fichier:", senses[2902]["fields"]["french_definition"]["source_span"]["raw_source_excerpt"])
        layers = [layer for source in read("sources.json")["sources"]
                  for layer in source["original_historical_dictionary_layers"]]
        self.assertEqual(len(layers), 31)
        for layer in layers:
            self.assertEqual(layer["original_authors"], "Académie française")
            self.assertTrue(layer["conservative_expiry"].startswith("2006-01-01"))
            self.assertIn("issue-129", layer["established_project_evidence"])
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())


if __name__ == "__main__":
    unittest.main()
