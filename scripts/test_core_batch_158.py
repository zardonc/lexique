"""Check #158 at the approved resource/admission boundary."""
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
EVIDENCE = ROOT / "docs/evidence/issue-158"
MEMBERSHIP = [('majeur', 'ADJ'), ('correctement', 'ADV'), ('dizaine', 'NOM'), ('transfert', 'NOM'), ('obscur', 'ADJ'), ('brouillard', 'NOM'), ('délai', 'NOM'), ('quitte', 'ADJ'), ('recouvrir', 'VER'), ('retomber', 'VER'), ('champignon', 'NOM'), ('contrarier', 'VER'), ('poursuite', 'NOM'), ('confus', 'ADJ'), ('peso', 'NOM'), ('grille', 'NOM'), ('spatial', 'ADJ'), ('chameau', 'NOM'), ('probable', 'ADJ'), ('crayon', 'NOM'), ('gâter', 'VER'), ('adversaire', 'NOM'), ('aimé', 'ADJ'), ('loyal', 'ADJ'), ('surgir', 'VER'), ('atroce', 'ADJ'), ('espionner', 'VER'), ('look', 'NOM'), ('g', 'NOM'), ('balai', 'NOM'), ('canne', 'NOM'), ('désagréable', 'ADJ'), ('modifier', 'VER'), ('testament', 'NOM'), ('cm', 'NOM'), ('convoquer', 'VER'), ('minimum', 'NOM'), ('pétrin', 'NOM'), ('caution', 'NOM'), ('destination', 'NOM'), ('messager', 'NOM'), ('t-shirt', 'NOM'), ('trottoir', 'NOM'), ('bombarder', 'VER'), ('processus', 'NOM'), ('stress', 'NOM'), ('pincer', 'VER'), ('boeuf', 'NOM'), ('sportif', 'ADJ'), ('condoléance', 'NOM')]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch158Tests(unittest.TestCase):
    def test_owner_approved_cm_annotation_selects_no_fields_and_keeps_exact_decision(self):
        group = next(g for g in read('groups.json') if g['frozen_row'] == 3885)
        self.assertEqual(group['senses'], [])
        self.assertEqual((group['headword'], group['lexique_cgram'],
                          group['subtitle_lemma_per_million'], group['subtitle_pos_per_million']),
                         ('cm', 'NOM', '10.89', '10.89'))
        self.assertEqual((group['admitted_fields'], group['missing_fields']), (0, 4))
        self.assertEqual(group['annotation_kind'], 'owner_approved_zero_field')
        approval = read('owner-approval.json')
        decision = group['annotation_exception_decision']
        self.assertEqual(group['owner_approval'], decision)
        self.assertEqual((decision['approved_by'], decision['approved_on'], decision['timezone']),
                         ('project_owner', '2026-11-23', 'America/Vancouver'))
        self.assertEqual(decision['issue'], 'https://github.com/zardonc/lexique/issues/158')
        self.assertEqual(decision['exact_reply'], 'Annotate ya and cm with zero fields')
        self.assertEqual(decision['evidence'], 'docs/evidence/issue-158/owner-approval.json')
        self.assertEqual(decision['decision'], approval['annotation_exception_decision'])
        self.assertEqual(approval['decision_record'], 'context/approved-scope-decisions.md')
        self.assertEqual((approval['frozen_row'], approval['admitted_fields']), (3885, 0))
        review = read('field-review.json')
        self.assertEqual((len(review['fields']), len(review['usage'])), (196, 49))
        self.assertEqual(len(review['spelling_relations']), 49)
        self.assertEqual([a['frozen_row'] for a in review['annotation_exceptions']], [3885])
        self.assertEqual(review['annotation_exceptions'][0]['decision'], decision)
        self.assertEqual(len(review['unresolved_fields']), 4)
        self.assertEqual({(r['frozen_row'], r['field']) for r in review['unresolved_fields']},
                         {(3885, name) for name in ('french_definition', 'ipa', 'english', 'chinese')})
        for record in review['unresolved_fields']:
            self.assertEqual(record['status'], 'unresolved')
            self.assertEqual(record['annotation_exception_decision'], decision)
        self.assertEqual(len(read('rights.json')['fields']), 196)
        self.assertEqual(len(read('source-excerpts.json')['fields']), 196)
        rejected = read('rejected-cm-proposal.json')
        authored = next(g for g in read('authored/groups.json') if g['frozen_row'] == 3885)
        self.assertEqual(rejected['authored_proposal'], authored)
        self.assertEqual(rejected['reviewed_proposal']['senses'][0]['independent_review'],
                         'unresolved_exact_NOM_binding')
        self.assertEqual((len(rejected['field_review']), len(rejected['usage']),
                          len(rejected['rights']), len(rejected['source_excerpts']),
                          len(rejected['spelling_relations'])), (4, 1, 4, 4, 1))
        self.assertEqual(rejected['annotation_exception_decision'], decision)
        admission = read('admission.json')
        self.assertEqual((admission['frozen_groups'], admission['scope_denominator_groups'],
                          admission['accepted_groups'], admission['accepted_fields'],
                          admission['annotation_exception_groups'], admission['missing_fields']),
                         (50, 4087, 49, 196, 1, 4))
        self.assertEqual(admission['unresolved_field_blockers'], [])
        self.assertEqual(admission['approved_annotations'], [group])

    def test_selected_petrin_retains_familiar_register_in_three_meanings(self):
        sense = next(g for g in read('groups.json') if g['frozen_row'] == 3888)['senses'][0]
        self.assertIn('familier', sense['fields']['french_definition']['text'])
        self.assertIn('informal', sense['fields']['english']['text'].lower())
        self.assertIn('口语', sense['fields']['chinese']['text'])

    def test_review_selects_nominal_letter_g_and_retains_rejected_cm_binding(self):
        groups = {g['frozen_row']: g for g in read('groups.json')}
        g = groups[3879]['senses'][0]
        self.assertIn('lettre', g['fields']['french_definition']['text'])
        self.assertEqual(g['fields']['ipa']['text'], '/ʒe/')
        self.assertEqual(g['independent_review'], 'passed')
        cm = read('rejected-cm-proposal.json')['reviewed_proposal']['senses'][0]
        self.assertEqual(cm['independent_review'], 'unresolved_exact_NOM_binding')
        self.assertEqual(read('admission.json')['independently_accepted_new_fields'], 196)

    def test_real_admission_rejects_an_author_review_without_independent_acceptance(self):
        assert_core_load_inputs_rejected(
            self, ROOT / "resources/core-pilot-79", MATERIALS, load_inputs,
            lambda staged: (staged / "field-review.json").write_bytes((EVIDENCE / "authored/field-review.json").read_bytes()),
            "A separate independent field review is required", "admission-158-",
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
        self.assertEqual(len(selected), 196)
        for record in [*excerpts["fields"], *excerpts["supporting_evidence"],
                       *read('rejected-cm-proposal.json')['source_excerpts']]:
            line = raw[record["source_id"]][record["line_1_based"] - 1]
            column = record["column_1_based"] - 1
            self.assertEqual(line[column:column + len(record["reviewed_excerpt"])],
                             record["reviewed_excerpt"])
            self.assertEqual(record["revid"], sources[record["source_id"]]["revid"])
            self.assertNotRegex(record["reviewed_excerpt"].lstrip(), r"^#+\*")
        pos_frames = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "ADV": "adverbe"}
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        for group in read("groups.json"):
            if not group['senses']:
                continue
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
                    if group['frozen_row'] == 3879:
                        self.assertIn('{{S|lettre|fr', pos)
                    else:
                        self.assertIn("{{S|" + pos_frames[group["lexique_cgram"]] + "|fr", pos)
        names, anonymous, chains = verify_archived_contributor_captures(
            self, captures["contributor_registry"]["parts"], resolve)
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])

    def test_measurement_spelling_and_plural_readings_retain_source_restrictions(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        support = read("source-excerpts.json")["supporting_evidence"]
        modern = {r["frozen_row"]: r for r in read("modern-corroboration.json")["records"]}
        for row in (3879, 3885):
            record = modern[row]
            self.assertTrue(record['independent_review'].startswith('passed'))
            self.assertTrue(record['reviewer'])
            self.assertFalse(record["learner_text_copied"])
            self.assertTrue(Path(record["raw_path"]).is_file())
            body = Path(record["readable_path"]).read_text(encoding="utf-8")
            for exact in record["exact_excerpts"]:
                self.assertIn(exact, body)
        g = groups[3879]["senses"][0]
        self.assertEqual((groups[3879]["headword"], groups[3879]["lexique_cgram"]), ("g", "NOM"))
        self.assertEqual(g["fields"]["ipa"]["text"], "/ʒe/")
        self.assertIn('lettre minuscule g', g['fields']['french_definition']['text'])
        self.assertIn('gram', g['excluded_uses'])
        self.assertTrue(any(r['frozen_row'] == 3879 and '{{S|lettre|fr}}' in r['reviewed_excerpt'] for r in support))
        nominal = modern[3879]['nominal_corroboration']
        self.assertEqual(len(nominal), 2)
        for record in nominal:
            body = Path(record['readable_path']).read_text(encoding='utf-8')
            self.assertIn('nom masculin', body)
            for exact in record['exact_excerpts']:
                self.assertIn(exact, body)
        cm = read('rejected-cm-proposal.json')['reviewed_proposal']['senses'][0]
        self.assertEqual((groups[3885]["headword"], groups[3885]["lexique_cgram"]), ("cm", "NOM"))
        self.assertEqual(cm["source_headword"], "centimètre")
        self.assertEqual(cm["fields"]["ipa"]["text"], "/sɑ̃.ti.mɛtʁ/")
        self.assertIn("not a lexical noun", next(r for r in read("source-context.json")["spelling_relations"]
                                                  if r["frozen_row"] == 3885)["construction_and_flexion_interpretation"])
        self.assertTrue(any(r["frozen_row"] == 3885 and "{{S|symbole|fr}}" in r["reviewed_excerpt"] for r in support))
        self.assertEqual(groups[3865]["senses"][0]["source_headword"], "péso")
        self.assertTrue(any(r["frozen_row"] == 3865 and "{{variante de|péso|fr}}" in r["reviewed_excerpt"] for r in support))
        self.assertEqual(groups[3898]["headword"], "boeuf")
        self.assertEqual(groups[3898]["senses"][0]["source_headword"], "bœuf")
        self.assertEqual(groups[3898]["senses"][0]["fields"]["ipa"]["text"], "/bœf/")
        self.assertTrue(any(r["frozen_row"] == 3898 and "Variante par contrainte typographique" in r["reviewed_excerpt"] for r in support))
        for field in ("french_definition", "english", "chinese"):
            self.assertIn("condoléances", groups[3900]["senses"][0]["fields"][field]["text"])
        self.assertTrue(any(r["frozen_row"] == 3900 and "{{généralement pluriel|fr}}" in r["reviewed_excerpt"] for r in support))
        sources = {s["title"]: s for s in read("sources.json")["sources"]}
        transfer = sources["transfert"]
        self.assertTrue(any("Import:CFC" in r["raw_reference"] and r["author_rights_proposal"] == "excluded_from_selected_content"
                            for r in transfer["external_reference_markers"]))
        for location in [*read("source-excerpts.json")["fields"], *support]:
            self.assertFalse(location["source_id"] == transfer["id"] and location["line_1_based"] in (38, 39))
        self.assertTrue(all(r["alias_admission"] == "not_admitted" for r in read("source-context.json")["spelling_relations"]))

    def test_selected_rights_and_quality_have_exact_accepted_alignment(self):
        rights, admission, review = read("rights.json"), read("admission.json"), read("field-review.json")
        self.assertEqual(rights['independent_review'], 'passed_at_selected_text_boundary')
        self.assertTrue(rights['reviewer'])
        self.assertEqual(rights['independent_rights_fields_accepted'], 196)
        self.assertEqual(admission['independently_accepted_new_groups'], 49)
        self.assertEqual(admission['independently_accepted_new_fields'], 196)
        self.assertEqual(admission["missing_admitted_fields"], 4)
        self.assertEqual(admission['pending_independent_review_fields'], 0)
        self.assertEqual(admission['pending_independent_rights_fields'], 0)
        self.assertTrue(review["independent_acceptance"])
        self.assertTrue(review['independence'])
        self.assertTrue(review['reviewer'])
        self.assertEqual((admission["scope_denominator_groups"], admission["proposed_groups"], admission["proposed_fields"]), (4087, 49, 196))
        self.assertEqual(admission["missing_authored_fields"], 0)
        self.assertEqual(admission['unresolved_field_blockers'], [])
        self.assertEqual(admission['pending_independent_review_rows'], [])
        self.assertEqual(len(review["fields"]), 196)
        expected = {(sense["sense_id"], name): field
                    for g in read("groups.json")
                    for sense in g['senses']
                    for name, field in sense["fields"].items()}
        self.assertEqual({(r['sense_id'], r['field']) for r in review['fields']}, set(expected))
        selected = {(r['sense_id'], r['field']): r for r in read('source-excerpts.json')['fields']}
        for record in review['fields']:
            field = expected[record['sense_id'], record['field']]
            self.assertTrue(record['reviewer'])
            self.assertTrue(record['review_reason'])
            self.assertTrue(record['selected_text_rights_reason'])
            exact = record['exact_source']
            self.assertEqual(exact['source_id'], field['source_id'])
            self.assertEqual(exact['line_1_based'], field['source_span']['line_1_based'])
            self.assertEqual(exact['reviewed_excerpt'], field['source_span']['raw_source_excerpt'])
            self.assertEqual(record['accepted_text_quality_status'], 'passed')
            self.assertEqual(record['accepted_text'], field['text'])
            self.assertEqual(selected[record['sense_id'], record['field']]['accepted_text'], record['accepted_text'])
        self.assertEqual(len(review['usage']), 49)
        for use in review['usage']:
            self.assertTrue(use['reviewer'])
            self.assertTrue(use['everyday_sense_reason'])
            self.assertEqual(use['accepted_usage_quality_status'], 'passed')
            group = next(g for g in read('groups.json') if g['frozen_row'] == use['frozen_row'])
            self.assertEqual(use['accepted_usage'], group['senses'][0]['use'])
        self.assertEqual({(r["sense_id"], r["field"]) for r in rights["fields"]}, set(expected))
        for record in rights["fields"]:
            field = expected[record["sense_id"], record["field"]]
            self.assertEqual(record["exact_source_span"], field["source_span"])
            self.assertEqual(record["source_id"], field["source_id"])
            self.assertEqual(record["modification"], field["modification"])
            self.assertEqual(record["license"], "CC-BY-SA-4.0")
            self.assertTrue(record["attribution"])
            self.assertEqual(record['independent_review'], 'passed_at_selected_text_boundary')
            self.assertTrue(record['reviewer'])
            self.assertTrue(record['selected_text_rights_reason'])
            self.assertEqual(record['reviewed_text'], field['text'])
        self.assertEqual((EVIDENCE / "COPYING").read_bytes(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip() + b"\n")
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        for source in read("sources.json")["sources"]:
            self.assertIn(source["fixed_url"], notices)
            self.assertIn(source["history_url"], notices)

    def test_exact_frozen_groups_preserve_frequencies_and_only_cm_has_zero_fields(self):
        groups = read("groups.json") if (EVIDENCE / "groups.json").exists() else []
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3851, 3901)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        frozen = frozen[3850:3900]
        for group, row in zip(groups, frozen, strict=True):
            for key in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[key], row[key])
            self.assertEqual(len(group["senses"]), 0 if group['frozen_row'] == 3885 else 1)
            for sense in group["senses"]:
                self.assertTrue(sense["use"])
                self.assertIsNone(sense["cefr"])
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["source_id"])
                    self.assertEqual(field['independent_review'], 'passed')

    def test_author_snapshot_replacements_and_unresolved_group_are_preserved(self):
        original = read('authored/groups.json')
        authored = {g['frozen_row']: g['senses'][0] for g in original}
        self.assertEqual([(g['headword'], g['lexique_cgram']) for g in original], MEMBERSHIP)
        self.assertIn('9,80665', authored[3879]['fields']['french_definition']['text'])
        replacements = read('accepted-replacements.json')
        self.assertEqual(len(replacements), 7)
        self.assertEqual({(r['frozen_row'], r['field']) for r in replacements},
                         {(3879, k) for k in ('french_definition', 'ipa', 'english', 'chinese')} |
                         {(3888, k) for k in ('french_definition', 'english', 'chinese')})
        for record in replacements:
            self.assertEqual(record['proposed_text'], authored[record['frozen_row']]['fields'][record['field']]['text'])
            self.assertTrue(record['proposed_exact_source'])
        self.assertEqual(read('admission.json')['replacement_text_fields'], 6)
        self.assertEqual({r['field'] for r in read('field-review.json')['unresolved_fields']},
                         {'french_definition', 'ipa', 'english', 'chinese'})
        gap = read('scope-gap.json')
        self.assertEqual((gap['frozen_row'], gap['headword'], gap['lexique_cgram']), (3885, 'cm', 'NOM'))
        self.assertTrue(gap['owner_approval_observed'])
        self.assertTrue(gap['implemented_annotation'])
        self.assertEqual(gap['annotation_exception_decision'],
                         next(g for g in read('groups.json') if g['frozen_row'] == 3885)['annotation_exception_decision'])
        self.assertEqual((gap['accepted_fields'], gap['missing_fields']), (0, 4))


if __name__ == "__main__":
    unittest.main()
