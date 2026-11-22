"""Check #133 content proposals at the approved resource/admission seam."""
import csv
import json
from pathlib import Path
import re
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-133"
MEMBERSHIP = [
    ("circonstance", "NOM"), ("salade", "NOM"), ("salopard", "NOM"),
    ("cercle", "NOM"), ("piéger", "VER"), ("chauffer", "VER"),
    ("gouverneur", "NOM"), ("garce", "NOM"), ("cuire", "VER"),
    ("anneau", "NOM"), ("cependant", "ADV"), ("paire", "NOM"),
    ("pré", "NOM"), ("fleuve", "NOM"), ("domaine", "NOM"),
    ("soulager", "VER"), ("immense", "ADJ"), ("incident", "NOM"),
    ("essentiel", "NOM"), ("mercredi", "NOM"), ("pourvoir", "VER"),
    ("placard", "NOM"), ("fermé", "ADJ"), ("fan", "NOM"),
    ("jugement", "NOM"), ("r", "ADV"), ("magazine", "NOM"),
    ("estimer", "VER"), ("pile", "NOM"), ("renfort", "NOM"),
    ("relâcher", "VER"), ("réputation", "NOM"), ("logique", "ADJ"),
    ("poubelle", "NOM"), ("for", "NOM"), ("degré", "NOM"),
    ("tension", "NOM"), ("règlement", "NOM"), ("assiette", "NOM"),
    ("craquer", "VER"), ("serment", "NOM"), ("chanteur", "NOM"),
    ("clinique", "NOM"), ("célibataire", "ADJ"), ("merveille", "NOM"),
    ("décoller", "VER"), ("ingénieur", "NOM"), ("réserve", "NOM"),
    ("autorisation", "NOM"), ("voyou", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def retained_path(name):
    parts = name.replace("\\", "/").split("/")
    local = ROOT.joinpath(*parts)
    if local.is_file():
        return local
    return Path(read("admission.json")["author_evidence_root"]).joinpath(*parts[2:])


class Batch133Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_accounted_resource_content(self):
        self.assertTrue((EVIDENCE / "groups.json").is_file(),
                        "No #133 resource admission/content records for frozen rows 2601–2650")
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2601, 2651)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group in groups:
            row = frozen[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"]),
                             (row["word"], row["lexique_cgram"]))
            for key in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[key], row[key])
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)

    def test_author_package_keeps_unresolved_and_review_gates_visible(self):
        groups = read("groups.json")
        self.assertEqual(groups, read("author-proposals.json")["groups"])
        candidates = [g for g in groups if g["senses"]]
        self.assertEqual(len(candidates), 49)
        self.assertEqual([g["frozen_row"] for g in groups if not g["senses"]], [2626])
        unresolved = read("unresolved.json")["groups"][0]
        self.assertEqual((unresolved["headword"], unresolved["lexique_cgram"]), ("r", "ADV"))
        self.assertFalse(unresolved["approved_exception"])
        self.assertEqual(set(unresolved["missing_fields"]),
                         {"french_definition", "ipa", "english", "chinese"})
        admission = read("admission.json")
        for key, value in {
            "frozen_groups": 50, "proposed_groups": 49, "proposed_fields": 196,
            "missing_authored_fields": 4, "missing_admitted_fields": 200,
            "pending_independent_review_fields": 196, "pending_independent_rights_fields": 196,
            "independently_accepted_new_groups": 0, "independently_accepted_new_fields": 0,
            "annotation_exception_groups": 0, "specialist_exception_groups": 0,
        }.items():
            self.assertEqual(admission[key], value, key)
        self.assertIsNone(admission["independent_reviewer"])
        self.assertEqual(admission["native_checks"], "integration_owned_not_run")
        self.assertEqual(admission["conversion_and_packaging"], "integration_owned_not_run")
        reviews = read("field-review.json")
        self.assertFalse(reviews["independent_acceptance"])
        self.assertIsNone(reviews["reviewer"])
        self.assertEqual(len(reviews["fields"]), 196)
        self.assertEqual(len(reviews["usage"]), 49)

    def test_fixed_sources_bind_four_fields_to_exact_french_pos_and_use(self):
        captures = read("source-captures.json")
        self.assertEqual(Path(captures["capture_root"]), Path(read("admission.json")["author_evidence_root"]))
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        raw = {}
        self.assertEqual(len(captures["sources"]), 51)
        for capture in captures["sources"]:
            api = json.loads(retained_path(capture["api_path"]).read_text(encoding="utf-8-sig"))
            page = next(p for p in api["query"]["pages"] if p["pageid"] == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            body = retained_path(capture["raw_path"]).read_bytes().decode("utf-8")
            self.assertEqual(body, revision["slots"]["main"]["content"])
            source = sources[capture["source_id"]]
            self.assertEqual((source["title"], source["revid"], source["timestamp"]),
                             (page["title"], revision["revid"], revision["timestamp"]))
            self.assertIn("oldid=" + str(revision["revid"]), source["fixed_url"])
            self.assertTrue(source["attribution"])
            self.assertTrue(source["history_url"])
            self.assertTrue(contexts[source["id"]]["current_contributors"])
            raw[source["id"]] = body.splitlines()
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        bindings = {(b["sense_id"], b["field"]): b for b in read("source-excerpts.json")["fields"]}
        rights = {(b["sense_id"], b["field"]): b for b in read("rights.json")["fields"]}
        self.assertEqual(len(bindings), 196)
        self.assertEqual(set(bindings), set(rights))
        for group in read("groups.json"):
            if not group["senses"]:
                continue
            self.assertEqual(len(group["senses"]), 1)
            sense = group["senses"][0]
            self.assertTrue(sense["use"].strip())
            self.assertTrue(sense["excluded_uses"].strip())
            self.assertIsNone(sense["cefr"])
            self.assertEqual(set(sense["fields"]), set(languages))
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(field["language"], languages[name])
                    self.assertEqual(field["independent_review"], "pending")
                    self.assertTrue(field["modification"].strip())
                    b = bindings[(sense["sense_id"], name)]
                    span = field["source_span"]
                    self.assertEqual((field["source_id"], span["line_1_based"], span["column_1_based"]),
                                     (b["source_id"], b["line_1_based"], b["column_1_based"]))
                    line = raw[field["source_id"]][span["line_1_based"] - 1]
                    self.assertEqual(span["raw_source_excerpt"], line)
                    before = raw[field["source_id"]][:span["line_1_based"]]
                    language = next(l for l in reversed(before) if re.match(r"^==\s*[^=]", l))
                    self.assertRegex(language, r"^==\s*\{\{langue\|fr\}\}\s*==$")
                    pos = next(l for l in reversed(before) if l.startswith("=== {{S|"))
                    expected = {"NOM": "nom", "ADJ": "adjectif", "VER": "verbe", "ADV": "adverbe"}
                    self.assertIn("{{S|" + expected[group["lexique_cgram"]] + "|fr", pos)
                    if name == "ipa":
                        self.assertEqual(field["text"], "/" + b["ipa_variant_excerpt"] + "/")
                        self.assertIn(b["ipa_variant_excerpt"], line)
                    else:
                        self.assertTrue(line.startswith("#") and not line.startswith("#*"))
                    proposal = rights[(sense["sense_id"], name)]
                    self.assertEqual(proposal["proposed_text"], field["text"])
                    self.assertEqual(proposal["content_license"], "CC-BY-SA-4.0")
                    self.assertEqual(proposal["independent_review"], "pending")
                    self.assertTrue(proposal["attribution"])
        supports = read("source-excerpts.json")["supporting_evidence"]
        supports += read("unresolved.json")["groups"][0]["inspected_evidence"]
        for b in supports:
            line = raw[b["source_id"]][b["line_1_based"] - 1]
            start = b["column_1_based"] - 1
            self.assertEqual(b["reviewed_excerpt"], line[start:start + len(b["reviewed_excerpt"])])
        relations = read("source-context.json")["spelling_relations"]
        self.assertEqual(len(relations), 49)
        self.assertTrue(all(r["alias_admission"] == "not_admitted" for r in relations))

    def test_paginated_contributor_capture_and_actual_license_material(self):
        captures = read("source-captures.json")
        parts = [retained_path(p) for p in captures["contributor_registry"]["parts"]]
        names, anonymous = {}, {}
        for support in (False, True):
            continuation = {}
            chain = [p for p in parts if ("support" in p.name) == support]
            self.assertTrue(chain)
            for path in chain:
                query = parse_qs(urlparse(path.with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                for key, value in continuation.items():
                    self.assertEqual(query[key], [str(value)])
                response = json.loads(path.read_text(encoding="utf-8-sig"))
                self.assertNotIn("error", response)
                for page in response["query"]["pages"]:
                    names.setdefault(page["title"], set()).update(c["name"] for c in page.get("contributors", []))
                    if "anoncontributors" in page:
                        anonymous[page["title"]] = page["anoncontributors"]
                continuation = response.get("continue", {})
            self.assertFalse(continuation)
        contexts = read("source-context.json")["current_content"]
        self.assertEqual(set(names), {s["title"] for s in contexts})
        for source in contexts:
            self.assertEqual(source["current_contributors"], sorted(names[source["title"]]))
            self.assertEqual(source["anonymous_contributors_count"], anonymous.get(source["title"], 0))
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_text(encoding="utf-8").rstrip())
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        credits = (EVIDENCE / "SOURCE-NOTICES.md").read_text(encoding="utf-8")
        self.assertIn("CC BY-SA 4.0", notices)
        self.assertIn("Boris New", notices)
        self.assertIn("Christophe Pallier", notices)
        self.assertIn("Académie française", notices)
        self.assertEqual(read("rights.json")["independent_rights_fields_accepted"], 0)
        for source in read("sources.json")["sources"]:
            self.assertIn(source["fixed_url"], credits)
            self.assertIn(source["history_url"], credits)
            for original in source["original_historical_dictionary_layers"]:
                self.assertEqual(original["conservative_expiry"], "2006-01-01")
                self.assertEqual(original["original_authors"], "Académie française")
        policy = json.loads(retained_path("target/issue-133/rights/fr-policy.api.json").read_text(encoding="utf-8-sig"))
        for page in policy["query"]["pages"]:
            body = retained_path(f"target/issue-133/rights/policy-{page['pageid']}.wiki").read_text(encoding="utf-8")
            self.assertEqual(body, page["revisions"][0]["slots"]["main"]["content"].replace("\r\n", "\n"))


if __name__ == "__main__":
    unittest.main()
