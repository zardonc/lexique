import { useEffect, useState } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import type { InterfacePreferences } from './platform/interface';

const messages = {
  zh: { loading: '正在读取界面偏好…', saving: '正在保存界面偏好…', saved: '界面偏好已保存。',
    preview: '浏览器预览：语言和主题仅在本次预览中使用，不会保存。', failed: '界面偏好操作失败', retry: '重试界面偏好' },
  en: { loading: 'Loading interface preferences…', saving: 'Saving interface preferences…', saved: 'Interface preferences saved.',
    preview: 'Browser preview: language and theme apply only to this preview and are not saved.', failed: 'Interface preference operation failed', retry: 'Retry interface preferences' },
  fr: { loading: 'Chargement des préférences d’interface…', saving: 'Enregistrement des préférences d’interface…', saved: 'Préférences d’interface enregistrées.',
    preview: 'Aperçu web : la langue et le thème sont temporaires et ne sont pas enregistrés.', failed: 'Échec de l’opération sur les préférences d’interface', retry: 'Réessayer les préférences d’interface' },
} as const;

export function useInterfacePreferences(runtime: RuntimeAdapter) {
  const [value, setValue] = useState<InterfacePreferences>({ locale: 'zh', theme: 'white' });
  const [state, setState] = useState<'loading' | 'ready' | 'preview' | 'saving' | 'saved' | 'failed'>('loading');
  const [error, setError] = useState('');
  const [pending, setPending] = useState<InterfacePreferences | null>(null);
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let active = true;
    runtime.interfacePreferences().then((saved) => {
      if (!active) return;
      if (saved) setValue(saved);
      setState(saved ? 'ready' : 'preview');
    }).catch((reason: unknown) => { if (active) { setError(String(reason)); setState('failed'); } });
    return () => { active = false; };
  }, [runtime, attempt]);
  async function choose(next: InterfacePreferences) {
    setPending(next); setError(''); setState('saving');
    try {
      const saved = await runtime.interfacePreferences(next);
      setValue(saved ?? next); setPending(null); setState(saved ? 'saved' : 'preview');
    } catch (reason) { setError(String(reason)); setState('failed'); }
  }
  function retry() {
    if (pending) void choose(pending);
    else { setError(''); setState('loading'); setAttempt((current) => current + 1); }
  }
  const text = messages[value.locale];
  const busy = state === 'loading' || state === 'saving';
  return { value, choose, retry, error, text, busy,
    disabled: busy || (state === 'failed' && pending === null),
    message: state === 'ready' || state === 'failed' ? '' : text[state] };
}
