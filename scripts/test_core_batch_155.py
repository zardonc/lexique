"""Check #155 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import unittest

from core_batch_test_helpers import verify_archived_source_captures, verify_archived_contributor_captures

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-155"
MEMBERSHIP = [
    ("colonne", "NOM"), ("clope", "NOM"), ("hériter", "VER"),
    ("nan", "NOM"), ("coopérer", "VER"), ("onde", "NOM"),
    ("chiant", "ADJ"), ("intact", "ADJ"), ("obligé", "ADJ"),
    ("précipiter", "VER"), ("retarder", "VER"), ("réciter", "VER"),
    ("implorer", "VER"), ("phare", "NOM"), ("terriblement", "ADV"),
    ("quelquefois", "ADV"), ("char", "NOM"), ("femelle", "NOM"),
    ("berger", "NOM"), ("chirurgie", "NOM"), ("opportunité", "NOM"),
    ("foire", "NOM"), ("essuyer", "VER"), ("malentendu", "NOM"),
    ("moine", "NOM"), ("cachette", "NOM"), ("instituteur", "NOM"),
    ("miser", "VER"), ("perturber", "VER"), ("octobre", "NOM"),
    ("après-demain", "ADV"), ("horaire", "NOM"), ("portugais", "NOM"),
    ("vieillard", "NOM"), ("familier", "ADJ"), ("sublime", "ADJ"),
    ("actif", "ADJ"), ("potentiel", "NOM"), ("talon", "NOM"),
    ("extra", "ADJ"), ("sueur", "NOM"), ("domicile", "NOM"),
    ("fatal", "ADJ"), ("ronger", "VER"), ("cérébral", "ADJ"),
    ("grenade", "NOM"), ("interrogatoire", "NOM"), ("juré", "NOM"),
    ("menotte", "NOM"), ("audition", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch155Tests(unittest.TestCase):
    def test_frozen_groups_have_complete_scoped_candidates(self):
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3701, 3751)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group, row in zip(groups, frozen[3700:3750], strict=True):
            self.assertEqual((group["headword"], group["lexique_cgram"]),
                             (row["word"], row["lexique_cgram"]))
            self.assertEqual(group["subtitle_lemma_per_million"], row["subtitle_lemma_per_million"])
            self.assertEqual(group["subtitle_pos_per_million"], row["subtitle_pos_per_million"])
            self.assertTrue(group["senses"])
            for sense in group["senses"]:
                self.assertTrue(sense["use"].strip())
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["source_span"]["raw_source_excerpt"].strip())

    def test_noun_spelling_and_selected_pos_frames_keep_their_actual_use(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        nan = groups[3704]["senses"][0]
        self.assertEqual(nan["source_headword"], "naan")
        self.assertEqual(nan["fields"]["ipa"]["text"], "/nan/")
        relations = read("source-context.json")["spelling_relations"]
        relation = next(r for r in relations if r["frozen_row"] == 3704)
        self.assertEqual(relation["relation"], "attested_french_noun_spelling_variant")
        self.assertEqual(relation["alias_admission"], "not_admitted")
        self.assertTrue(any(r["reviewed_excerpt"] == "* [[nan]]" for r in relation["proposed_evidence"]))
        pos_names = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "ADV": "adverbe"}
        for usage in read("field-review.json")["usage"]:
            group = groups[usage["frozen_row"]]
            supporting = [s["reviewed_excerpt"] for s in usage["supporting_sources"]]
            self.assertIn("=={{langue|fr}}==", {"".join(s.split()) for s in supporting})
            self.assertTrue(any(s.startswith("=== {{S|" + pos_names[group["lexique_cgram"]] + "|fr")
                                for s in supporting))
        for field in read("source-excerpts.json")["fields"]:
            self.assertNotIn("Import:CFC", field["reviewed_excerpt"])
        cfc = next(r for s in read("rights.json")["sources"] if s["title"] == "talon"
                   for r in s["reference_markers"] if "Import:CFC" in r["raw_reference"])
        self.assertEqual(cfc["author_rights_proposal"], "excluded_from_selected_content")

    def test_contributor_histories_are_complete_and_match_source_credits(self):
        captures = read("source-captures.json")
        names, anonymous, chains = verify_archived_contributor_captures(
            self, captures["contributor_registry"]["parts"], Path)
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(set(context["current_contributors"]), names[context["title"]])
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
            self.assertTrue(context["history_url"].endswith("&action=history"))

    def test_wave_and_day_count_explanations_match_the_selected_meanings(self):
        groups = {g['frozen_row']: g for g in read('groups.json')}
        wave = groups[3706]['senses'][0]['fields']
        expected = [
            ('onde/FR', wave['french_definition']['text'],
             'Onde : perturbation qui se propage, notamment une onde sonore ou électromagnétique.'),
            ('onde/EN', wave['english']['text'],
             'Wave; a propagating disturbance, such as a sound or electromagnetic wave.'),
            ('après-demain/ZH', groups[3731]['senses'][0]['fields']['chinese']['text'],
             '后天；今天之后的第二天（副词）。'),
        ]
        for name, actual, text in expected:
            with self.subTest(field=name):
                self.assertEqual(actual, text)

    def test_audition_selection_scope_uses_casting_equivalents(self):
        sense = read('groups.json')[-1]['senses'][0]
        with self.subTest(field='english'):
            self.assertEqual(sense['fields']['english']['text'],
                             'Casting; a process of selecting performers for roles in a film, play or programme.')
        with self.subTest(field='chinese'):
            self.assertEqual(sense['fields']['chinese']['text'],
                             '选角；为电影、戏剧或节目角色挑选表演者的方式（阴性名词）。')

    def test_independent_acceptance_matches_exact_proposals_sources_and_rights(self):
        admission, review, rights = read("admission.json"), read("field-review.json"), read("rights.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["independently_accepted_new_groups"], 50)
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual(admission["missing_authored_fields"], 0)
        self.assertEqual(review['reviewer'], 'lexique-review-155 / codex/d2-review-155')
        self.assertTrue(review["independent_acceptance"])
        self.assertEqual(review['unresolved_fields'], [])
        accepted = {(g["frozen_row"], name): field for g in read("groups.json")
                    for name, field in g["senses"][0]["fields"].items()}
        proposed = {(g['frozen_row'], name): field for g in read('authored/groups.json')
                    for name, field in g['senses'][0]['fields'].items()}
        reviews = {(r['frozen_row'], r['field']): r for r in review['fields']}
        self.assertEqual(set(accepted), set(proposed))
        self.assertEqual(set(accepted), set(reviews))
        self.assertEqual(len(rights["fields"]), 200)
        for record in rights["fields"]:
            key = record['frozen_row'], record['field']
            field = accepted[key]
            self.assertEqual(record["exact_source_span"], field["source_span"])
            self.assertEqual(record["license"], "CC-BY-SA-4.0")
            self.assertTrue(record["attribution"].strip())
            self.assertEqual(record['independent_review'], 'passed')
            self.assertEqual(record['accepted_text'], field['text'])
            self.assertEqual(record['reviewer'], review['reviewer'])
            self.assertEqual(record['rights_eligibility_status'], 'passed_at_selected_text_boundary')
            checked = reviews[key]
            self.assertEqual(checked['proposed_text'], proposed[key]['text'])
            self.assertEqual(checked['accepted_text'], field['text'])
            self.assertEqual(checked['accepted_text_quality_status'], 'passed')
            self.assertEqual(checked['rights_review_reason'], record['review_reason'])
            self.assertTrue(checked['review_reason'].strip())
            self.assertEqual(checked['accepted_source'], checked['exact_source'])
            self.assertEqual(field['independent_review'], 'passed')
        changed = {key for key in accepted if accepted[key]['text'] != proposed[key]['text']}
        self.assertEqual(changed, {(3706, 'french_definition'), (3706, 'english'), (3731, 'chinese'),
                                  (3750, 'english'), (3750, 'chinese')})
        replacements = {(r['frozen_row'], r['field']): r for r in read('accepted-replacements.json')}
        self.assertEqual(set(replacements), changed)
        for key, replacement in replacements.items():
            self.assertEqual(replacement['proposed_text'], proposed[key]['text'])
            self.assertEqual(replacement['accepted_text'], accepted[key]['text'])
        uses = {r['frozen_row']: r for r in review['usage']}
        for group in read('groups.json'):
            usage = uses[group['frozen_row']]
            self.assertEqual(usage['accepted_usage'], group['senses'][0]['use'])
            self.assertEqual(usage['accepted_usage_quality_status'], 'passed')
            self.assertEqual(usage['everyday_sense_status'], 'passed')
            self.assertTrue(usage['everyday_sense_reason'].strip())
        self.assertEqual(len(review['spelling_relations']), 50)
        for relation in review['spelling_relations']:
            self.assertEqual(relation['independent_review'], 'passed')
            self.assertEqual(relation['alias_admission'], 'not_admitted')

    def test_original_expression_and_combined_notice_evidence_is_actual(self):
        originals = read('original-expression-review.json')
        self.assertEqual(len(originals['sources']), 52)
        self.assertEqual(originals['unresolved_sources'], [])
        for source in originals['sources']:
            for original in source['original_comparisons']:
                raw_path = Path(original['raw_path'])
                self.assertTrue(raw_path.is_file())
                text = Path(original['plain_text_path']).read_text(encoding='utf-8')
                start = original['plain_text_column_1_based'] - 1
                self.assertEqual(text[start:start + len(original['original_expression'])],
                                 original['original_expression'])
                self.assertTrue(original['dictionary'])
                self.assertEqual(original['independent_review'], 'passed_at_original_work_boundary')
    def test_franceterme_original_has_its_exact_terms_and_notice(self):
        source = next(s for s in read('sources.json')['sources'] if s['title'] == 'audition')
        layers = [l for l in source['original_third_party_work_layers']
                  if l.get('original_work_id') == 'FranceTerme-CULT430']
        self.assertTrue(layers, 'The selected audition wording requires the FranceTerme original-work layer')
        layer = layers[0]
        self.assertEqual(layer['license'], 'Licence-Ouverte-1.0')
        self.assertEqual(layer['publication_date'], '2005-01-18')
        self.assertEqual(layer['url'], 'https://www.culture.fr/franceterme/terme/CULT430')
        self.assertTrue((EVIDENCE/'LICENSE-FranceTerme.pdf').read_bytes().startswith(b'%PDF-'))
        self.assertIn('FranceTerme', (EVIDENCE/'NOTICES.md').read_text(encoding='utf-8'))
        self.assertIn('18 January 2005', (EVIDENCE/'NOTICES.md').read_text(encoding='utf-8'))
        self.assertEqual(read('rights.json')['final_combined_package_status'], 'coordinator_owned_pending')

    def test_fixed_source_bytes_and_selected_spans_match_recorded_revisions(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        lines = verify_archived_source_captures(self, captures["sources"], sources, Path)
        excerpts = {(f["sense_id"], f["field"]): f for f in read("source-excerpts.json")["fields"]}
        for group in read("groups.json"):
            for sense in group["senses"]:
                for name, field in sense["fields"].items():
                    exact = excerpts[sense["sense_id"], name]
                    raw = lines[field["source_id"]][exact["line_1_based"] - 1]
                    self.assertEqual(raw, field["source_span"]["raw_source_excerpt"])
                    self.assertEqual(raw, exact["reviewed_excerpt"])
                    self.assertEqual(exact["revid"], sources[field["source_id"]]["revid"])
                    if name == "ipa":
                        self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
                        start = exact["ipa_variant_column_1_based"] - 1
                        self.assertEqual(raw[start:start + len(exact["ipa_variant_excerpt"])], exact["ipa_variant_excerpt"])
                    else:
                        self.assertTrue(raw.startswith("#"))
                        self.assertFalse(raw.startswith("#*"))
        for record in read("source-excerpts.json")["supporting_evidence"]:
            raw = lines[record["source_id"]][record["line_1_based"] - 1]
            start = record["column_1_based"] - 1
            self.assertEqual(raw[start:start + len(record["reviewed_excerpt"])], record["reviewed_excerpt"])


if __name__ == "__main__":
    unittest.main()
