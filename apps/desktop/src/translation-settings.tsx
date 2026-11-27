import { useEffect, useState } from 'react';
import type { Locale } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import type { TranslationSettings } from './platform/translation';
import { translationError } from './translation-errors';

const labels = {
  zh: { title: '服务与密钥', provider: '当前翻译服务', disabled: '未启用', tier: 'DeepL 套餐', url: '服务地址', model: '模型', key: '替换 API 密钥', state: '已保存的密钥状态', missing: '未配置', configured: '已配置', unreadable: '无法读取', clear: '清除密钥', save: '保存服务设置', test: '测试连接', success: '连接成功', saved: '服务设置已保存', preview: '在线翻译设置仅在桌面应用中可用。', failure: '服务设置操作失败' },
  en: { title: 'Services and keys', provider: 'Translation service', disabled: 'Disabled', tier: 'DeepL plan', url: 'Service URL', model: 'Model', key: 'Replace API key', state: 'Saved key status', missing: 'Missing', configured: 'Configured', unreadable: 'Unreadable', clear: 'Clear key', save: 'Save service settings', test: 'Test connection', success: 'Connection successful', saved: 'Service settings saved', preview: 'Online translation settings are available in the desktop app.', failure: 'Service settings failed' },
  fr: { title: 'Services et clés', provider: 'Service de traduction', disabled: 'Désactivé', tier: 'Offre DeepL', url: 'Adresse du service', model: 'Modèle', key: 'Remplacer la clé API', state: 'État de la clé enregistrée', missing: 'Absente', configured: 'Configurée', unreadable: 'Illisible', clear: 'Effacer la clé', save: 'Enregistrer le service', test: 'Tester la connexion', success: 'Connexion réussie', saved: 'Service enregistré', preview: 'Les services de traduction sont disponibles dans l’application de bureau.', failure: 'Échec du service' },
} as const;

export function TranslationSettingsPanel({ runtime, locale }: { readonly runtime: RuntimeAdapter; readonly locale: Locale }) {
  const text = labels[locale];
  const [settings, setSettings] = useState<TranslationSettings | null>();
  const [key, setKey] = useState('');
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);
  useEffect(() => { let active = true; void runtime.translationSettings(locale).then((value) => { if (active) setSettings(value); }).catch(() => { if (active) setMessage(text.failure); }); return () => { active = false; }; }, [runtime, locale, text.failure]);
  if (settings === null) return <section><h3>{text.title}</h3><p>{text.preview}</p></section>;
  if (!settings) return <section><h3>{text.title}</h3><p role="status">{message}</p></section>;
  const preferences = settings.preferences;
  const slot: 'openai_compatible' | 'deepl' = preferences.active_provider === 'openai_compatible' ? 'openai_compatible' : 'deepl';
  const replacement = key ? { provider: slot, key } : undefined;
  async function save() {
    setBusy(true); setMessage('');
    try { const next = await runtime.translationSettings(locale, preferences, replacement); if (next) setSettings(next); setKey(''); setMessage(text.saved); }
    catch (error: unknown) { setMessage(`${text.failure}: ${translationError(locale, error)}`); }
    finally { setBusy(false); }
  }
  async function clear() {
    setBusy(true); setMessage('');
    try { await runtime.translationClearKey(slot); const next = await runtime.translationSettings(locale); if (next) setSettings({ ...next, preferences }); setKey(''); }
    catch (error: unknown) { setMessage(`${text.failure}: ${translationError(locale, error)}`); }
    finally { setBusy(false); }
  }
  async function test() {
    setBusy(true); setMessage('');
    try { await runtime.testTranslationConnection(preferences, replacement); setMessage(text.success); }
    catch (error: unknown) { setMessage(`${text.failure}: ${translationError(locale, error)}`); }
    finally { setBusy(false); }
  }
  return <section className="speech-settings"><h3>{text.title}</h3>
    <label>{text.provider}<select value={preferences.active_provider} disabled={busy} onChange={(event) => {
      const value = event.target.value; if (value === 'disabled' || value === 'deepl' || value === 'openai_compatible') { setKey(''); setSettings({ ...settings, preferences: { ...preferences, active_provider: value } }); }
    }}><option value="disabled">{text.disabled}</option><option value="deepl">DeepL</option><option value="openai_compatible">OpenAI-compatible</option></select></label>
    {slot === 'deepl' ? <label>{text.tier}<select value={preferences.deepl.tier} disabled={busy} onChange={(event) => {
      const value = event.target.value; if (value === 'free' || value === 'pro') setSettings({ ...settings, preferences: { ...preferences, deepl: { tier: value } } });
    }}><option value="free">Free</option><option value="pro">Pro</option></select></label> : <>
      <label>{text.url}<input value={preferences.openai_compatible.base_url} maxLength={512} disabled={busy} onChange={(event) => setSettings({ ...settings, preferences: { ...preferences, openai_compatible: { ...preferences.openai_compatible, base_url: event.target.value } } })} /></label>
      <label>{text.model}<input value={preferences.openai_compatible.model} maxLength={256} disabled={busy} onChange={(event) => setSettings({ ...settings, preferences: { ...preferences, openai_compatible: { ...preferences.openai_compatible, model: event.target.value } } })} /></label>
    </>}
    <label>{text.key}<input type="password" value={key} autoComplete="off" disabled={busy} onChange={(event) => {
      const value = event.target.value; setKey(value); if (slot === 'deepl' && value.endsWith(':fx')) setSettings({ ...settings, preferences: { ...preferences, deepl: { tier: 'free' } } });
    }} /></label>
    <p>{text.state}: {text[settings.keys[slot]]}</p>
    <div className="actions"><button disabled={busy} onClick={() => void save()}>{text.save}</button><button disabled={busy} onClick={() => void clear()}>{text.clear}</button><button disabled={busy || preferences.active_provider === 'disabled'} onClick={() => void test()}>{text.test}</button></div>
    {message && <p role="status">{message}</p>}
  </section>;
}
