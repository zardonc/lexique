import { useEffect, useRef, useState } from 'react';
import type { Locale } from './content';
import type { RetainedCopy } from './platform/backup';
import type { RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';
import { restoreMessages } from './restore-content';

export function RetainedCopies({ runtime, copies, locale }: {
  readonly runtime: RuntimeAdapter; readonly copies: readonly RetainedCopy[]; readonly locale: Locale;
}) {
  const text = restoreMessages[locale], language = locale === 'zh' ? 'zh-CN' : locale;
  const [opening, setOpening] = useState(false), [error, setError] = useState('');
  const pending = useRef(false), live = useRef(true);
  useEffect(() => { live.current = true; return () => { live.current = false; }; }, []);
  async function open(copy: RetainedCopy) {
    if (pending.current) return;
    pending.current = true; setOpening(true); setError('');
    try { await runtime.openRetainedDirectory(copy.id); }
    catch (cause) { if (live.current) setError(errorMessage(cause)); }
    finally { pending.current = false; if (live.current) setOpening(false); }
  }
  if (!copies.length) return null;
  return <section className="retained-copies" aria-label={text.copies}><h3>{text.copies}</h3>
    {copies.map((copy) => <div key={copy.id}><h4>{text.kinds[copy.kind]}</h4><dl className="backup-result">
      <div><dt>{text.path}</dt><dd>{copy.path}</dd></div>
      <div><dt>{text.size}</dt><dd>{copy.byteLength.toLocaleString(language)} {text.bytes}</dd></div>
    </dl><button disabled={opening} onClick={() => { void open(copy); }}>{text.open}</button></div>)}
    {error && <p role="alert">{text.openFailed}: {error}</p>}
  </section>;
}
