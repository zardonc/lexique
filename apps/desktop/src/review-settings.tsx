import { useEffect, useState } from 'react';
import type { Locale } from './content';
import type { ReviewSettings, RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';

const messages = {
  zh: { title: '复习设置', retention: '目标保持率 (%)', newLimit: '每日新卡上限', reviewLimit: '每日复习作答上限', save: '保存复习设置', saved: '已保存', loading: '正在读取设置', preview: '浏览器预览无法保存复习设置。', error: '复习设置失败', retry: '重新读取' },
  en: { title: 'Review settings', retention: 'Target retention (%)', newLimit: 'Daily new card limit', reviewLimit: 'Daily review answer limit', save: 'Save review settings', saved: 'Saved', loading: 'Loading settings', preview: 'Review settings are unavailable in browser preview.', error: 'Review settings failed', retry: 'Reload' },
  fr: { title: 'Réglages de révision', retention: 'Rétention cible (%)', newLimit: 'Limite quotidienne de nouvelles cartes', reviewLimit: 'Limite quotidienne de réponses', save: 'Enregistrer les réglages', saved: 'Enregistré', loading: 'Chargement des réglages', preview: 'Les réglages ne sont pas disponibles dans l’aperçu web.', error: 'Échec des réglages', retry: 'Recharger' },
} as const;

const autoLabels = { zh: '今后收藏自动加入复习', en: 'Automatically add future saved words to review', fr: 'Ajouter automatiquement les nouveaux mots aux révisions' } as const;

function AutoEnroll({ runtime, locale }: { readonly runtime: RuntimeAdapter; readonly locale: Locale }) {
  const [value, setValue] = useState<boolean | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  useEffect(() => {
    let active = true;
    void runtime.reviewAutoEnroll().then((result) => { if (active) setValue(result); })
      .catch((cause: unknown) => { if (active) setError(errorMessage(cause)); });
    return () => { active = false; };
  }, [runtime]);
  async function update(enabled: boolean) {
    setBusy(true); setError('');
    try { const result = await runtime.reviewAutoEnroll(enabled); if (result === null) throw new Error(messages[locale].preview); setValue(result); }
    catch (cause) { setError(errorMessage(cause)); }
    finally { setBusy(false); }
  }
  return <><label><input type="checkbox" checked={value ?? false} disabled={value === null || busy}
    onChange={(event) => { void update(event.target.checked); }} />{autoLabels[locale]}</label>
    {error && <p role="alert">{messages[locale].error}: {error}</p>}</>;
}

export function ReviewSettingsPanel({ runtime, locale }: { readonly runtime: RuntimeAdapter; readonly locale: Locale }) {
  const text = messages[locale];
  const [settings, setSettings] = useState<ReviewSettings | null>(null);
  const [status, setStatus] = useState<'loading' | 'ready' | 'saving' | 'saved' | 'preview' | 'error'>('loading');
  const [error, setError] = useState('');
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let active = true;
    void runtime.reviewSettings().then((value) => { if (active) { setSettings(value); setStatus(value ? 'ready' : 'preview'); } })
      .catch((cause: unknown) => { if (active) { setError(errorMessage(cause)); setStatus('error'); } });
    return () => { active = false; };
  }, [runtime, attempt]);
  async function save() {
    if (!settings) return;
    setStatus('saving'); setError('');
    try { const value = await runtime.reviewSettings(settings); setSettings(value); setStatus(value ? 'saved' : 'preview'); }
    catch (cause) { setError(errorMessage(cause)); setStatus('error'); }
  }
  const fields = [['retention_percent', text.retention, 70, 99], ['new_limit', text.newLimit, 0, 1_000_000], ['review_limit', text.reviewLimit, 0, 1_000_000]] as const;
  return <section className="speech-settings" aria-labelledby="review-settings-title">
    <h3 id="review-settings-title">{text.title}</h3>
    <AutoEnroll runtime={runtime} locale={locale} />
    {settings && <><div className="speech-options">{fields.map(([key, label, min, max]) => <label key={key}>{label}
      <input type="number" min={min} max={max} step={1} value={settings[key]} disabled={status === 'saving'}
        onChange={(event) => { const value = event.currentTarget.valueAsNumber; setSettings({ ...settings, [key]: value }); setStatus('ready'); }} />
    </label>)}</div><div className="actions"><button disabled={status === 'saving' || fields.some(([key, , min, max]) => !Number.isInteger(settings[key]) || settings[key] < min || settings[key] > max)} onClick={() => void save()}>{text.save}</button></div></>}
    <p role="status">{status === 'loading' || status === 'saving' ? text.loading : status === 'preview' ? text.preview : status === 'saved' ? text.saved : ''}</p>
    {error && <div role="alert">{text.error}: {error}{!settings && <button onClick={() => { setError(''); setStatus('loading'); setAttempt((value) => value + 1); }}>{text.retry}</button>}</div>}
  </section>;
}
