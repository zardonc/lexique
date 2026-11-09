import { useEffect, useRef, useState } from 'react';
import { dictionaryFailure } from './platform/dictionary';
import type { DictionaryCatalog, DictionaryLookup, DictionaryFailure } from './platform/dictionary';
import type { RuntimeAdapter } from './platform/runtime';
import type { DictionaryView } from './conjugation-navigation';

interface Visit { readonly result?: DictionaryLookup | null; readonly error?: DictionaryFailure }
const builtin = [{ id: 'builtin', title: 'FreeDict fra-zho' }];

function initialVisits(initial?: DictionaryView): Readonly<Record<string, Visit>> {
  if (!initial) return {};
  return { ...Object.fromEntries((initial.result.warnings ?? []).map((warning) => [warning.dictionary_id, { error: dictionaryFailure({ code: warning.code, message: warning.message }) }])),
    [initial.result.dictionary_id ?? 'builtin']: { result: initial.result } };
}

async function read(runtime: RuntimeAdapter, query: string, id: string): Promise<Visit> {
  try { return { result: await runtime.lookupDictionary(query, id) }; }
  catch (cause) { return { error: dictionaryFailure(cause) }; }
}

function startingQuery(initial?: DictionaryView) { return initial?.result.entries.find((entry) => entry.ordinal === initial.ordinal)?.key ?? ''; }
function startingDictionary(initial?: DictionaryView) { return initial?.result.dictionary_id ?? 'builtin'; }

export function useDictionaryTabs(runtime: RuntimeAdapter, initial?: DictionaryView) {
  const [catalog, setCatalog] = useState<DictionaryCatalog | null>();
  const [catalogError, setCatalogError] = useState<DictionaryFailure>();
  const [retry, setRetry] = useState(0);
  const [query, setQuery] = useState(() => startingQuery(initial));
  const [selected, setSelected] = useState(() => startingDictionary(initial));
  const [visits, setVisits] = useState(initialVisits(initial));
  const [loading, setLoading] = useState(false);
  const identity = useRef(0);
  useEffect(() => {
    let cancelled = false;
    setCatalog(undefined); setCatalogError(undefined);
    void runtime.dictionarySettings().then((value) => { if (!cancelled) setCatalog(value); })
      .catch((cause: unknown) => { if (!cancelled) setCatalogError(dictionaryFailure(cause)); });
    return () => { cancelled = true; identity.current++; };
  }, [runtime, retry]);
  async function search(key: string, id?: string) {
    const request = ++identity.current, items = id === undefined ? catalog ?? builtin : (catalog ?? builtin).filter((item) => item.id === id);
    if (!items.length) return;
    setQuery(key); setVisits({}); setLoading(true); setSelected(items[0].id);
    for (const item of items) {
      const visit = await read(runtime, key, item.id);
      if (request !== identity.current) return;
      setVisits((values) => ({ ...values, [item.id]: visit }));
      if (!visit.error) { setSelected(item.id); break; }
    }
    if (request === identity.current) setLoading(false);
  }
  async function select(id: string, force = false) {
    const request = ++identity.current;
    setSelected(id);
    if (visits[id] && !force) { setLoading(false); return; }
    setLoading(true);
    const visit = await read(runtime, query, id);
    if (request !== identity.current) return;
    setVisits((values) => ({ ...values, [id]: visit })); setLoading(false);
  }
  return { catalog: catalog ?? builtin, catalogLoading: catalog === undefined && !catalogError, catalogError, query, selected, visits,
    result: loading ? undefined : visits[selected]?.result, error: loading ? undefined : visits[selected]?.error, loading, search, select,
    reload: () => setRetry((value) => value + 1), cancel: () => { identity.current++; setLoading(false); } };
}
