"""Check #156 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import unittest

from core_batch_test_helpers import (
    assert_core_load_inputs_rejected,
    verify_archived_contributor_captures,
    verify_archived_source_captures,
)
from core_pilot import load_inputs, MATERIALS

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-156"
MEMBERSHIP = [
    ("caravane", "NOM"),
    ("comté", "NOM"),
    ("fascinant", "ADJ"),
    ("survie", "NOM"),
    ("symptôme", "NOM"),
    ("frappe", "NOM"),
    ("intellectuel", "ADJ"),
    ("redescendre", "VER"),
    ("damer", "VER"),
    ("antenne", "NOM"),
    ("descente", "NOM"),
    ("potion", "NOM"),
    ("répugnant", "ADJ"),
    ("collège", "NOM"),
    ("culpabilité", "NOM"),
    ("tribu", "NOM"),
    ("évoquer", "VER"),
    ("mafia", "NOM"),
    ("orgueil", "NOM"),
    ("ramper", "VER"),
    ("fermeture", "NOM"),
    ("mouchoir", "NOM"),
    ("obséder", "VER"),
    ("gaspiller", "VER"),
    ("bail", "NOM"),
    ("précédemment", "ADV"),
    ("troupeau", "NOM"),
    ("calcul", "NOM"),
    ("coquin", "NOM"),
    ("évidence", "NOM"),
    ("plaider", "VER"),
    ("scanner", "NOM"),
    ("ya", "NOM"),
    ("honorable", "ADJ"),
    ("ressusciter", "VER"),
    ("télégramme", "NOM"),
    ("majorité", "NOM"),
    ("enfiler", "VER"),
    ("glacé", "ADJ"),
    ("insupportable", "ADJ"),
    ("graine", "NOM"),
    ("piqûre", "NOM"),
    ("tâcher", "VER"),
    ("tonnerre", "NOM"),
    ("valet", "NOM"),
    ("actuellement", "ADV"),
    ("cheminée", "NOM"),
    ("maquillage", "NOM"),
    ("dérouler", "VER"),
    ("hanter", "VER"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch156Tests(unittest.TestCase):
    def test_only_owner_approved_ya_has_zero_fields_and_four_explicit_missing_fields(self):
        group = next(g for g in read('groups.json') if g['frozen_row'] == 3783)
        self.assertEqual((group['headword'], group['lexique_cgram']), ('ya', 'NOM'))
        self.assertEqual((group['subtitle_lemma_per_million'], group['subtitle_pos_per_million']),
                         ('11.45', '11.45'))
        self.assertEqual(group['senses'], [])
        self.assertEqual(group['status'], 'approved_annotation_exception')
        self.assertEqual(group['annotation_kind'], 'owner_approved_zero_field')
        self.assertEqual((group['admitted_fields'], group['missing_fields']), (0, 4))
        self.assertEqual(group['alias_admission'], 'not_admitted')
        review = read('field-review.json')
        self.assertEqual((len(review['fields']), len(review['usage'])), (196, 49))
        self.assertEqual([r['frozen_row'] for r in review['annotation_exceptions']], [3783])
        decision = review['annotation_exceptions'][0]['decision']
        self.assertEqual((decision['approved_by'], decision['approved_on'], decision['timezone'], decision['issue']),
                         ('project_owner', '2026-11-23', 'America/Vancouver',
                          'https://github.com/zardonc/lexique/issues/156'))
        self.assertEqual(decision['exact_reply'], 'Annotate ya and cm with zero fields')
        approval = read('owner-approval.json')
        self.assertEqual(approval['status'], 'approved_by_project_owner')
        self.assertEqual((approval['issue'], approval['frozen_row'], approval['headword'], approval['lexique_cgram']),
                         (156, 3783, 'ya', 'NOM'))
        self.assertEqual(approval['admitted_fields'], 0)
        self.assertEqual(approval['missing_fields'],
                         ['french_definition', 'ipa', 'english_meaning', 'chinese_meaning'])
        self.assertEqual(decision['decision'], approval['annotation_exception_decision'])
        self.assertEqual(decision['exact_reply'], approval['exact_reply'])
        self.assertEqual(group['owner_approval'], decision)
        self.assertEqual(group['annotation_exception_decision'], decision)
        self.assertEqual({(r['frozen_row'], r['field']) for r in review['unresolved_fields']},
                         {(3783, name) for name in ('french_definition', 'ipa', 'english', 'chinese')})
        for record in review['unresolved_fields']:
            self.assertEqual(record['status'], 'unresolved')
            self.assertEqual(record['annotation_exception_decision'], decision)
        admission = read('admission.json')
        self.assertEqual(admission['scope_denominator_groups'], 4087)
        self.assertEqual((admission['proposed_groups'], admission['proposed_fields']), (49, 196))
        self.assertEqual(admission['approved_annotation_rows'], [3783])
        self.assertEqual(admission['approved_annotations'], [group])
        self.assertEqual(admission['annotation_exception_groups'], 1)
        self.assertEqual(admission['specialist_exception_groups'], 0)
        self.assertEqual((admission['missing_admitted_fields'], admission['annotation_missing_fields'],
                          admission['ordinary_missing_fields']), (4, 4, 0))
        self.assertEqual(admission['unresolved_field_blockers'], [])
        rejected = read('rejected-proposals.json')['proposals'][0]
        self.assertEqual(rejected['authored_proposal'],
                         next(g for g in read('authored/groups.json') if g['frozen_row'] == 3783))
        self.assertEqual(len(rejected['independently_reviewed_fields']), 4)
        self.assertTrue(all(r['accepted_text'] is None for r in rejected['independently_reviewed_fields']))
        self.assertEqual(rejected['independently_reviewed_usage']['everyday_sense_status'], 'unresolved')

    def test_independent_review_accepts_only_selected_fields_and_retains_proposal_rights(self):
        review = read('field-review.json')
        self.assertTrue(review['independent_acceptance'])
        self.assertTrue(review['reviewer'])
        self.assertEqual(len(review['fields']), 196)
        accepted = [r for r in review['fields'] if r['accepted_text_quality_status'] == 'passed']
        self.assertEqual(len(accepted), 196)
        self.assertEqual({r['frozen_row'] for r in review['unresolved_fields']}, {3783})
        self.assertEqual(len(review['unresolved_fields']), 4)
        for r in review['fields']:
            self.assertTrue(r['review_reason'])
            self.assertTrue(r['selected_text_rights_reason'])
            self.assertEqual(r['rights_eligibility_status'], 'passed_at_selected_text_boundary')
            self.assertEqual(r['reviewer'], review['reviewer'])
            self.assertTrue(r['independence'])
        for u in review['usage']:
            self.assertTrue(u['review_reason'])
            self.assertEqual(u['everyday_sense_status'], 'passed')
        admission = read('admission.json')
        self.assertEqual(admission['independently_accepted_new_groups'], 49)
        self.assertEqual(admission['independently_accepted_new_fields'], 196)
        self.assertEqual(admission['canonical_admitted_new_fields'], 0)
        expected = {(sense['sense_id'], name): (g, field)
                    for g in read('groups.json') for sense in g['senses']
                    for name, field in sense['fields'].items()}
        reviewed = {(r['sense_id'], r['field']): r for r in review['fields']}
        self.assertEqual(set(reviewed), set(expected))
        for key, (group, field) in expected.items():
            record = reviewed[key]
            self.assertEqual(record['proposed_text'], field['text'])
            self.assertEqual(record['accepted_text'], field['text'])
            self.assertEqual(record['exact_source']['reviewed_excerpt'], field['source_span']['raw_source_excerpt'])
            self.assertEqual(record['exact_source']['source_id'], field['source_id'])
        self.assertEqual(len({r['review_reason'] for r in review['fields']}), 196)
        self.assertEqual(len({u['review_reason'] for u in review['usage']}), 49)
        rejected = read('rejected-proposals.json')['proposals'][0]
        rejected_group = rejected['independently_reviewed_proposal']
        for sense in rejected_group['senses']:
            for name, field in sense['fields'].items():
                expected[sense['sense_id'], name] = (rejected_group, field)
        reviewed.update({(r['sense_id'], r['field']): r for r in rejected['independently_reviewed_fields']})
        rights = read('rights.json')
        self.assertEqual(rights['independent_rights_fields_accepted'], 200)
        self.assertEqual(len(rights['sources']), 50)
        self.assertEqual(len(rights['fields']), 200)
        self.assertEqual(admission['rights_accepted_fields'], 196)
        self.assertEqual(admission['audited_proposal_rights_fields'], 200)
        self.assertEqual(admission['audited_proposal_rights_uses'], 50)
        for record in rights['fields']:
            group, field = expected[record['sense_id'], record['field']]
            self.assertEqual(record['exact_source_span'], field['source_span'])
            self.assertEqual(record['independent_review'], 'passed_at_selected_text_boundary')
            self.assertEqual(record['accepted_text'], reviewed[record['sense_id'], record['field']]['accepted_text'])
            self.assertEqual(record['modification'], field['modification'])
        ya = read('ya-modern-use-review.json')
        self.assertEqual(ya['semantic_alignment'], 'confirmed_exact_masculine_slang_knife_noun')
        self.assertEqual(ya['everyday_sense_status'], 'unresolved')
        self.assertEqual(ya['primary_bob']['current_register'], 10)
        self.assertEqual(ya['primary_cordial']['http_status'], 200)
        self.assertFalse(ya['changed_spelling'])

    def test_mouchoir_selected_rights_separate_quillet_and_daf9_expression(self):
        source = next(s for s in read('sources.json')['sources'] if s['title'] == 'mouchoir')
        boundaries = source['selected_original_expression_boundaries']
        self.assertEqual({b['work'] for b in boundaries}, {'Dictionnaire encyclopédique Quillet', 'Dictionnaire de l’Académie française, ninth edition'})
        self.assertTrue(all(b['external_expression_admitted'] is False for b in boundaries))
        self.assertIn('cellulose', source['selected_text_rights_reason'])
        valet = next(s for s in read('sources.json')['sources'] if s['title'] == 'valet')
        later = valet['selected_original_expression_boundaries'][0]
        self.assertEqual(later['edition'], '1979')
        self.assertFalse(later['external_expression_admitted'])
        self.assertEqual(later['primary_selected_expression_authority'], 'https://www.dictionnaire-academie.fr/article/A8V0052')

    def test_difficult_bindings_preserve_pos_register_spelling_and_unresolved_use(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        scanner = groups[3782]["senses"][0]
        self.assertEqual(scanner["fields"]["ipa"]["text"], "/ska.nɛʁ/")
        self.assertIn("medical", scanner["use"].lower())
        self.assertIn("France", groups[3764]["senses"][0]["fields"]["english"]["text"])
        self.assertIn("familièrement", groups[3779]["senses"][0]["fields"]["french_definition"]["text"].lower())
        self.assertIn("tâcher de", groups[3793]["senses"][0]["fields"]["english"]["text"])
        self.assertIn("aspirated h", groups[3800]["senses"][0]["use"])
        ya = groups[3783]
        self.assertEqual((ya["headword"], ya["lexique_cgram"]), ("ya", "NOM"))
        rejected = read('rejected-proposals.json')['proposals'][0]
        self.assertEqual(rejected['independently_reviewed_proposal']['senses'][0]['fields']['french_definition']['text'],
                         'En argot, couteau.')
        self.assertEqual(ya['senses'], [])
        self.assertEqual(read("admission.json")["author_unresolved_content_rows"], [3783])
        usage = rejected['independently_reviewed_usage']
        self.assertEqual(usage["author_everyday_use_assessment"], "unresolved_modern_everyday_slang_use")
        self.assertEqual({r["frozen_row"] for r in read("field-review.json")["unresolved_fields"]}, {3783})
        self.assertEqual(len(read("field-review.json")["unresolved_fields"]), 4)
        for relation in read("source-context.json")["spelling_relations"]:
            self.assertEqual(relation["relation_role"], "source_binding_only")
            self.assertEqual(relation["alias_admission"], "not_admitted")

    def test_real_admission_rejects_an_author_review_without_independent_acceptance(self):
        assert_core_load_inputs_rejected(
            self, ROOT / "resources/core-pilot-79", MATERIALS, load_inputs,
            lambda staged: (staged / "field-review.json").write_bytes((EVIDENCE / "authored/field-review.json").read_bytes()),
            "A separate independent field review is required", "admission-156-",
        )

    def test_fixed_source_bytes_spans_frames_and_complete_contributors(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        archive = Path(captures["capture_root"])
        def resolve(value):
            path = Path(value)
            self.assertTrue(path.is_relative_to(archive), value)
            self.assertTrue(path.is_file(), value)
            return path
        raw = verify_archived_source_captures(self, captures["sources"], sources, resolve)
        excerpts = read("source-excerpts.json")
        selected = {(r["sense_id"], r["field"]): r for r in excerpts["fields"]}
        self.assertEqual(len(selected), 200)
        for record in [*excerpts["fields"], *excerpts["supporting_evidence"]]:
            line = raw[record["source_id"]][record["line_1_based"] - 1]
            column = record["column_1_based"] - 1
            self.assertEqual(line[column:column + len(record["reviewed_excerpt"])],
                             record["reviewed_excerpt"])
            self.assertEqual(record["revid"], sources[record["source_id"]]["revid"])
            self.assertNotRegex(record["reviewed_excerpt"].lstrip(), r"^#+\*")
        pos_frames = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "ADV": "adverbe"}
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        rejected = read('rejected-proposals.json')['proposals'][0]['independently_reviewed_proposal']
        for group in [g for g in read('groups.json') if g['senses']] + [rejected]:
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                exact = selected[sense["sense_id"], name]
                self.assertEqual(field["source_id"], exact["source_id"])
                self.assertEqual(field["language"], languages[name])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], exact["reviewed_excerpt"])
                before = raw[exact["source_id"]][:exact["line_1_based"]]
                language = next(line for line in reversed(before) if line.startswith("== "))
                self.assertRegex(language, r"^==\s*\{\{langue\|fr\}\}\s*==$")
                if name == "ipa":
                    self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
                    column = exact["ipa_variant_column_1_based"] - 1
                    self.assertEqual(exact["reviewed_excerpt"][column:column + len(exact["ipa_variant_excerpt"])],
                                     exact["ipa_variant_excerpt"])
                else:
                    pos = next(line for line in reversed(before) if line.startswith("=== {{S|"))
                    self.assertIn("{{S|" + pos_frames[group["lexique_cgram"]] + "|fr", pos)
        names, anonymous, chains = verify_archived_contributor_captures(
            self, captures["contributor_registry"]["parts"], resolve)
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])

    def test_authored_snapshot_preserves_pending_proposals_and_exact_text(self):
        rights, admission, review = read("authored/rights.json"), read("authored/admission.json"), read("authored/field-review.json")
        self.assertEqual(rights["independent_review"], "pending")
        self.assertIsNone(rights["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 0)
        self.assertEqual(admission["independently_accepted_new_groups"], 0)
        self.assertEqual(admission["independently_accepted_new_fields"], 0)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["pending_independent_review_fields"], 200)
        self.assertEqual(admission["pending_independent_rights_fields"], 200)
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        rejected = read('rejected-proposals.json')['proposals'][0]['independently_reviewed_proposal']
        expected = {(g["senses"][0]["sense_id"], name): field
                    for g in [g for g in read('groups.json') if g['senses']] + [rejected]
                    for name, field in g["senses"][0]["fields"].items()}
        self.assertEqual({(r["sense_id"], r["field"]) for r in rights["fields"]}, set(expected))
        for record in rights["fields"]:
            field = expected[record["sense_id"], record["field"]]
            self.assertEqual(record["exact_source_span"], field["source_span"])
            self.assertEqual(record["source_id"], field["source_id"])
            self.assertEqual(record["modification"], field["modification"])
            self.assertEqual(record["license"], "CC-BY-SA-4.0")
            self.assertTrue(record["attribution"])
            self.assertEqual(record["independent_review"], "pending")
            self.assertIsNone(record["reviewer"])
        self.assertEqual((EVIDENCE / "COPYING").read_bytes(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip() + b"\n")

    def test_exact_frozen_groups_have_four_selected_fields_or_approved_annotation(self):
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3751, 3801)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            frozen = list(csv.DictReader(handle))[3750:3800]
        for group, row in zip(groups, frozen, strict=True):
            for key in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[key], row[key])
            if group['frozen_row'] == 3783:
                self.assertEqual(group['senses'], [])
                continue
            self.assertTrue(group["senses"])
            for sense in group["senses"]:
                self.assertTrue(sense["use"])
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["source_id"])
                    self.assertEqual(field["independent_review"], "passed")

    def test_real_admission_rejects_an_unresolved_independent_quality_decision(self):
        def mutate(staged):
            review = json.loads((staged / "field-review.json").read_text(encoding="utf-8"))
            review["fields"][0]["accepted_text_quality_status"] = "unresolved"
            (staged / "field-review.json").write_text(json.dumps(review), encoding="utf-8")

        assert_core_load_inputs_rejected(
            self, ROOT / "resources/core-pilot-79", MATERIALS, load_inputs,
            mutate, "A selected field is not independently reviewed or rights-eligible", "quality-156-",
        )


if __name__ == "__main__":
    unittest.main()
