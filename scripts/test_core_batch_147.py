"""Verify the exact #147 resource/admission handoff boundary."""
import csv
import json
from pathlib import Path
import subprocess
import unittest
from urllib.parse import parse_qs, urlparse

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-147"
MEMBERSHIP = [
    ("marge", "NOM"), ("plier", "VER"), ("mélanger", "VER"), ("métal", "NOM"),
    ("touriste", "NOM"), ("aiguille", "NOM"), ("détourner", "VER"), ("obscurité", "NOM"),
    ("tarte", "NOM"), ("grain", "NOM"), ("poignarder", "VER"), ("trône", "NOM"),
    ("destruction", "NOM"), ("allié", "NOM"), ("tsar", "NOM"), ("profit", "NOM"),
    ("avenue", "NOM"), ("semblable", "ADJ"), ("éclat", "NOM"), ("coiffeur", "NOM"),
    ("cuir", "NOM"), ("détenu", "NOM"), ("manche", "NOM"), ("syndicat", "NOM"),
    ("batterie", "NOM"), ("dégât", "NOM"), ("lame", "NOM"), ("prostitué", "NOM"),
    ("griller", "VER"), ("âne", "NOM"), ("objection", "NOM"), ("outil", "NOM"),
    ("malédiction", "NOM"), ("producteur", "NOM"), ("survivant", "NOM"), ("golf", "NOM"),
    ("auparavant", "ADV"), ("explosif", "NOM"), ("conducteur", "NOM"), ("isoler", "VER"),
    ("lécher", "VER"), ("échelle", "NOM"), ("chèvre", "NOM"), ("légitime", "ADJ"),
    ("arroser", "VER"), ("composer", "VER"), ("distribuer", "VER"), ("homicide", "NOM"),
    ("fourrer", "VER"), ("rarement", "ADV"),
]
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch147Tests(unittest.TestCase):
    def test_avenue_retains_variable_size_and_importance(self):
        sense = next(g for g in read('groups.json') if g['headword'] == 'avenue')['senses'][0]
        expected = {
            'french_definition': 'Voie de circulation en ville, de taille et d’importance variables.',
            'english': 'An avenue: an urban street whose size and importance may vary; feminine noun.',
            'chinese': '大街：城市道路，其规模和重要程度不一；阴性名词。',
        }
        for name, value in expected.items():
            self.assertEqual(sense['fields'][name]['text'], value)
        self.assertEqual(sense['use'], 'Feminine noun: an urban avenue of variable size and importance; no tree-lined requirement.')

    def test_poignarder_does_not_require_a_successful_injury(self):
        sense = next(g for g in read('groups.json') if g['headword'] == 'poignarder')['senses'][0]
        self.assertEqual(sense['fields']['english']['text'],
                         'To strike, wound or kill someone with a dagger; transitive verb.')

    def test_manche_uses_the_selected_covering_facts(self):
        sense = next(g for g in read('groups.json') if g['headword'] == 'manche')['senses'][0]
        self.assertEqual(sense['fields']['french_definition']['text'],
                         'Élément d’un vêtement qui couvre tout ou partie du bras.')

    def test_selected_expression_rights_are_specific(self):
        review = read('rights.json')
        self.assertEqual(review.get('worldwide_daf8_clearance'), 'unproved_not_relied_on')
        decisions = read('original-expression-review.json')
        self.assertEqual(len(decisions['uses']), 50)
        for decision in decisions['uses']:
            self.assertIn(decision['determination'],
                          ('noncopyrightable_short_expression', 'minimal_factual_spans'))
            self.assertTrue(decision['expression_reason'].strip())
            self.assertTrue(decision['meaning_reason'].strip())

    def test_fields_bind_to_complete_archived_sources_and_credits(self):
        captures = read("source-captures.json")
        archive = Path(captures["capture_root"])
        self.assertTrue(archive.is_dir())
        self.assertEqual(Path(captures["capture_root_fallback"]), archive)
        self.assertEqual(Path(read("admission.json")["reviewer_evidence_root"]), archive)
        self.assertTrue(Path(read("admission.json")["author_evidence_root"]).is_dir())
        self.assertTrue(Path(captures["preparation_script"]).is_file())
        self.assertTrue(Path(captures["selection_file"]).is_file())

        def retained(name):
            return archive.joinpath(*Path(name).parts[2:])

        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        self.assertEqual((len(captures["sources"]), len(sources), len(contexts)), (50, 50, 50))
        lines = {}
        for capture in captures["sources"]:
            data = json.loads(retained(capture["api_path"]).read_text(encoding="utf-8-sig"))
            page = next(p for p in data["query"]["pages"] if p["pageid"] == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            body = revision["slots"]["main"]["content"]
            self.assertEqual(body.encode("utf-8"), retained(capture["raw_path"]).read_bytes())
            self.assertEqual((page["title"], revision["timestamp"]), (capture["title"], capture["timestamp"]))
            source = sources[capture["source_id"]]
            self.assertEqual(source["revid"], capture["revid"])
            self.assertEqual(parse_qs(urlparse(source["fixed_url"]).query)["oldid"], [str(capture["revid"])])
            lines[source["id"]] = body.splitlines()
        excerpts = read("source-excerpts.json")
        by_field = {(r["sense_id"], r["field"]): r for r in excerpts["fields"]}
        self.assertEqual(len(by_field), 200)
        for record in [*excerpts["fields"], *excerpts["supporting_evidence"]]:
            line = lines[record["source_id"]][record["line_1_based"] - 1]
            column = record["column_1_based"] - 1
            self.assertEqual(line[column:column + len(record["reviewed_excerpt"])], record["reviewed_excerpt"])
            self.assertFalse(record["reviewed_excerpt"].lstrip().startswith(("#*", "##*")))
            self.assertEqual(record["revid"], sources[record["source_id"]]["revid"])
        frames = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "ADV": "adverbe"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            support = [r for r in excerpts["supporting_evidence"] if r["sense_id"] == sense["sense_id"]]
            self.assertTrue(any(r["reviewed_excerpt"] == "== {{langue|fr}} ==" for r in support))
            self.assertTrue(any("{{S|" + frames[group["lexique_cgram"]] + "|fr" in r["reviewed_excerpt"] for r in support))
            for name, field in sense["fields"].items():
                exact = by_field[sense["sense_id"], name]
                self.assertEqual((field["source_id"], field["source_span"]["raw_source_excerpt"]),
                                 (exact["source_id"], exact["reviewed_excerpt"]))
                self.assertEqual(field["source_span"]["line_1_based"], exact["line_1_based"])
                if name == "ipa":
                    self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
                    start = exact["ipa_variant_column_1_based"] - 1
                    self.assertEqual(exact["reviewed_excerpt"][start:start + len(exact["ipa_variant_excerpt"])], exact["ipa_variant_excerpt"])

        names, anonymous, parts = {}, {}, {}
        for name in captures["contributor_registry"]["parts"]:
            path = retained(name)
            data = json.loads(path.read_text(encoding="utf-8"))
            parts[int(path.name.removeprefix("contributors-").removesuffix(".api.json"))] = (path, data)
            for page in data["query"]["pages"]:
                names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
        self.assertEqual(sorted(parts), list(range(1, len(parts) + 1)))
        for part, (_, data) in parts.items():
            continuation = data.get("continue")
            if part == len(parts):
                self.assertFalse(continuation)
            else:
                query = parse_qs(urlparse(parts[part + 1][0].with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                for key, value in continuation.items():
                    self.assertEqual(query[key], [str(value)])
        self.assertEqual(set(names), {s["title"] for s in sources.values()})
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])
        for context in contexts.values():
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        for authority in read("rights.json")["authority_captures"]:
            path = Path(authority["path"])
            self.assertTrue(path.is_relative_to(archive))
            self.assertEqual(path.read_bytes(), Path(authority["copy_from"]).read_bytes())
            if authority.get("pages"):
                api = json.loads(path.read_text(encoding="utf-8"))
                for page_record in authority["pages"]:
                    page = next(p for p in api["query"]["pages"] if p["pageid"] == page_record["pageid"])
                    revision = next(r for r in page["revisions"] if r["revid"] == page_record["revid"])
                    raw = Path(page_record["raw_path"])
                    self.assertTrue(raw.is_relative_to(archive))
                    self.assertEqual(raw.read_bytes(), Path(page_record["copy_from"]).read_bytes())
                    self.assertEqual(raw.read_text(encoding="utf-8"), revision["slots"]["main"]["content"].replace("\r\n", "\n"))

        reuse = read("reuse.json")
        baseline = json.loads(subprocess.check_output(["git", "show", reuse["base_commit"] + ":" + reuse["comparison_input"]], cwd=ROOT))
        self.assertEqual((baseline["content_version"], len(baseline["groups"])),
                         (reuse["canonical_content_version"], reuse["existing_admitted_groups"]))
        overlap = {(g["headword"], g["lexique_cgram"]) for g in baseline["groups"]} & set(MEMBERSHIP)
        self.assertEqual(overlap, set())
        self.assertEqual((reuse["reused_groups"], reuse["reused_fields"], reuse["groups"]), (0, 0, []))

    def test_independent_review_keeps_coordinator_admission_pending(self):
        admission = read("admission.json")
        review = read("field-review.json")
        rights = read("rights.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (50, 200))
        self.assertEqual((admission["pending_independent_review_fields"], admission["pending_independent_rights_fields"]), (0, 0))
        self.assertEqual((admission["newly_admitted_fields"], admission["missing_admitted_fields"]), (0, 200))
        self.assertEqual((admission["annotation_exception_groups"], admission["specialist_exception_groups"]), (0, 0))
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertEqual((admission["missing_authored_fields"], admission["author_unresolved_content_rows"]), (0, []))
        self.assertTrue(review["independent_acceptance"])
        self.assertTrue(review['independence'])
        self.assertEqual(review['kind'], 'separate_independent_field_review')
        self.assertEqual(review["reviewer"], admission['independent_reviewer'])
        fields = {(r["sense_id"], r["field"]): r for r in review["fields"]}
        legal = {(r["sense_id"], r["field"]): r for r in rights["fields"]}
        self.assertEqual(len(fields), 200)
        self.assertEqual(set(fields), set(legal))
        self.assertEqual((len(review["usage"]), len(review["spelling_relations"])), (50, 50))
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        authored = {g['frozen_row']:g for g in read('authored/review-base/groups.json')}
        for group in read("groups.json"):
            sense = group["senses"][0]
            self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
            for name, field in sense["fields"].items():
                key = sense["sense_id"], name
                record = fields[key]
                proposed = authored[group['frozen_row']]['senses'][0]['fields'][name]
                self.assertEqual(record["proposed_text"], proposed["text"])
                self.assertEqual(record['accepted_text'], field['text'])
                self.assertEqual(record['proposed_source_span'], proposed['source_span'])
                self.assertEqual(record['accepted_source_span'], field['source_span'])
                self.assertEqual((record["frozen_row"], record["headword"], record["pos"]),
                                 (group["frozen_row"], group["headword"], group["lexique_cgram"]))
                self.assertEqual(record["accepted_text_quality_status"], "passed")
                self.assertEqual(record['rights_eligibility_status'], 'passed_at_selected_text_boundary')
                self.assertEqual(record['reviewer'], review['reviewer'])
                self.assertTrue(record['review_reason'].strip())
                self.assertTrue(record['selected_text_rights_reason'].strip())
                self.assertEqual(legal[key]["independent_review"], "passed_at_selected_text_boundary")
                self.assertEqual(legal[key]['reviewer'], review['reviewer'])
                self.assertEqual(legal[key]['accepted_text'], field['text'])
                self.assertEqual(legal[key]["exact_source_span"], field["source_span"])
                self.assertEqual(legal[key]["modification"], field["modification"])
        for relation in review["spelling_relations"]:
            self.assertEqual(relation["alias_admission"], "not_admitted")
            self.assertEqual(relation["frozen_headword"], relation["source_headword"])
            self.assertEqual(relation["independent_review"], "passed_for_source_binding_only")
            self.assertTrue(relation['review_reason'].strip())
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())
        author_admission = read('authored/review-base/admission.json')
        self.assertEqual(author_admission['independently_accepted_new_fields'], 0)
        self.assertEqual(read('authored/review-base/field-review.json')['reviewer'], None)
        self.assertEqual(read('independent-review-identity.json')['author_commit'],
                         'd75ae7e95ef266629d636a86feb555f463a2ed7d')

    def test_exact_use_field_and_expression_alignment(self):
        review = read('field-review.json')
        uses = {r['sense_id']:r for r in review['usage']}
        expressions = {r['sense_id']:r for r in read('original-expression-review.json')['uses']}
        replacements = {(r['sense_id'],r['field']):r for r in read('accepted-replacements.json')}
        fields = {(r['sense_id'],r['field']):r for r in review['fields']}
        self.assertEqual((len(uses),len(expressions),len(replacements)),(50,50,5))
        changed = set()
        for group in read('groups.json'):
            sense = group['senses'][0]
            sid = sense['sense_id']
            use = uses[sid]
            self.assertEqual((use['frozen_row'],use['headword'],use['pos']),
                             (group['frozen_row'],group['headword'],group['lexique_cgram']))
            self.assertEqual(use['accepted_usage'],sense['use'])
            self.assertEqual(use['accepted_usage_quality_status'],'passed')
            self.assertEqual(use['everyday_sense_status'],'passed')
            self.assertTrue(use['review_reason'].strip())
            expression = expressions[sid]
            self.assertEqual(expression['accepted_use'],sense['use'])
            for name, field in sense['fields'].items():
                record = fields[sid,name]
                self.assertEqual(expression['accepted_texts'][name],field['text'])
                if name != 'ipa':
                    selected = expression['selected_units'][0]
                    self.assertEqual((selected['source_id'],selected['line_1_based'],selected['column_1_based'],selected['reviewed_excerpt']),
                                     (field['source_id'],field['source_span']['line_1_based'],field['source_span']['column_1_based'],field['source_span']['raw_source_excerpt']))
                    supporting = {(r['source_id'],r['line_1_based'],r['column_1_based'],r['reviewed_excerpt']) for r in record['supporting_sources']}
                    for unit in expression['selected_units'][1:]:
                        self.assertIn((unit['source_id'],unit['line_1_based'],unit['column_1_based'],unit['reviewed_excerpt']),supporting)
                if record['proposed_text'] != record['accepted_text']:
                    changed.add((sid,name))
                    replacement = replacements[sid,name]
                    self.assertEqual((replacement['proposed_text'],replacement['accepted_text']),
                                     (record['proposed_text'],record['accepted_text']))
                    self.assertEqual(replacement['accepted_source_span'],field['source_span'])
        self.assertEqual(changed,set(replacements))

    def test_live_reference_tokens_exclude_cfc_prose(self):
        for source in read('sources.json')['sources']:
            for ref in source['external_reference_markers']:
                self.assertTrue(ref['raw_reference'].startswith(('{{R:', '{{R|', '{{Import:')))
                self.assertIn('column_1_based', ref)
        frames = read('source-excerpts.json')['supporting_evidence']
        for frame in frames:
            if frame['frozen_row']==3343 and frame['line_1_based']==16:
                self.assertIn(frame['reviewed_excerpt'], ('{{caprins|fr|sous-cat=Chèvres}}','[[mammifère]]'))

    def test_exact_frozen_groups_have_complete_resource_records(self):
        if (EVIDENCE / "groups.json").exists():
            groups = read("groups.json")
        else:
            canonical = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
            groups = [g for g in canonical["groups"] if 3301 <= g["frozen_row"] <= 3350]
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP,
                         "The frozen #147 resource slice lacks complete batch content")
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3301, 3351)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group in groups:
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


if __name__ == "__main__":
    unittest.main()
