"""Check #136 independently reviewed content at the resource/admission boundary."""
import csv
import json
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-136"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
MEMBERSHIP = [
    ("nôtre", "PRO:pos"), ("sénateur", "NOM"), ("paniquer", "VER"),
    ("poison", "NOM"), ("déclaration", "NOM"), ("garantir", "VER"),
    ("veine", "NOM"), ("tempête", "NOM"), ("accusation", "NOM"),
    ("meuble", "NOM"), ("bandit", "NOM"), ("dévorer", "VER"),
    ("trente", "ADJ:num"), ("drapeau", "NOM"), ("détacher", "VER"),
    ("bourse", "NOM"), ("récompense", "NOM"), ("enquêter", "VER"),
    ("mesurer", "VER"), ("solitude", "NOM"), ("empoisonner", "VER"),
    ("mystérieux", "ADJ"), ("opéra", "NOM"), ("commerce", "NOM"),
    ("puits", "NOM"), ("sincère", "ADJ"), ("statue", "NOM"),
    ("cap", "NOM"), ("sécher", "VER"), ("empire", "NOM"),
    ("employer", "VER"), ("pro", "NOM"), ("hôte", "NOM"),
    ("régime", "NOM"), ("jumeau", "NOM"), ("volontiers", "ADV"),
    ("mental", "ADJ"), ("mouiller", "VER"), ("top", "NOM"),
    ("misérable", "ADJ"), ("populaire", "ADJ"), ("man", "NOM"),
    ("alarme", "NOM"), ("frigo", "NOM"), ("conséquence", "NOM"),
    ("demi-tour", "NOM"), ("fauteuil", "NOM"), ("bagarre", "NOM"),
    ("cinglé", "NOM"), ("correct", "ADJ"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def retained_path(name):
    parts = name.replace("\\", "/").split("/")
    local = ROOT.joinpath(*parts)
    if local.is_file():
        return local
    return Path(read("admission.json")["author_evidence_root"]).joinpath(*parts[2:])


class Batch136Tests(unittest.TestCase):
    def test_cingle_person_noun_retains_familiar_register_in_all_meanings(self):
        fields = read("groups.json")[48]["senses"][0]["fields"]
        self.assertEqual(fields["french_definition"]["text"],
                         "Personne que l’on qualifie de folle ; nom masculin familier, féminin cinglée.")
        self.assertEqual(fields["english"]["text"],
                         "A person described as crazy; informal masculine person noun, with feminine form cinglée.")
        self.assertEqual(fields["chinese"]["text"],
                         "疯子；被称为疯狂的人。此处为口语阳性人称名词，阴性形式为 cinglée。")

    def test_reference_only_original_dictionaries_keep_separate_credit(self):
        sources = {s["title"]: s for s in read("sources.json")["sources"]}
        for title in ("meuble", "garantir", "tempête", "déclaration", "drapeau",
                      "bourse", "opéra", "fauteuil", "bagarre"):
            with self.subTest(title=title):
                layers = sources[title]["original_historical_dictionary_layers"]
                self.assertTrue(any(layer["original_authors"] == "Académie française"
                                    and "2006-01-01" in layer["conservative_expiry"]
                                    for layer in layers))
        for title in ("statue", "enquêter", "alarme"):
            self.assertTrue(any(layer["original_authors"] == "Émile Littré"
                                and "1952-01-01" in layer["conservative_expiry"]
                                for layer in sources[title]["original_historical_dictionary_layers"]))
        by_id = {s["id"]: s for s in sources.values()}
        for filename, key, id_name in (("source-context.json", "current_content", "id"),
                                       ("rights.json", "sources", "source_id"),
                                       ("rights.json", "fields", "source_id"),
                                       ("rights.json", "construction_sources", "source_id")):
            for record in read(filename)[key]:
                self.assertEqual(record["original_historical_dictionary_layers"],
                                 by_id[record[id_name]]["original_historical_dictionary_layers"])
        for source in sources.values():
            for layer in source["original_historical_dictionary_layers"]:
                if layer["original_authors"] == "Émile Littré":
                    self.assertEqual(layer["legal_basis_url"],
                                     "https://www.legifrance.gouv.fr/codes/article_lc/LEGIARTI000006278937")

    def test_frozen_groups_keep_49_reviewed_uses_and_exact_man_annotation(self):
        self.assertTrue((EVIDENCE / "groups.json").is_file(),
                        "The exact frozen #136 groups lack an evidence package")
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2751, 2801)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        field_count = 0
        for group in groups:
            row = frozen[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"]),
                             (row["word"], row["lexique_cgram"]))
            for name in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[name], row[name])
            if group["frozen_row"] == 2792:
                self.assertEqual(group["senses"], [])
                self.assertTrue(group["annotation_exception"])
                self.assertEqual((group["admitted_fields"], group["missing_fields"]), (0, 4))
                self.assertEqual(set(group["pending_fields"]), set(LANGUAGES))
                for field in group["pending_fields"].values():
                    self.assertIsNone(field["accepted_text"])
                    self.assertEqual(field["accepted_fields"], 0)
                decision = group["annotation_exception_decision"]
                self.assertEqual((decision["frozen_row"], decision["approved_by"], decision["approved_on"]),
                                 (2792, "project_owner", "2026-11-21"))
                self.assertEqual(decision["exact_reply"], "批准仅 man/NOM 为零字段注记")
                self.assertEqual(decision["timezone"], "America/Vancouver")
                self.assertEqual(decision["issue"], "https://github.com/zardonc/lexique/issues/136")
                continue
            self.assertEqual(len(group["senses"]), 1)
            sense = group["senses"][0]
            self.assertTrue(sense["use"].strip())
            self.assertEqual(sense["independent_review"], "passed")
            self.assertEqual(sense["resource_admission"], "passed_independent_selected_content_and_rights_only")
            self.assertEqual(set(sense["fields"]), set(LANGUAGES))
            for name, field in sense["fields"].items():
                self.assertTrue(field["text"].strip())
                self.assertEqual(field["language"], LANGUAGES[name])
                self.assertTrue(field["source_id"])
                self.assertTrue(field["source_span"])
                field_count += 1
        self.assertEqual(field_count, 196)
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["proposed_fields"], 200)
        self.assertEqual(admission["independently_accepted_new_fields"], 196)
        self.assertEqual(admission["independently_accepted_new_groups"], 49)
        self.assertTrue(admission["independent_reviewer"])
        self.assertEqual(read("groups-independent-review.json"), groups)

    def test_every_accepted_field_keeps_exact_source_quality_rights_and_use(self):
        rights = read("rights.json")
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        captures = {record["source_id"]: record for record in read("source-captures.json")["sources"]}
        self.assertEqual(len(sources), 55)
        raw_lines = {}
        for source_id, capture in captures.items():
            raw = retained_path(capture["raw_path"]).read_text(encoding="utf-8")
            raw_lines[source_id] = raw.splitlines()
            if capture["api_path"]:
                api = json.loads(retained_path(capture["api_path"]).read_text(encoding="utf-8-sig"))
                page = next(page for page in api["query"]["pages"] if page["title"] == capture["title"])
                revision = page["revisions"][0]
                self.assertEqual(revision["slots"]["main"]["content"].encode("utf-8"),
                                 retained_path(capture["raw_path"]).read_bytes())
                self.assertEqual(revision["revid"], sources[source_id]["revid"])
                self.assertEqual(revision["timestamp"], sources[source_id]["timestamp"])
            else:
                self.assertEqual(sources[source_id]["content_license"], "not_established_research_only")
        fields = read("field-review.json")["fields"]
        self.assertEqual(len(fields), 196)
        self.assertEqual(len(rights["fields"]), 196)
        groups = read("groups.json")
        by_identity = {(sense["sense_id"], name): field
                       for group in groups for sense in group["senses"] for name, field in sense["fields"].items()}
        proposals = {(sense["sense_id"], name): field
                     for group in read("authored/groups.json") for sense in group["senses"]
                     for name, field in sense["fields"].items()}
        rights_by_identity = {(f["sense_id"], f["field"]): f for f in rights["fields"]}
        replacements = {(f["sense_id"], f["field"]): f for f in read("accepted-replacements.json")}
        self.assertEqual(len(replacements), 3)
        for field in fields:
            key = (field["sense_id"], field["field"])
            value = by_identity[key]
            self.assertEqual(field["proposed_text"], proposals[key]["text"])
            self.assertEqual(field["accepted_text"], value["text"])
            self.assertEqual(field["accepted_text_quality_status"], "passed")
            self.assertEqual(field["rights_eligibility_status"], "passed_at_selected_text_boundary")
            self.assertEqual(field["reviewer"], value["reviewer"])
            self.assertTrue(field["independence"])
            self.assertTrue(field["review_reason"])
            self.assertTrue(field["selected_text_rights_reason"])
            self.assertEqual(field["exact_source"]["source_id"], value["source_id"])
            self.assertEqual(field["exact_source"]["line_1_based"], value["source_span"]["line_1_based"])
            self.assertEqual(field["exact_source"]["reviewed_excerpt"], value["source_span"]["raw_source_excerpt"])
            right = rights_by_identity[key]
            self.assertEqual(right["accepted_text"], value["text"])
            self.assertEqual(right["source_id"], value["source_id"])
            self.assertEqual(right["modification"], value["modification"])
            self.assertEqual(right["reviewer"], field["reviewer"])
            self.assertEqual(right["selected_text_rights_reason"], field["selected_text_rights_reason"])
            self.assertEqual(sources[value["source_id"]]["content_license"], "CC-BY-SA-4.0")
            self.assertEqual(sources[value["source_id"]]["selected_text_review"], "passed_at_selected_text_boundary")
            if value["text"] != proposals[key]["text"]:
                self.assertEqual(replacements[key]["proposed_text"], proposals[key]["text"])
                self.assertEqual(replacements[key]["accepted_text"], value["text"])
            for span in [field["exact_source"], *field["supporting_sources"]]:
                line = raw_lines[span["source_id"]][span["line_1_based"] - 1]
                column = span["column_1_based"] - 1
                self.assertEqual(line[column:column + len(span["reviewed_excerpt"])], span["reviewed_excerpt"])
                self.assertEqual(span["revid"], sources[span["source_id"]]["revid"])
            lines = raw_lines[value["source_id"]][:value["source_span"]["line_1_based"]]
            language_frame = next(line for line in reversed(lines) if line.startswith("== "))
            pos_frame = next(line for line in reversed(lines) if line.startswith("=== "))
            self.assertIn("{{langue|fr}}", language_frame)
            pos_name = {"NOM": "nom", "ADJ": "adjectif", "ADJ:num": "adjectif numéral",
                        "VER": "verbe", "ADV": "adverbe", "PRO:pos": "pronom possessif"}[field["pos"]]
            if field["frozen_row"] == 2751 and field["field"] == "ipa":
                pos_name = "adjectif possessif"
                self.assertEqual(value["text"], "/notʁ/")
            self.assertIn("{{S|" + pos_name + "|fr", pos_frame)
        for field in rights["fields"]:
            self.assertEqual(field["independent_review"], "passed_at_selected_text_boundary")
            self.assertTrue(field["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 196)
        usages = {u["frozen_row"]: u for u in read("field-review.json")["usage"]}
        for group in groups:
            usage = usages[group["frozen_row"]]
            if group["frozen_row"] == 2792:
                self.assertIsNone(usage["accepted_usage"])
                continue
            self.assertEqual(usage["accepted_usage"], group["senses"][0]["use"])
            self.assertEqual(usage["accepted_usage_quality_status"], "passed")
            self.assertEqual(usage["everyday_sense_status"], "passed")
            self.assertTrue(usage["everyday_sense_reason"])
        admission = read("admission.json")
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["canonical_pending_fields"], 196)
        self.assertEqual(admission["missing_fields"], 4)
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual(admission["author_unresolved_content_rows"], [2792])
        self.assertEqual(admission["native_checks"], "integration_owned_not_run")
        self.assertEqual(admission["conversion_and_packaging"], "integration_owned_not_run")
        self.assertEqual(admission["annotation_exception_groups"], 1)
        self.assertEqual(admission["specialist_exception_groups"], 0)
        self.assertEqual(read("reuse.json")["reused_fields"], 0)

    def test_stable_fallback_retains_source_material_and_original_proposals(self):
        archive = Path(read("admission.json")["author_evidence_root"])
        self.assertTrue(archive.is_dir(), "Stable author-evidence fallback has not been retained")
        for capture in read("source-captures.json")["sources"]:
            for name in ("raw_path", "api_path"):
                if capture[name]:
                    parts = capture[name].replace("\\", "/").split("/")
                    path = archive.joinpath(*parts[2:])
                    self.assertTrue(path.is_file(), str(path))
                    self.assertEqual(path.read_bytes(), retained_path(capture[name]).read_bytes())
        for authority in read("rights.json")["authority_captures"]:
            self.assertTrue(Path(authority["path"]).is_file(), authority["path"])
            for page in authority.get("pages", []):
                self.assertTrue(Path(page["raw_path"]).is_file(), page["raw_path"])
        for name in ("groups.json", "field-review.json", "rights.json", "admission.json",
                     "sources.json", "source-context.json", "source-excerpts.json", "source-captures.json", "reuse.json"):
            original = archive / "fallback-checkout/docs/evidence/issue-136/authored" / name
            self.assertEqual((EVIDENCE / "authored" / name).read_bytes(), original.read_bytes())
        man = read("authored/groups.json")[41]
        self.assertEqual(len(man["senses"][0]["fields"]), 4)
        self.assertEqual(man["senses"][0]["independent_review"], "pending")
        self.assertEqual(len(read("field-review.json")["unresolved_fields"]), 4)
        self.assertEqual(len(read("rights.json")["unresolved_author_proposals"]), 4)
        for name in ("selection.py", "prepare_candidates.py", "finish_author_package.py", "reproduce.py"):
            self.assertTrue((archive / name).is_file(), name)

    def test_complete_contributor_continuations_and_selected_excerpts(self):
        captures = read("source-captures.json")
        names, anonymous, continuation = {}, {}, {}
        for part in captures["contributor_registry"]["parts"]:
            path = retained_path(part)
            query = parse_qs(urlparse(path.with_suffix(".request.txt").read_text(encoding="utf-8")).query)
            if "pccontinue" not in query:
                self.assertFalse(continuation)
            for key, value in continuation.items():
                self.assertEqual(query[key], [str(value)])
            response = json.loads(path.read_text(encoding="utf-8-sig"))
            self.assertNotIn("error", response)
            for page in response["query"]["pages"]:
                names.setdefault(page["title"], set()).update(n["name"] for n in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
            continuation = response.get("continue", {})
        self.assertFalse(continuation)
        for context in read("source-context.json")["current_content"]:
            if context["content_license"] == "CC-BY-SA-4.0":
                self.assertEqual(sorted(names[context["title"]]), context["current_contributors"])
                self.assertEqual(anonymous.get(context["title"], 0), context["anonymous_contributors_count"])
        fields = {(r["sense_id"], r["field"]): r for r in read("field-review.json")["fields"]}
        for excerpt in read("source-excerpts.json")["fields"]:
            record = fields[(excerpt["sense_id"], excerpt["field"])]
            self.assertEqual(excerpt["accepted_text"], record["accepted_text"])
            self.assertEqual(excerpt["source_id"], record["exact_source"]["source_id"])
            self.assertEqual(excerpt["reviewed_excerpt"], record["exact_source"]["reviewed_excerpt"])


if __name__ == "__main__":
    unittest.main()
