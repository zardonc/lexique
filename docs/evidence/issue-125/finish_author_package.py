"""Attach selected rights proposals and notices to #125 fixed-source candidates."""
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[3]
CAPTURES = ROOT / 'target/issue-125'
EVIDENCE = ROOT / 'docs/evidence/issue-125'


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding='utf-8'))


def write(name, value):
    (EVIDENCE / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')


sources = read('sources.json')
contexts = read('source-context.json')
captures = {item['title']: item for item in read('source-captures.json')['sources']}
rights_sources = []
for source, context in zip(sources['sources'], contexts['current_content'], strict=True):
    assert source['id'] == context['id']
    lines = (ROOT / captures[source['title']]['raw_path']).read_text(encoding='utf-8').splitlines()
    markers = [dict(line_1_based=n, raw_reference=line,
                    role='import_or_bibliographic_identity_not_selected_sentence_authorship')
               for n, line in enumerate(lines, 1)
               if re.search(r'\{\{(?:R[:|]|Import:)', line) and not line.startswith('#*')]
    imports = [marker for marker in markers if '{{Import:' in marker['raw_reference']]
    assert all('{{Import:DAF8' in marker['raw_reference'] or
               '{{Import:Littré' in marker['raw_reference'] for marker in imports), source['title']
    layers = source['original_historical_dictionary_layers']
    if any('{{Import:Littré' in marker['raw_reference'] for marker in imports):
        assert source['title'] == 'rang'
        layers.append(dict(original_authors='Émile Littré',
                           edition='Dictionnaire de la langue française, 1872–1877',
                           author_proposed_original_rights='public_domain_at_original_historical_work_boundary',
                           established_project_evidence='docs/evidence/issue-100/source-context.json: étranger original-littre-1872-1877',
                           source_notice_evidence='Modèle:Import:Littré revision 30857967; retained rights/import-littre.wiki',
                           policy_evidence='Convention:Droit d’auteur revision 38903168 explicitly permits public-domain Littré originals',
                           scope='Conservative original historical edition credit at the article import boundary; no selected-sentence authorship inferred.',
                           wiki_layer='Fixed contributor transcription/adaptation, CC BY-SA 4.0.',
                           lexique_layer='Scoped French adaptation and new EN/ZH explanations, CC BY-SA 4.0.',
                           independent_review='pending'))
        source['attribution'] += ' Original historical dictionary layer where imported: Émile Littré, 1872–1877.'
        context['attribution'] = source['attribution']
    context['original_historical_dictionary_layers'] = layers
    context['external_reference_markers'] = markers
    source['external_reference_markers'] = markers
    context['all_import_notices'] = imports
    context['reference_boundary'] = 'Bibliography does not grant rights in reference prose. Selected lexical/IPA/form spans exclude literary and news examples, etymologies, media, translation tables and other-language entries.'
    selected = context['selected_locations'] + context['selected_grammar_and_construction_evidence']
    for location in selected:
        raw = location.get('raw_source_excerpt', location.get('reviewed_excerpt'))
        assert raw == lines[location['line_1_based'] - 1]
        assert not raw.lstrip().startswith('#*')
        assert '<ref' not in raw and 'source=' not in raw, (source['title'], location)
    rights_sources.append(dict(source_id=source['id'], title=source['title'], revid=source['revid'],
                              fixed_url=source['fixed_url'], history_url=source['history_url'],
                              attribution=source['attribution'], current_registry_names=context['current_registry_names'],
                              current_contributors=context['current_contributors'],
                              anonymous_contributors_count=context['anonymous_contributors_count'],
                              anonymous_attribution=context['anonymous_attribution'],
                              original_historical_dictionary_layers=layers, import_notices=imports,
                              reference_markers=markers, selected_locations=selected,
                              independent_review='pending', reviewer=None))
write('sources.json', sources)
write('source-context.json', contexts)

by_id = {source['id']: source for source in sources['sources']}
fields = []
for group in read('groups.json'):
    for sense in group['senses']:
        for name, field in sense['fields'].items():
            source = by_id[field['source_id']]
            original = ', '.join(layer['original_authors'] for layer in source['original_historical_dictionary_layers'])
            boundary = ('One recorded isolated-headword IPA variant at its exact template substring; no media or liaison imported.'
                        if name == 'ipa' else
                        'Selected lexical definition adapted with the stated use restrictions; EN/ZH are new explanations of that same French sense, not imported bilingual text.')
            if original:
                boundary += ' Retain original historical credit to ' + original + ' where imported; marker alone does not assign sentence authorship.'
            if group['frozen_row'] == 2240:
                boundary += ' Fixed cents adjective plural line 6 and cent cardinal/number notes supply the exact form relationship; currency-noun pronunciation is excluded.'
            fields.append(dict(frozen_row=group['frozen_row'], headword=group['headword'],
                               sense_id=sense['sense_id'], field=name, language=field['language'],
                               proposed_text=field['text'], source_id=source['id'], revid=source['revid'],
                               fixed_url=source['fixed_url'], history_url=source['history_url'],
                               exact_source_span=field['source_span'], modification=field['modification'],
                               selected_use=sense['use'], selected_rights_boundary=boundary,
                               proposed_license='CC-BY-SA-4.0',
                               author_rights_assessment='eligible_proposal_at_selected_boundary',
                               independent_review='pending', reviewer=None))
assert len(fields) == 200
write('rights.json', dict(status='author_proposal_pending_separate_selected_span_rights_review',
                         independent_review='pending', reviewer=None, proposed_adaptation_license='CC-BY-SA-4.0',
                         complete_legal_text='docs/evidence/issue-125/COPYING',
                         authority_captures=json.loads((CAPTURES / 'rights-authorities.json').read_text(encoding='utf-8')),
                         original_daf8_basis='Académie française collective edition 1932–1935; CPI L123-3 seventy-year term from following January; conservative expiry 2006-01-01. Reused established source record, no live statute refresh.',
                         original_littre_basis='Established #100 original 1872–1877 work basis and retained fixed import/policy templates; separate from contributor adaptation and new translations.',
                         sources=rights_sources, fields=fields,
                         redistribution_obligations=[
                             'Retain fixed article URLs, complete linked histories, named contributor registries and anonymous history credit.',
                             'Credit original historical DAF8 and Littré layers where imported, preserving their separate original-work basis.',
                             'Identify CC BY-SA 4.0, include its complete legal text, state changes and license adaptations under the same terms.',
                             'Apply ShareAlike and database obligations to the actual assembled adapted data; add no downstream restrictions.',
                             'Keep third-party resources distinct from the MIT application code and the retained GPL FreeDict supplement.',
                             'Parent verifies the actual combined converter/package notices and source-material delivery after independent review.',
                         ], packaging_obligations_status='parent_owned_pending_actual_combined_artifact'))

notices = ['# Issue #125 selected-content notices', '',
           'Proposed selected content and Lexique adaptations/explanations: CC BY-SA 4.0.',
           'Wiktionnaire contributors are credited through the fixed articles and complete histories below.',
           'Named contributor registries and anonymous counts are retained in source-context.json.',
           'Registries may include contributors after the selected revision and do not identify sentence authorship.', '',
           'Original historical layers where imported: Académie française, Dictionnaire de l’Académie française,',
           'eighth edition (1932–1935); Émile Littré, Dictionnaire de la langue française (1872–1877).',
           'Original public-domain works remain separate from contributor adaptations.', '',
           'Changes: selection of one everyday use per frozen group, scoped French adaptations, markup removal,',
           'new English/Chinese explanations and one isolated IPA variant with delimiters.',
           'The numeral cents retains exact plural/form restrictions; no runtime spelling alias is proposed.', '',
           'Licence: https://creativecommons.org/licenses/by-sa/4.0/legalcode.en; full text in COPYING.',
           'Frozen membership: Boris New and Christophe Pallier, Lexique 3.83, CC BY-SA 4.0.',
           'https://chrplr.github.io/openlexicon/datasets-info/Lexique383/README-Lexique.html', '',
           '| Article | Fixed revision | Complete history |', '| --- | --- | --- |']
for source in sources['sources']:
    notices.append(f"| {source['title']} | [revision {source['revid']}]({source['fixed_url']}) | [contributors]({source['history_url']}) |")
(EVIDENCE / 'NOTICES.md').write_text('\n'.join(notices) + '\n', encoding='utf-8', newline='\n')
admission = read('admission.json')
admission.update(pending_independent_review_fields=200, pending_independent_rights_fields=200,
                 author_rights_assessment='completed_proposal_pending_independent_selected_span_review',
                 packaging_obligations_status='parent_owned_pending_actual_combined_artifact',
                 source_reproduction='see retained reproduction.log; author check only',
                 pending_parent_gates=['Independent field/use/spelling/rights review', 'Canonical conversion and combined notices',
                                       'Actual native lookup/save/reopen', 'Learner acceptance', 'Formal release readiness'])
write('admission.json', admission)
print(json.dumps(dict(fields=200, sources=len(sources['sources']), rights_proposals_complete=True,
                      independent_acceptance=False, canonical_admission=False)))
