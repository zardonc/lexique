import type { NotebookText } from './notebook-content';
import type { VocabularySnapshot } from './platform/vocabulary';

export function VocabularySource({ snapshot, text }: { readonly snapshot: VocabularySnapshot; readonly text: NotebookText }) {
  const source = snapshot.reference;
  return <footer className="dictionary-source">
    {snapshot.import_source && <p>{text.interchange.source}: {snapshot.import_source.filename} ({snapshot.import_source.format})</p>}
    {source ? <><h4>{text.source}</h4><p>{source.dictionary} · {source.version}</p>
      <p>{source.attribution ?? text.missing} · {source.license ?? text.missing}</p>
      <p>{source.source_url ?? text.missing}</p><p>{source.license_url ?? text.missing}</p></>
      : <p role="status">{text.interchange.missing}</p>}
  </footer>;
}
