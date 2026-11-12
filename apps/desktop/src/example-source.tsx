import { useState } from 'react';
import { RecordOriginal } from './dictionary';
import { messages } from './content';
import type { NotebookText } from './notebook-content';
import type { RuntimeAdapter } from './platform/runtime';
import type { VocabularyRecord } from './platform/vocabulary';

export function ExampleSource({ source, text, runtime }: {
  readonly source: VocabularyRecord['example_source']; readonly text: NotebookText; readonly runtime: RuntimeAdapter;
}) {
  const [open, setOpen] = useState(false);
  const locale = text.frequency.locale.startsWith('zh') ? 'zh' : text.frequency.locale.startsWith('fr') ? 'fr' : 'en';
  if (!source) return <p>{text.unknownSource}</p>;
  return <aside className="dictionary-source"><p>{text.source}: {source.dictionary} · {source.version} · {source.lookup_key}</p>
    <p>{source.attribution ?? text.unknownSource} · {source.license ?? text.unknownSource}</p>
    <p>{source.source_url ?? text.unknownSource}</p><p>{source.license_url ?? text.unknownSource}</p>
    <button type="button" onClick={() => setOpen(!open)} aria-expanded={open}>{text.reopenExample}</button>
    {open && <RecordOriginal reference={source} runtime={runtime} text={messages[locale].dictionary} title={text.reopenExample} />}
  </aside>;
}
