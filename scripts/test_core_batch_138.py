"""Check reviewed #138 evidence; --admitted separately requires canonical admission."""
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


def review_path(name):
    archive = Path(read('admission.json')['review_evidence_root'])
    path = archive / name
    return path if path.is_file() else ROOT / 'target/issue-138-review' / name


class Batch138Tests(unittest.TestCase):
    def test_accepted_wording_retains_material_modality_and_scope(self):
        groups = {g['frozen_row']: g for g in read('groups.json')}
        expected = {
            (2856, 'french_definition'): 'Nom qu’une personne écrit de sa main à la fin d’une lettre, d’un contrat ou d’un acte pour le certifier ou le rendre valable.',
            (2856, 'english'): 'A signature: a person’s handwritten name at the end of a letter, contract or other document to certify or validate it.',
            (2856, 'chinese'): '签名：本人亲笔写在信件、合同或其他文书末尾的姓名，用以确认或使其有效。',
            (2872, 'chinese'): '小动物笼：用铁丝或柳条制成、可搬动的封闭小笼子。',
            (2886, 'french_definition'): 'Morceau de bois assez long que l’on peut tenir à la main et qui peut servir d’appui, d’arme ou de hampe.',
            (2886, 'english'): 'A stick or staff: a fairly long piece of wood that can be held in the hand, usable as a support, weapon or shaft.',
            (2887, 'chinese'): '塑料：以称为聚合物的大分子为基础的合成有机材料。',
            (2892, 'chinese'): '蜂蜜：蜜蜂利用从花朵和植物叶片采集的汁液制成的黏稠、甜味液态物质。',
            (2893, 'chinese'): '同时发生：几件事在同一时间出现。',
        }
        for (row, name), text in expected.items():
            with self.subTest(row=row, field=name):
                self.assertEqual(groups[row]['senses'][0]['fields'][name]['text'], text)

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

    def test_review_preserves_frozen_frequencies_four_fields_and_honest_gates(self):
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
                status = 'unresolved_everyday_eligibility' if group['frozen_row'] == 2894 else 'passed'
                self.assertEqual(sense['independent_review'], status)
                self.assertEqual(sense['resource_admission'],
                                 'blocked_unresolved_everyday_eligibility' if group['frozen_row'] == 2894
                                 else 'pending_integration_admission')
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"].strip())
                    self.assertEqual(field['independent_review'], status)
                    self.assertEqual(field['rights_eligibility_status'], 'passed_at_selected_text_boundary')
        reuse = read("reuse.json")
        self.assertEqual((reuse["reused_groups"], reuse["reused_fields"], reuse["groups"]), (0, 0, []))
        admission = read("admission.json")
        self.assertEqual(admission["author_evidence_root"],
                         "C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-138")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"],
                          admission["independently_accepted_new_fields"]), (49, 196))
        self.assertEqual(admission['independent_reviewer'], read('field-review.json')['reviewer'])
        self.assertEqual(admission['pending_independent_review_rows'], [2894])
        self.assertEqual(admission['pending_independent_review_fields'], 4)
        self.assertEqual(admission['pending_independent_rights_fields'], 0)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission['newly_admitted_fields'], 0)
        self.assertEqual(admission['annotation_exception_groups'], 0)
        self.assertEqual(admission['specialist_exception_groups'], 0)
        self.assertEqual(admission['unresolved_semantic_rows'], [2894])
        self.assertEqual(admission['unresolved_source_rows'], [])
        self.assertEqual(admission['unresolved_legal_rows'], [])
        self.assertEqual(admission["missing_authored_fields"], 0)
        self.assertEqual(admission["native_checks"], "integration_owned_not_run")
        self.assertEqual(admission['conversion_and_packaging'], 'integration_owned_not_run')
        self.assertEqual(admission['learner_acceptance'], 'not_run')

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

    def test_fields_bind_exact_french_pos_ipa_rights_and_original_proposals(self):
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        raw = {c["source_id"]: retained_path(c["raw_path"]).read_text(encoding="utf-8").splitlines()
               for c in read("source-captures.json")["sources"]}
        rights = read("rights.json")
        proposals = {(r["sense_id"], r["field"]): r for r in rights["fields"]}
        reviews = {(r["sense_id"], r["field"]): r for r in read("field-review.json")["fields"]}
        excerpts = {(r["sense_id"], r["field"]): r for r in read("source-excerpts.json")["fields"]}
        authored = {(g['senses'][0]['sense_id'], name): field
                    for g in read('authored/at-81f6aa9/groups.json')
                    for name, field in g['senses'][0]['fields'].items()}
        self.assertEqual(len(proposals), 200)
        self.assertEqual(set(proposals), set(reviews))
        self.assertEqual(set(proposals), set(excerpts))
        self.assertTrue(read('field-review.json')['independent_acceptance'])
        self.assertEqual(read('field-review.json')['kind'], 'separate_independent_field_review')
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
                    self.assertEqual(review['proposed_text'], authored[key]['text'])
                    self.assertEqual(review['accepted_source_span'], field['source_span'])
                    self.assertEqual(excerpt["reviewed_excerpt"], span["raw_source_excerpt"])
                    accepted = None if group['frozen_row'] == 2894 else field['text']
                    self.assertEqual(review['accepted_text'], accepted)
                    self.assertEqual(excerpt['accepted_text'], accepted)
                    self.assertEqual(proposal['accepted_text'], accepted)
                    self.assertEqual(review['accepted_text_quality_status'], field['independent_review'])
                    self.assertEqual(review['reviewer'], read('field-review.json')['reviewer'])
                    self.assertTrue(review['independence'])
                    self.assertTrue(review['review_reason'])
                    self.assertTrue(review['selected_text_rights_reason'])
                    self.assertEqual(review['rights_eligibility_status'], 'passed_at_selected_text_boundary')
                    self.assertEqual(proposal['independent_review'], 'passed_at_selected_text_boundary')
                    self.assertEqual(proposal['quality_admission_status'], field['independent_review'])
                    self.assertTrue(proposal["author_rights_basis"])
                    self.assertTrue(proposal["attribution"])
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
        for support in read("source-excerpts.json")["supporting_evidence"]:
            line = raw[support["source_id"]][support["line_1_based"] - 1]
            column = support["column_1_based"] - 1
            self.assertEqual(line[column:column + len(support["reviewed_excerpt"])], support["reviewed_excerpt"])
        self.assertEqual(rights['independent_rights_fields_accepted'], 200)
        self.assertEqual(rights['final_combined_package_status'], 'pending_coordinator_combined_artifact_verification')
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

    def test_source_context_markers_match_completed_selected_text_verdicts(self):
        context = read('source-context.json')
        contexts = context['current_content']
        rights = read('rights.json')
        sources = {s['source_id']: s for s in rights['sources']}
        audit = {s['source_id']: s for s in read('source-audit.json')['sources']}
        reviews = {(f['sense_id'], f['field']): f
                   for f in read('field-review.json')['fields']}
        fields = {(f['sense_id'], f['field']): f for f in rights['fields']}
        passed = 'passed_at_selected_text_boundary'
        self.assertEqual(len(contexts), 50)
        self.assertEqual({c['id'] for c in contexts}, set(sources))
        self.assertEqual(set(sources), set(audit))
        for c in contexts:
            with self.subTest(source=c['id']):
                source, capture = sources[c['id']], audit[c['id']]
                self.assertEqual(c.get('selected_text_rights_status'), passed)
                self.assertEqual(c['status'], 'passed_independent_selected_source_audit')
                self.assertEqual(source['independent_review'], passed)
                self.assertEqual(c['current_content_revid'], source['revid'])
                self.assertEqual(c['current_content_revid'], capture['selected_revid'])
                self.assertEqual(c['selected_locations'], source['selected_locations'])
                self.assertEqual(c['selected_locations'], capture['selected_locations'])
                self.assertEqual(c['independent_source_audit'], 'source-audit.json#' + c['id'])
                self.assertTrue(c['selected_text_rights_reason'])
                self.assertEqual(c['selected_text_rights_reason'], source['selected_text_rights_reason'])
                self.assertEqual(c['selected_text_rights_reason'], capture['selected_source_rights_reason'])
                for verdict in (c, source, capture):
                    self.assertEqual(verdict['reviewer'], context['independent_reviewer'])
                self.assertEqual({p['field'] for p in c['selected_locations']}, set(LANGUAGES))
                for location in c['selected_locations']:
                    key = (location['sense_id'], location['field'])
                    field, review = fields[key], reviews[key]
                    self.assertEqual(field['source_id'], c['id'])
                    self.assertEqual(field['source_revision'], c['current_content_revid'])
                    self.assertEqual(field['independent_review'], passed)
                    self.assertEqual(review['rights_eligibility_status'], passed)
                    self.assertTrue(review['selected_text_rights_reason'])
                    self.assertEqual(field['selected_text_rights_reason'], review['selected_text_rights_reason'])
                    if location['frozen_row'] == 2894:
                        self.assertEqual(field['quality_admission_status'], 'unresolved_everyday_eligibility')
                        self.assertEqual(review['accepted_text_quality_status'], 'unresolved_everyday_eligibility')
                        self.assertIsNone(field['accepted_text'])
                        self.assertIsNone(review['accepted_text'])

    def test_actual_import_boundary_and_retained_policy_material(self):
        contexts = read("source-context.json")["current_content"]
        signature = next(c for c in contexts if c["title"] == "signature")
        marker = next(m for m in signature["external_reference_markers"] if "Import:CFC" in m["raw_reference"])
        self.assertIn('unselected', marker['selected_boundary'])
        self.assertIn('3156847', marker['selected_boundary'])
        self.assertIn('24952889', marker['selected_boundary'])
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

    def test_each_use_spelling_and_replacement_has_an_independent_decision(self):
        review = read('field-review.json')
        groups = {g['frozen_row']: g for g in read('groups.json')}
        self.assertEqual(len(review['usage']), 50)
        self.assertEqual(len(review['spelling_relations']), 50)
        self.assertEqual([(r['frozen_row'], r['field']) for r in review['unresolved_fields']],
                         [(2894, name) for name in LANGUAGES])
        for use in review['usage']:
            row = use['frozen_row']
            self.assertTrue(use['review_reason'])
            self.assertEqual(use['reviewer'], review['reviewer'])
            self.assertEqual(use['everyday_sense_status'], 'unresolved' if row == 2894 else 'passed')
            self.assertEqual(use['accepted_usage'], None if row == 2894 else groups[row]['senses'][0]['use'])
        for relation in review['spelling_relations']:
            self.assertEqual(relation['independent_review'], 'passed')
            self.assertEqual(relation['relation'], 'exact_french_source_spelling')
            self.assertEqual(relation['alias_admission'], 'not_admitted')
            self.assertTrue(relation['review_reason'])
        authored = {(g['frozen_row'], name): f['text']
                    for g in read('authored/at-81f6aa9/groups.json')
                    for name, f in g['senses'][0]['fields'].items()}
        actual_changes = {(row, name) for row, g in groups.items()
                          for name, f in g['senses'][0]['fields'].items()
                          if f['text'] != authored[row, name]}
        replacements = read('accepted-replacements.json')
        self.assertEqual(len(replacements), 9)
        self.assertEqual({(r['frozen_row'], r['field']) for r in replacements}, actual_changes)
        for r in replacements:
            self.assertEqual(r['proposed_text'], authored[r['frozen_row'], r['field']])
            field = groups[r['frozen_row']]['senses'][0]['fields'][r['field']]
            self.assertEqual(r['accepted_text'], field['text'])
            self.assertEqual(r['exact_source']['reviewed_excerpt'], field['source_span']['raw_source_excerpt'])
            self.assertTrue(r['reason'])

    def test_own_fixed_capture_bytes_and_observed_original_work_boundaries(self):
        audit = read('source-audit.json')
        self.assertEqual((audit['source_count'], audit['fixed_capture_count'], audit['oldest_capture_count']), (50, 50, 50))
        self.assertEqual(audit['missing_or_unresolved_source_boundaries'], [])
        captures = {c['source_id']: c for c in read('source-captures.json')['sources']}
        contexts = {c['id']: c for c in read('source-context.json')['current_content']}
        responses = [json.loads(review_path(f'sources/fixed-{i}.api.json').read_bytes()) for i in range(1, 6)]
        revisions = {r['revid']: (p, r) for response in responses for p in response['query']['pages'] for r in p['revisions']}
        for source in audit['sources']:
            page, revision = revisions[source['selected_revid']]
            self.assertEqual((page['pageid'], page['title']), (source['pageid'], source['title']))
            capture = captures[source['source_id']]
            fixed_bytes = review_path('sources/' + Path(source['fixed_capture_raw_path']).name).read_bytes()
            self.assertEqual(fixed_bytes, revision['slots']['main']['content'].encode('utf-8'))
            self.assertEqual(fixed_bytes, retained_path(capture['raw_path']).read_bytes())
            oldest = source['oldest_revision']
            response = json.loads(review_path('sources/' + Path(oldest['api_path']).name).read_bytes())
            old_page = response['query']['pages'][0]
            old_revision = old_page['revisions'][0]
            self.assertEqual((old_page['pageid'], old_page['title']), (source['pageid'], source['title']))
            self.assertEqual((old_revision['revid'], old_revision['parentid']), (oldest['revid'], 0))
            self.assertEqual(review_path('sources/' + Path(oldest['raw_path']).name).read_bytes(),
                             old_revision['slots']['main']['content'].encode('utf-8'))
            self.assertTrue(source['selected_source_rights_reason'])
            self.assertEqual(source['selected_locations'], contexts[source['source_id']]['selected_locations'])
        for row in (2865, 2873, 2876):
            source = next(s for s in audit['sources'] if s['frozen_row'] == row)
            self.assertTrue(any('Littré' in m['raw_source_excerpt'] for m in source['oldest_import_markers']))
            layers = contexts[source['source_id']]['original_historical_dictionary_layers']
            lit = next(x for x in layers if 'Littré' in x['original_authors'])
            self.assertEqual(lit['conservative_expiry'], '1952-01-01')
            self.assertEqual(lit['independent_review'], 'passed_at_original_work_boundary')
        signature = contexts['b138_fr_33906']
        self.assertTrue(any('Académie' in x['original_authors'] for x in signature['original_historical_dictionary_layers']))
        for name in ('signature-daf-fusion', 'signature-boundaries'):
            response = json.loads(review_path(f'sources/{name}.api.json').read_bytes())
            for page in response['query']['pages']:
                self.assertEqual(page['pageid'], 33906)
                for revision in page['revisions']:
                    self.assertEqual(review_path(f"sources/signature-{revision['revid']}.wiki").read_bytes(),
                                     revision['slots']['main']['content'].encode('utf-8'))
        history = json.loads(review_path('sources/signature-history-1.api.json').read_bytes())
        self.assertNotIn('continue', history)
        self.assertEqual(len(history['query']['pages'][0]['revisions']), audit['signature_history']['revision_count'])
        before = review_path('sources/signature-3060169.wiki').read_text(encoding='utf-8')
        fusion = review_path('sources/signature-3156847.wiki').read_text(encoding='utf-8').splitlines()
        self.assertNotIn('Nom d’une personne', before)
        self.assertIn('Nom d’une personne', fusion[9])
        self.assertIn('fusiondaf', fusion[0])
        for before_id, after_id in ((24719744, 24952889), (25813538, 25814756)):
            before_lines = review_path(f'sources/signature-{before_id}.wiki').read_text(encoding='utf-8').splitlines()
            after_lines = review_path(f'sources/signature-{after_id}.wiki').read_text(encoding='utf-8').splitlines()
            self.assertEqual(before_lines[7:9], after_lines[7:9])
        imported = review_path('sources/signature-24952889.wiki').read_text(encoding='utf-8').splitlines()
        self.assertIn('Import:CFC', imported[32])
        self.assertIn('cartographie', imported[32])

    def test_halte_noun_evidence_cannot_admit_the_frozen_interjection(self):
        group = next(g for g in read('groups.json') if g['frozen_row'] == 2894)
        self.assertEqual((group['headword'], group['lexique_cgram']), ('halte', 'ONO'))
        self.assertEqual(group['senses'][0]['independent_review'], 'unresolved_everyday_eligibility')
        uses = next(u for u in read('field-review.json')['usage'] if u['frozen_row'] == 2894)
        self.assertIn('feminine noun', uses['review_reason'])
        self.assertIn('No exception', uses['review_reason'])


if __name__ == "__main__":
    unittest.main()
