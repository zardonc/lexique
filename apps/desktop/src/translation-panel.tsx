import { useEffect, useRef, useState } from 'react';
import type { Locale } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import type { TranslationPreferences, TranslationResult, TranslationSettings } from './platform/translation';
import { translationError } from './translation-errors';
import { ExampleSpeech } from './example-speech';
import { notebookMessages } from './notebook-content';

const labels = {
  zh: { source: '原文语言', target: '目标语言', auto: '自动检测', swap: '互换语言', translate: '翻译', retry: '重试', cancel: '取消', copy: '复制译文', settings: '前往设置', disabled: '在线翻译未启用', missing: '密钥未配置', unreadable: '密钥无法读取', preview: '在线翻译仅在桌面应用中可用', error: '翻译失败', incomplete: '译文不完整，请重试。', copied: '已复制译文', result: '译文' },
  en: { source: 'Source language', target: 'Target language', auto: 'Auto-detect', swap: 'Swap languages', translate: 'Translate', retry: 'Retry', cancel: 'Cancel', copy: 'Copy translation', settings: 'Open settings', disabled: 'Online translation is disabled', missing: 'API key is missing', unreadable: 'API key is unreadable', preview: 'Online translation is available in the desktop app', error: 'Translation failed', incomplete: 'Translation incomplete; retry.', copied: 'Translation copied', result: 'Translation' },
  fr: { source: 'Langue source', target: 'Langue cible', auto: 'Détection automatique', swap: 'Inverser les langues', translate: 'Traduire', retry: 'Réessayer', cancel: 'Annuler', copy: 'Copier la traduction', settings: 'Ouvrir les paramètres', disabled: 'La traduction en ligne est désactivée', missing: 'Clé API absente', unreadable: 'Clé API illisible', preview: 'La traduction en ligne est disponible dans l’application de bureau', error: 'Échec de la traduction', incomplete: 'Traduction incomplète ; réessayez.', copied: 'Traduction copiée', result: 'Traduction' },
} as const;
const languages = ['fr', 'zh-CN', 'en'] as const;
const languageNames = { zh: { fr: '法语', 'zh-CN': '简体中文', en: '英语' }, en: { fr: 'French', 'zh-CN': 'Simplified Chinese', en: 'English' }, fr: { fr: 'Français', 'zh-CN': 'Chinois simplifié', en: 'Anglais' } } as const;
const unconfiguredMessages = { zh: '服务未配置', en: 'Service is not configured', fr: 'Service non configuré' } as const;
const openaiNames = { zh: 'OpenAI 兼容服务', en: 'OpenAI-compatible', fr: 'Service compatible OpenAI' } as const;
type Language = typeof languages[number];

export function useTranslation(runtime: RuntimeAdapter, locale: Locale, { raw, visible }: { readonly raw: string; readonly visible: boolean }) {
  const [settings, setSettings] = useState<TranslationSettings | null>();
  const [result, setResult] = useState<TranslationResult>();
  const [status, setStatus] = useState('');
  const [busy, setBusy] = useState(false);
  const [complete, setComplete] = useState(false);
  const [saving, setSaving] = useState(false);
  const active = useRef<string | undefined>(undefined);
  useEffect(() => { let live = true; if (visible) void runtime.translationSettings(locale).then((value) => { if (live) setSettings(value); }).catch((error: unknown) => { if (live) setStatus(translationError(locale, error)); }); return () => { live = false; }; }, [runtime, locale, visible]);
  function cancel() { const id = active.current; active.current = undefined; if (id) void runtime.cancelTranslation(id).catch((error: unknown) => setStatus(translationError(locale, error))); setBusy(false); setComplete(false); setResult(undefined); }
  useEffect(() => () => { const id = active.current; active.current = undefined; if (id) void runtime.cancelTranslation(id).catch(() => {}); }, [runtime]);
  async function change(preferences: TranslationPreferences) {
    cancel(); setSaving(true); setStatus('');
    try { const next = await runtime.translationSettings(locale, preferences); if (next) setSettings(next); }
    catch (error: unknown) { setStatus(translationError(locale, error)); } finally { setSaving(false); }
  }
  async function translate() {
    cancel(); const id = crypto.randomUUID(); active.current = id; setBusy(true); setStatus('');
    let unlisten: (() => void) | undefined;
    try {
      unlisten = await runtime.listenTranslation((delta) => { if (active.current === id && delta.id === id) setResult((previous) => ({ text: (previous?.text ?? '') + delta.text, source: delta.source, target: delta.target })); });
      if (active.current !== id) return;
      const value = await runtime.translate(id, raw, locale); if (active.current === id) { setResult(value); setComplete(true); }
    }
    catch (error: unknown) { if (active.current === id) setStatus(translationError(locale, error)); }
    finally { unlisten?.(); if (active.current === id) { active.current = undefined; setBusy(false); } }
  }
  return { settings, result, status, setStatus, busy, complete, saving, cancel, change, translate };
}

function LanguageControls({ translation, locale, preferences }: { readonly translation: ReturnType<typeof useTranslation>; readonly locale: Locale; readonly preferences: TranslationPreferences }) {
  const text = labels[locale], { saving } = translation;
  function changeLanguage(field: 'source_language' | 'target_language', value: string) {
    if (field === 'source_language' && (value === 'auto' || languages.some((item) => item === value))) void translation.change({ ...preferences, source_language: value as TranslationPreferences['source_language'] });
    if (field === 'target_language' && languages.some((item) => item === value)) void translation.change({ ...preferences, target_language: value as Language });
  }
  return <div className="actions">
    <label>{text.source}<select value={preferences.source_language} disabled={saving} onChange={(event) => changeLanguage('source_language', event.target.value)}><option value="auto">{text.auto}</option>{languages.map((value) => <option key={value} value={value}>{languageNames[locale][value]}</option>)}</select></label>
    <label>{text.target}<select value={preferences.target_language} disabled={saving} onChange={(event) => changeLanguage('target_language', event.target.value)}>{languages.map((value) => <option key={value} value={value}>{languageNames[locale][value]}</option>)}</select></label>
    <button type="button" disabled={saving || preferences.source_language === 'auto'} onClick={() => { if (preferences.source_language !== 'auto') void translation.change({ ...preferences, source_language: preferences.target_language, target_language: preferences.source_language }); }}>{text.swap}</button>
  </div>;
}

function TranslationOutput({ translation, locale, runtime, onSelection, visible }: { readonly translation: ReturnType<typeof useTranslation>; readonly locale: Locale; readonly runtime: RuntimeAdapter; readonly onSelection: (value: string) => void; readonly visible: boolean }) {
  const text = labels[locale], { result, status } = translation;
  const output = useRef<HTMLPreElement>(null);
  useEffect(() => {
    function update() {
      const selection = window.getSelection(), node = output.current;
      onSelection(translation.complete && selection && node && node.contains(selection.anchorNode) && node.contains(selection.focusNode) ? selection.toString() : '');
    }
    document.addEventListener('selectionchange', update);
    return () => document.removeEventListener('selectionchange', update);
  }, [onSelection, translation.complete]);
  function select(node: HTMLElement) { const selection = window.getSelection(); if (translation.complete && selection && node.contains(selection.anchorNode) && node.contains(selection.focusNode)) onSelection(selection.toString()); }
  return <>{result && <section><h3>{text.result}</h3><p>{result.source === 'auto' ? text.auto : languageNames[locale][result.source]} → {languageNames[locale][result.target]}</p><pre ref={output} lang={result.target} tabIndex={0} onMouseUp={(event) => select(event.currentTarget)} onKeyUp={(event) => select(event.currentTarget)}>{result.text}</pre>{visible && translation.complete && result.target === 'fr' && <ExampleSpeech value={result.text} runtime={runtime} text={notebookMessages[locale]} />}{!translation.busy && !translation.complete && <p role="alert">{text.incomplete}</p>}<button type="button" disabled={!translation.complete} onClick={() => void navigator.clipboard.writeText(result.text).then(() => translation.setStatus('copied')).catch(() => translation.setStatus('clipboard_error'))}>{text.copy}</button></section>}
    {status && <p role="status">{status === 'copied' ? text.copied : `${text.error}: ${translationError(locale, status)}`}</p>}</>;
}

function openaiReadiness(preferences: TranslationPreferences, keyState: TranslationSettings['keys']['openai_compatible']) {
  try {
    const url = new URL(preferences.openai_compatible.base_url);
    const local = ['localhost', '127.0.0.1', '[::1]'].includes(url.hostname);
    const configured = (url.protocol === 'https:' || (local && url.protocol === 'http:')) && !!preferences.openai_compatible.model.trim();
    return { configured, available: configured && keyState !== 'unreadable' && (keyState === 'configured' || local) };
  } catch { return { configured: false, available: false }; }
}

function serviceState(settings: TranslationSettings, locale: Locale) {
  const text = labels[locale], preferences = settings.preferences;
  const openai = preferences.active_provider === 'openai_compatible';
  const keyState = settings.keys[openai ? 'openai_compatible' : 'deepl'];
  const readiness = openai ? openaiReadiness(preferences, keyState) : { configured: true, available: keyState === 'configured' };
  const available = preferences.active_provider !== 'disabled' && readiness.available;
  const reason = preferences.active_provider === 'disabled' ? text.disabled : !readiness.configured ? unconfiguredMessages[locale] : keyState === 'unreadable' ? text.unreadable : text.missing;
  let destination = preferences.deepl.tier === 'free' ? 'DeepL (Free · api-free.deepl.com)' : 'DeepL (Pro · api.deepl.com)';
  if (openai) { try { destination = `${openaiNames[locale]} · ${new URL(preferences.openai_compatible.base_url).host} · ${preferences.openai_compatible.model}`; } catch { destination = openaiNames[locale]; } }
  return { available, reason, destination };
}

function LoadedTranslation({ translation, locale, raw, onSettings, settings, runtime, onSelection, visible }: { readonly translation: ReturnType<typeof useTranslation>; readonly locale: Locale; readonly raw: string; readonly onSettings: () => void; readonly settings: TranslationSettings; readonly runtime: RuntimeAdapter; readonly onSelection: (value: string) => void; readonly visible: boolean }) {
  const text = labels[locale], { busy, saving } = translation;
  const preferences = settings.preferences;
  const { available, reason, destination } = serviceState(settings, locale);
  return <section><LanguageControls translation={translation} locale={locale} preferences={preferences} /><p>{available ? destination : <>{reason} <button type="button" onClick={onSettings}>{text.settings}</button></>}</p>
    <div className="actions"><button type="button" disabled={!available || busy || saving || !raw.trim()} onClick={() => void translation.translate()}>{translation.result && !translation.complete && !busy ? text.retry : text.translate}</button>{busy && <button type="button" onClick={translation.cancel}>{text.cancel}</button>}</div>
    <TranslationOutput translation={translation} locale={locale} runtime={runtime} onSelection={onSelection} visible={visible} />
  </section>;
}

export function TranslationPanel(props: { readonly translation: ReturnType<typeof useTranslation>; readonly locale: Locale; readonly raw: string; readonly onSettings: () => void; readonly runtime: RuntimeAdapter; readonly onSelection: (value: string) => void; readonly visible: boolean }) {
  const { settings, status } = props.translation, text = labels[props.locale];
  if (settings === null) return <p>{text.preview}</p>;
  if (!settings) return status ? <p role="alert">{text.error}: {translationError(props.locale, status)}</p> : null;
  return <LoadedTranslation {...props} settings={settings} />;
}
