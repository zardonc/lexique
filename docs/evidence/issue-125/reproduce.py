"""Rebuild #125 evidence from retained fixed captures and compare exact output bytes."""
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent
names = ('groups.json', 'sources.json', 'source-excerpts.json', 'source-context.json',
         'field-review.json', 'reuse.json', 'source-captures.json', 'admission.json', 'rights.json', 'NOTICES.md')
before = {name: (EVIDENCE / name).read_bytes() for name in names}
captures = json.loads(before['source-captures.json'])
for source in captures['sources']:
    response = json.loads((ROOT / source['api_path']).read_text(encoding='utf-8'))
    page = next(page for page in response['query']['pages'] if page['title'] == source['title'])
    revision = next(revision for revision in page['revisions'] if revision['revid'] == source['revid'])
    assert revision['timestamp'] == source['timestamp']
    assert revision['slots']['main']['content'].encode('utf-8') == (ROOT / source['raw_path']).read_bytes(), source['title']

for prefix, expected_titles in (('contributors-', 50), ('contributors-cent-', 1)):
    parts = []
    number = 1
    while (ROOT / f'target/issue-125/sources/{prefix}{number}.api.json').exists():
        parts.append(ROOT / f'target/issue-125/sources/{prefix}{number}.api.json')
        number += 1
    assert parts
    names_by_title, anonymous = {}, {}
    previous = None
    for path in parts:
        response = json.loads(path.read_text(encoding='utf-8'))
        assert 'error' not in response
        if previous:
            from urllib.parse import parse_qs, urlparse
            request = parse_qs(urlparse(path.with_suffix('.request.txt').read_text(encoding='utf-8')).query)
            assert all(request[key] == [str(value)] for key, value in previous.items())
        for page in response['query']['pages']:
            names_by_title.setdefault(page['title'], set()).update(person['name'] for person in page.get('contributors', []))
            if 'anoncontributors' in page:
                anonymous[page['title']] = page['anoncontributors']
        previous = response.get('continue')
    assert previous is None and len(names_by_title) == expected_titles
    contexts = json.loads(before['source-context.json'])['current_content']
    for title, nameset in names_by_title.items():
        context = next(context for context in contexts if context['title'] == title)
        assert context['current_contributors'] == sorted(nameset)
        assert context['anonymous_contributors_count'] == anonymous.get(title, 0)
    print(f'{prefix}: {len(parts)} complete parts, {expected_titles} titles')

for script in ('prepare_candidates.py', 'finish_author_package.py'):
    subprocess.run([sys.executable, '-B', '-X', 'utf8', str(EVIDENCE / script)], cwd=ROOT, check=True)
for name, content in before.items():
    assert (EVIDENCE / name).read_bytes() == content, name
    print(f'Identical: {name}')
print('51 fixed source wikitext files, full contributor pagination and 10 evidence outputs reproduced.')
