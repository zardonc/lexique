"""Prepare #125 candidate evidence from retained fixed first-party captures."""
import csv
from datetime import datetime, timezone
import json
from pathlib import Path
import re

from selection import SELECTION, SUPPORT, FLEXION

ROOT = Path(__file__).resolve().parents[3]
CAPTURES = ROOT / 'target/issue-125'
OUT = ROOT / 'docs/evidence/issue-125'
OUT.mkdir(parents=True, exist_ok=True)
LANGUAGES = {'french_definition': 'fr', 'ipa': 'und', 'english': 'en', 'chinese': 'zh'}


def write(name, value):
    (OUT / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')


with (ROOT / 'resources/core-pilot-79/scope-manifest.csv').open(encoding='utf-8-sig', newline='') as handle:
    scope = list(csv.DictReader(handle))
index = json.loads((CAPTURES / 'source-index.json').read_text(encoding='utf-8'))

for item in index:
    item['source_id'] = f"b125_fr_{item['pageid']}"
sources, contexts, lines = {}, {}, {}
contributors = {}
anonymous_contributors = {}
parts = sorted(path for path in (CAPTURES / 'sources').glob('contributors*.api.json')
               if not path.name.startswith('contributors-en-'))
assert parts
for prefix in ('contributors-', 'contributors-cent-'):
    numbered = [path for path in parts if re.fullmatch(re.escape(prefix) + r'\d+\.api\.json', path.name)]
    assert numbered and not json.loads(numbered[-1].read_text(encoding='utf-8')).get('continue')
for path in parts:
    data = json.loads(path.read_text(encoding='utf-8'))
    for page in data['query']['pages']:
        contributors.setdefault(page['title'], set()).update(person['name'] for person in page.get('contributors', []))
        if 'anoncontributors' in page:
            anonymous_contributors[page['title']] = page['anoncontributors']

for capture in index:
    title = capture['title']
    raw = json.loads((ROOT / capture['api_path']).read_text(encoding='utf-8-sig'))
    page = next(page for page in raw['query']['pages'] if page['title'] == title)
    revision = next(rev for rev in page['revisions'] if rev['revid'] == capture['revid'])
    body = revision['slots']['main']['content']
    assert body == (ROOT / capture['raw_path']).read_text(encoding='utf-8'), title
    assert revision['timestamp'] == capture['timestamp']
    lines[title] = body.splitlines()
    source_id = f"b125_fr_{capture['pageid']}"
    notices = [dict(line_1_based=number, raw_marker=line,
                    declared_source='Académie française, Dictionnaire de l’Académie française, eighth edition (1932–1935).',
                    selected_sentence_authorship='pending_separate_selected_span_review')
               for number, line in enumerate(lines[title], 1) if '{{Import:DAF8' in line]
    original_layers = []
    if notices:
        original_layers.append(dict(
            original_authors='Académie française', edition='Dictionnaire de l’Académie française, eighth edition, 1932–1935',
            author_proposed_original_rights='public_domain_at_original_collective_work_boundary',
            legal_basis='Code de la propriété intellectuelle, article L123-3: seventy years from 1 January following publication of a collective work.',
            legal_basis_url='https://www.legifrance.gouv.fr/loda/article_lc/LEGIARTI000006278941/2026-02-20',
            conservative_expiry='2006-01-01, using final publication in 1935',
            established_project_evidence='docs/evidence/issue-99/source-context.json: current_content[].original_authors_and_rights',
            scope='Original edition where imported/referenced; page import markers alone do not identify the author of a selected sentence.',
            wiki_layer='Fixed Wiktionnaire contributor transcription/adaptation, CC BY-SA 4.0.',
            lexique_layer='Scoped definition adaptations and new explanatory EN/ZH translations, Lexique contributors, CC BY-SA 4.0.',
            independent_review='pending'))
    bibliography = []
    for number, line in enumerate(lines[title], 1):
        if 'source=' in line:
            bibliography.append(dict(line_1_based=number, bibliographic_marker=line.split('source=', 1)[1],
                                     role='bibliographic_identity_only_no_quoted_example_prose'))
    attribution = 'Wiktionnaire contributors (full linked article history); Lexique contributors for selection, scoped adaptations and new explanatory translations.'
    if notices:
        attribution += ' Original historical dictionary layer where imported: Académie française, eighth edition.'
    sources[title] = dict(id=source_id, edition='fr', entry_language='fr', title=title,
        pageid=capture['pageid'], revid=capture['revid'], timestamp=capture['timestamp'],
        fixed_url=capture['fixed_url'], history_url=capture['history_url'], attribution=attribution,
        content_license='CC-BY-SA-4.0', license_url='https://creativecommons.org/licenses/by-sa/4.0/legalcode.en',
        source_frame='Actual revision returned by the retained first-party API capture.',
        selected_text_review='pending_independent_selected_span_review',
        original_historical_dictionary_layers=original_layers,
        modification='Lexique selects and adapts French semantics, supplies new English/Chinese explanations, and selects one isolated IPA variant; no attributed literary/news quotation, media or whole page redistributed. Source bindings retain frozen inflected forms and the selected construction.')
    contexts[title] = dict(id=source_id, title=title, pageid=capture['pageid'], current_content_revid=capture['revid'],
        timestamp=capture['timestamp'], fixed_url=capture['fixed_url'], history_url=capture['history_url'],
        content_license='CC-BY-SA-4.0', license_url=sources[title]['license_url'], attribution=attribution,
        status='author_fixed_span_validation_passed_independent_review_pending', selected_locations=[],
        selected_grammar_and_construction_evidence=[], import_notices=notices,
        original_historical_dictionary_layers=original_layers, bibliographic_identities=bibliography,
        contributor_capture_parts=[str(path.relative_to(ROOT)) for path in parts],
        current_registry_names=len(contributors[title]), current_contributors=sorted(contributors[title]),
        anonymous_contributors_count=anonymous_contributors.get(title, 0),
        anonymous_attribution='Anonymous identities are credited through the linked complete article history; the contributors API returns their count rather than individual names.',
        attribution_boundary='Complete paginated current named registry, returned anonymous count and full history credit; registry extends beyond the fixed revision and does not establish selected-sentence authorship.',
        selected_text_rights_status='pending_independent_selected_text_and_import_boundary_audit')


def binding(title, number):
    source = sources[title]
    return dict(source_id=source['id'], revid=source['revid'], url=source['fixed_url'], history_url=source['history_url'],
                line_1_based=number, column_1_based=1, reviewed_excerpt=lines[title][number - 1])


def frame(title, number):
    before = lines[title][:number]
    language = max(n for n, line in enumerate(before, 1) if line.startswith('== '))
    assert re.fullmatch(r'==\s*\{\{langue\|fr\}\}\s*==', before[language - 1]), (title, number)
    pos = max(n for n, line in enumerate(before, 1) if line.startswith('=== {{S|')
              and not any(tag in line for tag in ('étymologie', 'prononciation', 'références')))
    return [binding(title, language), binding(title, pos)]


groups, excerpts, reviews, usages, relations, supporting = [], [], [], [], [], []
for row, definition_title, definition_line, ipa_title, ipa_line, ipa, french, english, chinese, use, excluded in SELECTION:
    frozen = scope[row - 1]
    headword, pos = frozen['word'], frozen['lexique_cgram']
    identity = f"b125-{row}-{headword}-{pos.lower().replace(':', '-')}-selected"
    assert lines[definition_title][definition_line - 1].startswith('# '), row
    ipa_raw = lines[ipa_title][ipa_line - 1]
    assert ipa.strip('/') in ipa_raw, (row, ipa)
    definition_frame = frame(definition_title, definition_line)
    ipa_frame = frame(ipa_title, ipa_line)
    expected_pos = {'VER': 'verbe', 'NOM': 'nom', 'ADV': 'adverbe', 'ADJ': 'adjectif',
                    'PRO:per': 'pronom personnel', 'PRO:ind': 'pronom indéfini', 'ADJ:int': 'adjectif interrogatif', 'ADJ:num': 'adjectif numéral', 'PRO:pos': 'pronom possessif', 'PRO:dem': 'pronom démonstratif', 'PRE': 'préposition', 'ONO': 'interjection'}[pos]
    assert '{{S|' + expected_pos + '|' in definition_frame[1]['reviewed_excerpt'], (row, definition_frame)
    grammar = []
    for title, number in ((definition_title, definition_line), (ipa_title, ipa_line)):
        pos_line = frame(title, number)[1]['line_1_based']
        for n in range(pos_line + 1, number):
            line = lines[title][n - 1]
            if line.startswith(('{{fr-', "'''")):
                grammar.append(binding(title, n))
    source_support = definition_frame + ipa_frame + grammar + [binding(title, number) for title, number in SUPPORT.get(row, [])]
    source_support = list({(item['source_id'], item['line_1_based']): item for item in source_support}.values())
    for record in source_support:
        # Selected tracked support contains grammar, not literary example bodies.
        assert not record['reviewed_excerpt'].lstrip().startswith('#*'), (row, record)
        supporting.append(dict(sense_id=identity, frozen_row=row, role='language_pos_spelling_or_construction_frame', **record))
        source_title = next(title for title, source in sources.items() if source['id'] == record['source_id'])
        contexts[source_title]['selected_grammar_and_construction_evidence'].append(dict(sense_id=identity, **record))
    fields = {}
    for name, value in zip(LANGUAGES, (french, ipa, english, chinese), strict=True):
        exact = binding(ipa_title, ipa_line) if name == 'ipa' else binding(definition_title, definition_line)
        modification = ('One recorded isolated-headword IPA variant selected; slashes added; no liaison, audio or media copied.' if name == 'ipa'
            else 'Lexique scoped adaptation of selected French semantics; markup removed; selected use and restrictions retained.' if name == 'french_definition'
            else 'New Lexique explanatory translation of the selected French use, not an upstream bilingual quotation.')
        if row == 2240:
            modification += ' Applies to the exact frozen form through separate spelling or grammatical evidence; source binding only.'
        if name == 'ipa':
            exact['ipa_variant_column_1_based'] = ipa_raw.index(ipa.strip('/')) + 1
            exact['ipa_variant_excerpt'] = ipa.strip('/')
        fields[name] = dict(language=LANGUAGES[name], text=value, source_id=exact['source_id'],
            source_span=dict(line_1_based=exact['line_1_based'], column_1_based=1, raw_source_excerpt=exact['reviewed_excerpt']),
            content_license='CC-BY-SA-4.0', modification=modification,
            author_validation='passed_fixed_source_span_and_scoped_candidate_check', independent_review='pending',
            rights_eligibility_status='pending_separate_selected_span_audit')
        excerpts.append(dict(sense_id=identity, field=name, **exact))
        reviews.append(dict(sense_id=identity, frozen_row=row, headword=headword, pos=pos, field=name,
            language=LANGUAGES[name], proposed_text=value, exact_source=exact, supporting_sources=source_support,
            author_validation='passed_fixed_capture_binding_check', accepted_text_quality_status='pending',
            rights_eligibility_status='pending_separate_selected_span_audit', reviewer=None))
        source_title = ipa_title if name == 'ipa' else definition_title
        contexts[source_title]['selected_locations'].append(dict(sense_id=identity, frozen_row=row, field=name,
            line_1_based=exact['line_1_based'], column_1_based=1, raw_source_excerpt=exact['reviewed_excerpt']))
    sense = dict(sense_id=identity, source_headword=definition_title, use=use, use_language='en', excluded_uses=excluded,
        independent_review='pending', resource_admission='pending_separate_independent_review', cefr=None, fields=fields)
    groups.append(dict(frozen_row=row, headword=headword, lexique_cgram=pos,
        subtitle_lemma_per_million=frozen['subtitle_lemma_per_million'], subtitle_pos_per_million=frozen['subtitle_pos_per_million'], senses=[sense]))
    usages.append(dict(sense_id=identity, frozen_row=row, headword=headword, pos=pos, source_headword=definition_title,
        proposed_usage=use, language='en', excluded_uses=excluded, exact_source=binding(definition_title, definition_line),
        supporting_sources=source_support, construction_and_flexion_evidence=source_support,
        author_construction_and_flexion_interpretation=FLEXION[row],
        author_everyday_use_assessment='Modern selected sense with register, construction and region retained; unrelated technical, obsolete and grammatical alternatives are excluded.',
        accepted_usage_quality_status='pending', everyday_sense_status='pending_separate_review', reviewer=None))
    relation = dict(sense_id=identity, frozen_row=row, frozen_headword=headword, source_headword=definition_title,
        relation='plural_cardinal_numeral_inflection' if row == 2240 else 'exact_french_source_spelling',
        relation_role='source_binding_only', alias_admission='not_admitted',
        proposed_evidence=source_support, construction_and_flexion_interpretation=FLEXION[row], independent_review='pending')
    relations.append(relation)

assert [group['frozen_row'] for group in groups] == list(range(2201, 2251))
curated = json.loads((ROOT / 'resources/core-pilot-79/curated-inputs.json').read_text(encoding='utf-8'))
assert not {g['frozen_row'] for g in groups} & {g['frozen_row'] for g in curated['groups']}
assert len(excerpts) == len(reviews) == 200 and len(usages) == len(relations) == 50
assert len(sources) == len(index) == 51
write('groups.json', groups)
write('sources.json', dict(sources=list(sources.values())))
write('source-excerpts.json', dict(fields=excerpts, supporting_evidence=supporting))
write('source-context.json', dict(current_content=list(contexts.values()),
    rights_review='Author-proposed CC BY-SA 4.0 contribution/adaptation route and separately attributed DAF8 original-work basis; independent selected-span rights review remains pending.',
    spelling_relations=relations))
write('field-review.json', dict(schema='lexique-d2-field-review-draft-1', kind='author_prepared_for_separate_reviewer',
    independence='Content author checks are not independent quality or rights review.', independent_acceptance=False,
    reviewer=None, fields=reviews, usage=usages, spelling_relations=relations, unresolved_fields=[]))
write('reuse.json', dict(comparison_input='resources/core-pilot-79/curated-inputs.json', canonical_content_version=curated['content_version'], existing_admitted_groups=curated['selected_groups'], reused_groups=0, reused_fields=0, groups=[], note='No exact assigned frozen identity is already in the inspected canonical input.'))
write('source-captures.json', dict(sources=index,
    contributor_registry=dict(parts=[str(path.relative_to(ROOT)) for path in parts], pagination_complete=True,
        names=sum(len(value) for value in contributors.values()), titles=len(contributors),
        anonymous_counts_by_title=anonymous_contributors,
        selected_sentence_authorship_proved=False),
    capture_root=str(CAPTURES), captured_at_utc=datetime.fromtimestamp((CAPTURES / 'sources/batch-1.api.json').stat().st_mtime, timezone.utc).isoformat(),
    preparation_script=str(Path(__file__).resolve()), selection_file=str(Path(__file__).resolve().with_name('selection.py'))))
write('admission.json', dict(ticket=125, parent=78, schema='lexique-d2-content-candidate-handoff-1',
    status='authored_candidates_pending_separate_independent_review', scope_denominator_groups=4087,
    frozen_rows=list(range(2201, 2251)), frozen_groups=50, proposed_groups=50, proposed_senses=50, proposed_fields=200,
    source_count=len(sources), independently_accepted_new_groups=0, independently_accepted_new_fields=0,
    independent_reviewer=None, resource_admission='pending_separate_independent_review', unresolved_field_blockers=[],
    reused_groups=0, reused_fields=0, pending_independent_review_rows=list(range(2201, 2251)),
    independent_quality='pending', selected_span_rights='pending', construction_and_spelling_review='pending',
    final_output_validation='author_resource_boundary_check_only',
    native_checks='integration_owned_not_run', conversion_and_packaging='integration_owned_not_run',
    author_evidence_root=str(CAPTURES), proposed_field_characters=sum(len(item['proposed_text']) for item in reviews),
    longest_field_characters=max(len(item['proposed_text']) for item in reviews),
    imported_original_boundary_sources=[source['id'] for source in sources.values() if source['original_historical_dictionary_layers']]))
print(json.dumps(dict(frozen_groups=50, proposed_fields=200, fixed_sources=len(sources),
    selected_spans_validated=200, independent_acceptance=False,
    imported_original_boundary_sources=sum(bool(source['original_historical_dictionary_layers']) for source in sources.values()),
    evidence_root=str(CAPTURES)), ensure_ascii=False))
