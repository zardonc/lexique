"""Check #138 at the resource/admission boundary; --admitted checks canonical content."""
import csv
import json
from pathlib import Path
import sys
import unittest
from urllib.parse import parse_qs, urlparse

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-138"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
MEMBERSHIP = [
    ("boisson", "NOM"), ("catholique", "ADJ"), ("étrangler", "VER"),
    ("progrès", "NOM"), ("riz", "NOM"), ("signature", "NOM"),
    ("consulter", "VER"), ("dose", "NOM"), ("réveil", "NOM"),
    ("intelligence", "NOM"), ("bisou", "NOM"), ("transférer", "VER"),
    ("u", "NOM"), ("dégoûter", "VER"), ("personnellement", "ADV"),
    ("défaut", "NOM"), ("loger", "VER"), ("messe", "NOM"),
    ("bougie", "NOM"), ("ivre", "ADJ"), ("ouverture", "NOM"),
    ("cage", "NOM"), ("suffisamment", "ADV"), ("consacrer", "VER"),
    ("invitation", "NOM"), ("réellement", "ADV"), ("aïe", "ONO"),
    ("parking", "NOM"), ("araignée", "NOM"), ("opposer", "VER"),
    ("éternité", "NOM"), ("communication", "NOM"), ("fédéral", "ADJ"),
    ("métro", "NOM"), ("culotte", "NOM"), ("bâton", "NOM"),
    ("plastique", "NOM"), ("trait", "NOM"), ("tireur", "NOM"),
    ("appart", "NOM"), ("branche", "NOM"), ("miel", "NOM"),
    ("coïncidence", "NOM"), ("halte", "ONO"), ("absolu", "ADJ"),
    ("éclairer", "VER"), ("toile", "NOM"), ("féminin", "ADJ"),
    ("ruine", "NOM"), ("cravate", "NOM"),
]
ADMITTED = "--admitted" in sys.argv
if ADMITTED:
    sys.argv.remove("--admitted")


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def retained_path(name):
    parts = name.replace("\\", "/").split("/")
    archive = Path(read("admission.json")["author_evidence_root"])
    if archive.is_dir():
        return archive.joinpath(*parts[parts.index("issue-138") + 1:])
    path = Path(name)
    if path.is_file():
        return path
    local = ROOT / path
    if local.is_file():
        return local
    if "issue-138" in parts:
        return archive.joinpath(*parts[parts.index("issue-138") + 1:])
    return archive.joinpath(*parts[2:])


class Batch138Tests(unittest.TestCase):
    def test_exact_ticket_resource_groups(self):
        path = (ROOT / "resources/core-pilot-79/curated-inputs.json"
                if ADMITTED else EVIDENCE / "groups.json")
        if ADMITTED:
            from core_pilot import load_inputs
            data = load_inputs(path.parent)[0]
        else:
            data = json.loads(path.read_text(encoding="utf-8"))
        groups = [g for g in (data["groups"] if ADMITTED else data)
                  if 2851 <= g["frozen_row"] <= 2900]
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2851, 2901)))

    def test_candidates_preserve_frozen_frequencies_four_fields_and_review_gates(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        self.assertEqual(len(scope), 4087)
        for group in read("groups.json"):
            with self.subTest(row=group["frozen_row"]):
                frozen = scope[group["frozen_row"] - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (frozen["word"], frozen["lexique_cgram"]))
                for name in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[name], frozen[name])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertTrue(sense["use"].strip())
                self.assertTrue(sense["excluded_uses"].strip())
                self.assertIsNone(sense["cefr"])
                self.assertEqual(sense["independent_review"], "pending")
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"].strip())
                    self.assertEqual(field["independent_review"], "pending")
        reuse = read("reuse.json")
        self.assertEqual((reuse["reused_groups"], reuse["reused_fields"], reuse["groups"]), (0, 0, []))
        admission = read("admission.json")
        self.assertEqual(admission["author_evidence_root"],
                         "C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-138")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"],
                          admission["independently_accepted_new_fields"]), (0, 0))
        self.assertIsNone(admission["independent_reviewer"])
        self.assertEqual(admission["pending_independent_review_rows"], list(range(2851, 2901)))
        self.assertEqual(admission["pending_independent_review_fields"], 200)
        self.assertEqual(admission["pending_independent_rights_fields"], 200)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["missing_authored_fields"], 0)
        self.assertEqual(admission["native_checks"], "integration_owned_not_run")

    def test_captured_revisions_and_complete_attribution_chains(self):
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        captures = read("source-captures.json")
        self.assertEqual(len(captures["sources"]), 50)
        for capture in captures["sources"]:
            response = json.loads(retained_path(capture["api_path"]).read_text(encoding="utf-8"))
            page = next(p for p in response["query"]["pages"] if p["pageid"] == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            source = sources[capture["source_id"]]
            self.assertEqual((source["title"], source["revid"], source["timestamp"]),
                             (page["title"], revision["revid"], revision["timestamp"]))
            self.assertEqual(retained_path(capture["raw_path"]).read_bytes().decode("utf-8"),
                             revision["slots"]["main"]["content"])
            self.assertIn("oldid=" + str(revision["revid"]), source["fixed_url"])
            self.assertTrue(source["attribution"])
            self.assertTrue(source["history_url"])
        names, anonymous = {}, {}
        paths = [retained_path(p) for p in captures["contributor_registry"]["parts"]]
        for prefix in ("contributors-",):
            chain = sorted((p for p in paths if p.name.removeprefix(prefix).split(".")[0].isdigit()),
                           key=lambda p: int(p.name.removeprefix(prefix).split(".")[0]))
            self.assertTrue(chain)
            continuation = {}
            for path in chain:
                query = parse_qs(urlparse(path.with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                for key, value in continuation.items():
                    self.assertEqual(query[key], [str(value)])
                response = json.loads(path.read_text(encoding="utf-8"))
                self.assertNotIn("error", response)
                for page in response["query"]["pages"]:
                    names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                    if "anoncontributors" in page:
                        anonymous[page["title"]] = page["anoncontributors"]
                continuation = response.get("continue", {})
            self.assertFalse(continuation)
        self.assertEqual(set(names), {s["title"] for s in sources.values()})
        for source in sources.values():
            self.assertEqual(contexts[source["id"]]["current_contributors"], sorted(names[source["title"]]))
            self.assertEqual(contexts[source["id"]]["anonymous_contributors_count"], anonymous.get(source["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])

    def test_fields_bind_exact_french_pos_ipa_rights_and_unaccepted_proposals(self):
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        raw = {c["source_id"]: retained_path(c["raw_path"]).read_text(encoding="utf-8").splitlines()
               for c in read("source-captures.json")["sources"]}
        rights = read("rights.json")
        proposals = {(r["sense_id"], r["field"]): r for r in rights["fields"]}
        reviews = {(r["sense_id"], r["field"]): r for r in read("field-review.json")["fields"]}
        excerpts = {(r["sense_id"], r["field"]): r for r in read("source-excerpts.json")["fields"]}
        self.assertEqual(len(proposals), 200)
        self.assertEqual(set(proposals), set(reviews))
        self.assertEqual(set(proposals), set(excerpts))
        self.assertFalse(read("field-review.json")["independent_acceptance"])
        pos_names = {"VER": "verbe", "NOM": "nom", "ADJ": "adjectif", "ADV": "adverbe",
                     "ONO": "interjection", "PRO:int": "pronom interrogatif", "PRO:rel": "pronom relatif"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    lines = raw[field["source_id"]]
                    span = field["source_span"]
                    self.assertEqual(span["raw_source_excerpt"], lines[span["line_1_based"] - 1])
                    before = lines[:span["line_1_based"]]
                    language = next(line for line in reversed(before) if line.startswith("== "))
                    self.assertEqual(language.replace(" ", ""), "=={{langue|fr}}==")
                    pos = next(line for line in reversed(before) if line.startswith("=== {{S|"))
                    self.assertIn("{{S|" + ("lettre" if group["frozen_row"] == 2863 else pos_names[group["lexique_cgram"]]) + "|fr", pos)
                    if name == "ipa":
                        self.assertIn(field["text"].strip("/"), span["raw_source_excerpt"])
                    else:
                        self.assertTrue(span["raw_source_excerpt"].startswith("# "))
                    key = (sense["sense_id"], name)
                    proposal, review, excerpt = proposals[key], reviews[key], excerpts[key]
                    self.assertEqual(proposal["exact_source_span"], span)
                    self.assertEqual(proposal["source_revision"], sources[field["source_id"]]["revid"])
                    self.assertEqual(review["proposed_text"], field["text"])
                    self.assertEqual(excerpt["reviewed_excerpt"], span["raw_source_excerpt"])
                    self.assertEqual(review["accepted_text_quality_status"], "pending")
                    self.assertIsNone(review["reviewer"])
                    self.assertEqual(proposal["independent_review"], "pending")
                    self.assertTrue(proposal["author_rights_basis"])
                    self.assertTrue(proposal["attribution"])
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
        for support in read("source-excerpts.json")["supporting_evidence"]:
            line = raw[support["source_id"]][support["line_1_based"] - 1]
            column = support["column_1_based"] - 1
            self.assertEqual(line[column:column + len(support["reviewed_excerpt"])], support["reviewed_excerpt"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 0)
        self.assertIn("2006-01-01", rights["original_daf8_basis_scope"])
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())

    def test_letter_name_interjections_and_exact_noun_sections(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        letter = groups[2863]["senses"][0]
        self.assertEqual(groups[2863]["headword"], "u")
        self.assertEqual(groups[2863]["lexique_cgram"], "NOM")
        self.assertEqual(letter["fields"]["ipa"]["text"], "/y/")
        for name in ("french_definition", "english", "chinese"):
            self.assertIn("/y/", letter["fields"][name]["text"])
        relations = read("field-review.json")["spelling_relations"]
        relation = next(r for r in relations if r["frozen_row"] == 2863)
        self.assertEqual(relation["relation"], "exact_french_source_spelling")
        self.assertEqual(relation["alias_admission"], "not_admitted")
        evidence = [r["reviewed_excerpt"] for r in relation["proposed_evidence"]]
        self.assertTrue(any("Un grand" in r for r in evidence))
        self.assertTrue(any("{{m}} {{invar}}" in r for r in evidence))
        for row, ipa in ((2877, "/aj/"), (2894, "/alt/")):
            self.assertEqual(groups[row]["lexique_cgram"], "ONO")
            self.assertEqual(groups[row]["senses"][0]["fields"]["ipa"]["text"], ipa)
        for name, restriction in (("french_definition", "soldats"), ("english", "soldiers"), ("chinese", "士兵")):
            self.assertIn(restriction, groups[2894]["senses"][0]["fields"][name]["text"])
        self.assertIn("h aspiré", groups[2894]["senses"][0]["use"])
        # These actual homonyms must bind the modern selected noun section.
        for row, noun_number in ((2879, "num=2"), (2884, "num=2"), (2887, "num=2"), (2900, "num=2")):
            field = next(r for r in read("field-review.json")["fields"]
                         if r["frozen_row"] == row and r["field"] == "french_definition")
            self.assertTrue(any(noun_number in r["reviewed_excerpt"] for r in field["supporting_sources"]))

    def test_actual_import_boundary_and_retained_policy_material(self):
        contexts = read("source-context.json")["current_content"]
        signature = next(c for c in contexts if c["title"] == "signature")
        marker = next(m for m in signature["external_reference_markers"] if "Import:CFC" in m["raw_reference"])
        self.assertIn("unselected", marker["selected_boundary"])
        self.assertEqual(marker["line_1_based"], 191)
        self.assertTrue(all(location["line_1_based"] in (9, 10) for location in signature["selected_locations"]))
        for authority in read("rights.json")["authority_captures"]:
            path = retained_path(authority["path"])
            self.assertTrue(path.is_file())
            if authority.get("pages"):
                response = json.loads(path.read_text(encoding="utf-8"))
                for record in authority["pages"]:
                    page = next(p for p in response["query"]["pages"] if p["pageid"] == record["pageid"])
                    revision = next(r for r in page["revisions"] if r["revid"] == record["revid"])
                    self.assertEqual(retained_path(record["raw_path"]).read_bytes().decode("utf-8"),
                                     revision["slots"]["main"]["content"])


if __name__ == "__main__":
    unittest.main()
