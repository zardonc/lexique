"""Check #131's independently reviewed resource/admission evidence."""
import csv
import json
from pathlib import Path
import sys
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-131"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
MEMBERSHIP = [
    ("pratique", "NOM"), ("généreux", "ADJ"), ("humanité", "NOM"),
    ("enfoncer", "VER"), ("excellence", "NOM"), ("arabe", "NOM"),
    ("critique", "NOM"), ("analyse", "NOM"), ("cirque", "NOM"),
    ("divorce", "NOM"), ("objectif", "NOM"), ("serveur", "NOM"),
    ("lutte", "NOM"), ("rejeter", "VER"), ("agence", "NOM"),
    ("cancer", "NOM"), ("couronne", "NOM"), ("bye", "ONO"),
    ("poudre", "NOM"), ("sacrifice", "NOM"), ("pizza", "NOM"),
    ("sous-titre", "NOM"), ("cloche", "NOM"), ("crème", "NOM"),
    ("danseur", "NOM"), ("attitude", "NOM"), ("quart", "NOM"),
    ("déshabiller", "VER"), ("achever", "VER"), ("amen", "ONO"),
    ("charme", "NOM"), ("pompe", "NOM"), ("séance", "NOM"),
    ("neveu", "NOM"), ("robot", "NOM"), ("file", "NOM"),
    ("absence", "NOM"), ("pourrir", "VER"), ("bah", "ONO"),
    ("nôtres", "PRO:pos"), ("régner", "VER"), ("parvenir", "VER"),
    ("propriété", "NOM"), ("surface", "NOM"), ("huile", "NOM"),
    ("pourri", "ADJ"), ("instrument", "NOM"), ("chérir", "VER"),
    ("exercice", "NOM"), ("culture", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def retained_path(name):
    relative = Path(name).relative_to("target/issue-131")
    independent = ROOT / "target/issue-131-review/author-captures" / relative
    if independent.is_file():
        return independent
    archived = Path(read("source-captures.json")["review_capture_root"]) / relative
    if archived.is_file():
        return archived
    local = ROOT / name
    if local.is_file():
        return local
    return Path(read("admission.json")["author_evidence_root"]).joinpath(*Path(name).parts[2:])


class ReviewedPackage131Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_complete_source_bound_acceptances(self):
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2501, 2551)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        excerpts = {(f["sense_id"], f["field"]): f for f in read("source-excerpts.json")["fields"]}
        self.assertEqual(len(excerpts), 200)
        for group in groups:
            row = frozen[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"]), (row["word"], row["lexique_cgram"]))
            for key in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[key], row[key])
            self.assertEqual(len(group["senses"]), 1)
            sense = group["senses"][0]
            self.assertTrue(sense["use"].strip())
            self.assertIsNone(sense["cefr"])
            self.assertEqual(set(sense["fields"]), set(LANGUAGES))
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["modification"].strip())
                    binding = excerpts[(sense["sense_id"], name)]
                    self.assertEqual(field["source_id"], binding["source_id"])
                    self.assertEqual(field["source_span"]["raw_source_excerpt"], binding["reviewed_excerpt"])
                    self.assertEqual(binding["revid"], sources[field["source_id"]]["revid"])
                    self.assertEqual(field["independent_review"], "passed")
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
        self.assertEqual(groups[29]["senses"][0]["source_headword"], "en-amen")
        self.assertEqual(groups[39]["senses"][0]["source_headword"], "les nôtres")
        self.assertEqual(groups[39]["senses"][0]["fields"]["ipa"]["text"], "/notʁ/")
        self.assertIn("les nôtres", groups[39]["senses"][0]["use"])

    def test_fixed_sources_and_contributors_are_reproducible_at_the_retained_boundary(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        raw = {}
        self.assertEqual(len(captures["sources"]), 53)
        for capture in captures["sources"]:
            api = json.loads(retained_path(capture["api_path"]).read_text(encoding="utf-8-sig"))
            page = next(p for p in api["query"]["pages"] if p["pageid"] == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            source = sources[capture["source_id"]]
            self.assertEqual(source["upstream_title"], page["title"])
            self.assertEqual(source["timestamp"], revision["timestamp"])
            body = retained_path(capture["raw_path"]).read_bytes().decode("utf-8")
            self.assertEqual(body, revision["slots"]["main"]["content"])
            self.assertIn("oldid=" + str(revision["revid"]), source["fixed_url"])
            self.assertTrue(source["history_url"])
            self.assertTrue(source["attribution"])
            raw[capture["source_id"]] = body.splitlines()
        bindings = read("source-excerpts.json")
        for binding in bindings["fields"] + bindings["supporting_evidence"]:
            line = raw[binding["source_id"]][binding["line_1_based"] - 1]
            start = binding["column_1_based"] - 1
            self.assertEqual(binding["reviewed_excerpt"], line[start:start + len(binding["reviewed_excerpt"])])
            self.assertFalse(binding["reviewed_excerpt"].startswith("#*"))
            if "ipa_variant_excerpt" in binding:
                start = binding["ipa_variant_column_1_based"] - 1
                self.assertEqual(binding["ipa_variant_excerpt"], line[start:start + len(binding["ipa_variant_excerpt"])])
        positions = {"NOM": "nom", "ADJ": "adjectif", "VER": "verbe", "ONO": "interjection", "PRO:pos": "pronom possessif"}
        for group in read("groups.json"):
            field = group["senses"][0]["fields"]["french_definition"]
            before = raw[field["source_id"]][:field["source_span"]["line_1_based"]]
            if group["frozen_row"] == 2530:
                language = next(line for line in reversed(before) if line.startswith("==") and not line.startswith("==="))
                self.assertEqual(language, "==French==")
                self.assertIn("===Interjection===", before)
            else:
                language = next(line for line in reversed(before) if line.startswith("== "))
                pos = next(line for line in reversed(before) if line.startswith("=== "))
                self.assertEqual(language, "== {{langue|fr}} ==")
                self.assertIn("{{S|" + positions[group["lexique_cgram"]] + "|fr", pos)
        chains = {}
        for name in captures["contributor_registry"]["parts"]:
            path = retained_path(name)
            chains.setdefault(path.name.rsplit("-", 1)[0], []).append(path)
        self.assertEqual(len(chains), 53)
        contributors, anonymous = {}, {}
        for paths in chains.values():
            continuation = {}
            for path in sorted(paths):
                query = parse_qs(urlparse(path.with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                for key, value in continuation.items():
                    self.assertEqual(query[key], [str(value)])
                response = json.loads(path.read_text(encoding="utf-8"))
                self.assertNotIn("error", response)
                for page in response["query"]["pages"]:
                    key = ("en-" if "-en-" in path.name else "") + page["title"]
                    contributors.setdefault(key, set()).update(p["name"] for p in page.get("contributors", []))
                    if "anoncontributors" in page:
                        anonymous[key] = page["anoncontributors"]
                continuation = response.get("continue", {})
            self.assertFalse(continuation)
        self.assertEqual(set(contributors), {s["title"] for s in sources.values()})
        for source in sources.values():
            context = contexts[source["id"]]
            self.assertEqual(source["selected_text_review"], "passed_for_selected_spans")
            self.assertEqual(context["selected_text_rights_status"], "passed_at_selected_text_boundary")
            self.assertEqual(context["status"], "passed_independent_selected_source_frame_review")
            self.assertEqual(context["current_contributors"], sorted(contributors[source["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(source["title"], 0))

    def test_admission_and_exact_decisions_preserve_proposals_and_coordinator_gates(self):
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["independently_accepted_new_groups"], 50)
        self.assertTrue(admission["independent_reviewer"])
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual(admission["pending_independent_review_rows"], [])
        self.assertEqual(admission["missing_authored_fields"], 0)
        self.assertIn((admission["newly_admitted_fields"], admission["canonical_pending_fields"]),
                      ((0, 200), (200, 0)))
        self.assertEqual(admission["missing_admitted_fields"], admission["canonical_pending_fields"])
        self.assertEqual((admission["annotation_exception_groups"], admission["specialist_exception_groups"]), (0, 0))
        self.assertEqual(read("reuse.json")["groups"], [])
        review = read("field-review.json")
        rights = read("rights.json")
        authored = read("authored/field-review.json")
        authored_groups = read("authored/groups.json")
        self.assertTrue(review["independent_acceptance"])
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertEqual(review["reviewer"], admission["independent_reviewer"])
        self.assertEqual(read("authored/snapshot.json")["author_commit"], "f81207941dc340430f3a99a6a3d0825a4215ff16")
        self.assertIsNone(authored["reviewer"])
        self.assertEqual(len(review["fields"]), 200)
        self.assertEqual(len(review["usage"]), 50)
        self.assertEqual(len(review["spelling_relations"]), 50)
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        self.assertEqual(len(rights["fields"]), 200)
        self.assertEqual(rights["reviewer"], review["reviewer"])
        self.assertEqual(rights["independent_review"], "passed_at_selected_text_boundary")
        reviewed = {(r["sense_id"], r["field"]): r for r in review["fields"]}
        proposals = {(r["sense_id"], r["field"]): r for r in authored["fields"]}
        legal = {(r["sense_id"], r["field"]): r for r in rights["fields"]}
        bindings = {(r["sense_id"], r["field"]): r for r in read("source-excerpts.json")["fields"]}
        usage = {r["sense_id"]: r for r in review["usage"]}
        self.assertEqual(set(reviewed), set(proposals))
        self.assertEqual(set(reviewed), set(legal))
        self.assertEqual(set(reviewed), set(bindings))
        for group, author_group in zip(read("groups.json"), authored_groups, strict=True):
            sense = group["senses"][0]
            self.assertEqual(sense["sense_id"], author_group["senses"][0]["sense_id"])
            self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
            use = usage[sense["sense_id"]]
            self.assertEqual(use["accepted_usage"], sense["use"])
            self.assertEqual(use["accepted_usage_quality_status"], "passed")
            self.assertEqual(use["everyday_sense_status"], "passed")
            self.assertEqual(use["reviewer"], review["reviewer"])
            self.assertTrue(use["review_reason"].strip())
            self.assertTrue(use["everyday_sense_reason"].strip())
            for name, field in sense["fields"].items():
                key = sense["sense_id"], name
                record, proposal, right, binding = reviewed[key], proposals[key], legal[key], bindings[key]
                self.assertEqual(record["proposed_text"], proposal["proposed_text"])
                self.assertEqual(record["exact_source"], proposal["exact_source"])
                self.assertEqual(record["supporting_sources"], proposal["supporting_sources"])
                self.assertEqual(record["accepted_text"], field["text"])
                self.assertEqual(record["proposed_text"], record["accepted_text"])
                self.assertEqual(record["accepted_text_quality_status"], "passed")
                self.assertEqual(record["rights_eligibility_status"], "passed_at_selected_text_boundary")
                self.assertEqual(record["reviewer"], review["reviewer"])
                self.assertTrue(record["review_reason"].strip())
                self.assertTrue(record["selected_text_rights_reason"].strip())
                self.assertEqual(right["accepted_text"], field["text"])
                self.assertEqual(binding["accepted_text"], field["text"])
        for field in rights["fields"]:
            proposal = reviewed[(field["sense_id"], field["field"])]
            self.assertEqual(field["source_id"], proposal["exact_source"]["source_id"])
            self.assertEqual(field["source_revision"], proposal["exact_source"]["revid"])
            self.assertEqual(field["exact_source_span"]["raw_source_excerpt"], proposal["exact_source"]["reviewed_excerpt"])
            self.assertTrue(field["author_rights_basis"])
            self.assertTrue(field["selected_boundary"])
            self.assertTrue(field["attribution"])
            self.assertEqual(field["reviewer"], review["reviewer"])
            self.assertEqual(field["independent_review"], "passed_at_selected_text_boundary")
            self.assertTrue(field["rights_reason"].strip())
        for relation in review["spelling_relations"]:
            self.assertEqual(relation["independent_review"], "passed")
            self.assertEqual(relation["relation_role"], "source_binding_only")
            self.assertEqual(relation["alias_admission"], "not_admitted")
            self.assertEqual(relation["reviewer"], review["reviewer"])
            self.assertTrue(relation["review_reason"].strip())
        for authority in rights["authority_captures"]:
            self.assertTrue(Path(authority["path"]).is_file())
            for page in authority.get("pages", []):
                response = json.loads(Path(authority["path"]).read_text(encoding="utf-8-sig"))
                captured = next(p for p in response["query"]["pages"] if p["pageid"] == page["pageid"])
                revision = next(r for r in captured["revisions"] if r["revid"] == page["revid"])
                self.assertEqual(Path(page["raw_path"]).read_text(encoding="utf-8"), revision["slots"]["main"]["content"])
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_text(encoding="utf-8").rstrip())


class IndependentReview131Tests(unittest.TestCase):
    def test_selected_larousse_original_work_has_independent_catalogue_basis(self):
        rights = read("rights.json")
        cherir = [f for f in rights["fields"] if f["frozen_row"] == 2548]
        self.assertEqual(len(cherir), 4)
        for field in cherir:
            layer = next(x for x in field["original_historical_dictionary_layers"]
                         if x["original_authors"] == "Pierre Larousse")
            self.assertEqual(layer["conservative_expiry"], "1961-01-01, using BnF's final publication year 1890 including supplements")
            self.assertEqual(layer["independent_review"], "passed_at_original_work_boundary")
            self.assertIn("cb33995829b", layer["independent_original_work_source"])
        replacements = read("accepted-replacements.json")
        self.assertEqual({(r["frozen_row"], r["field"]) for r in replacements}, {(2548, f) for f in LANGUAGES})
        authored = {(r["sense_id"], r["field"]): r for r in read("authored/rights.json")["fields"]}
        for replacement in replacements:
            self.assertEqual(replacement["proposed_text"], replacement["accepted_text"])
            self.assertEqual(replacement["proposed_rights"], authored[(replacement["sense_id"], replacement["field"])]["original_historical_dictionary_layers"])
        work = read("original-work-review.json")
        self.assertEqual((work["statutory_basis"]["start"], work["statutory_basis"]["expiry"]), ("1891-01-01", "1961-01-01"))
        catalogue = Path(work["original_work_catalogue"]["capture"])
        local = ROOT / "target/issue-131-review/independent-authorities/bnf-larousse.html"
        self.assertIn("1866-1890", (local if local.is_file() else catalogue).read_text(encoding="utf-8"))
        for record in read("source-context.json")["current_content"]:
            for layer in record.get("original_historical_dictionary_layers", []):
                if layer["original_authors"] == "Académie française":
                    self.assertTrue(layer["conservative_expiry"].startswith("2006-01-01"))
                    self.assertEqual(layer["independent_review"], "passed_at_original_work_boundary")

    def test_all_fields_have_separate_quality_and_rights_decisions(self):
        review = read("field-review.json")
        self.assertTrue(review["independent_acceptance"])
        self.assertEqual(len(review["fields"]), 200)
        for field in review["fields"]:
            self.assertEqual(field["accepted_text_quality_status"], "passed")
            self.assertEqual(field["rights_eligibility_status"], "passed_at_selected_text_boundary")


class AdmittedCore131Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_admitted_four_field_content(self):
        curated = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
        actual = {g["frozen_row"]: g for g in curated["groups"]}
        accepted = {g["frozen_row"]: g for g in read("groups.json")}
        sources = {s["id"]: s for s in curated["sources"]}
        reviews = {(r["sense_id"], r["field"]): r for r in read("field-review.json")["fields"]}
        missing = [row for row in range(2501, 2551) if row not in actual]
        self.assertEqual(missing, [], f"#131 frozen groups lack admitted content: {missing}")
        for row, pair in enumerate(MEMBERSHIP, 2501):
            group = actual[row]
            self.assertEqual((group["headword"], group["lexique_cgram"]), pair)
            self.assertTrue(group["senses"])
            self.assertEqual(len(group["senses"]), len(accepted[row]["senses"]))
            for sense, expected in zip(group["senses"], accepted[row]["senses"], strict=True):
                self.assertTrue(sense["use"].strip())
                self.assertEqual((sense["sense_id"], sense["source_headword"], sense["use"]),
                                 (expected["sense_id"], expected["source_headword"], expected["use"]))
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for name, field in sense["fields"].items():
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(field["independent_review"], "passed")
                    for key in ("text", "language", "source_id", "source_span", "content_license", "modification"):
                        self.assertEqual(field[key], expected["fields"][name][key])
                    source = sources[field["source_id"]]
                    selected = reviews[sense["sense_id"], name]
                    self.assertEqual((source["revid"], source["fixed_url"]),
                                     (selected["exact_source"]["revid"], selected["exact_source"]["url"]))
                    self.assertEqual(source["selected_text_review"], "passed_for_selected_spans")
        self.assertEqual(curated["scope_denominator_groups"], 4087)


if __name__ == "__main__":
    admitted = "--admitted" in sys.argv
    if admitted:
        sys.argv.remove("--admitted")
    unittest.main(defaultTest="AdmittedCore131Tests" if admitted else ("ReviewedPackage131Tests", "IndependentReview131Tests"))
