import { useEffect, useState } from 'react';
import type { Locale } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import type { ShortcutStatus } from './platform/selection';
import { errorMessage } from './platform/vocabulary';

const messages = {
  zh: { title: '取词', shortcut: '取词快捷键', save: '保存取词快捷键', current: '当前使用：', saved: '取词快捷键已保存。',
    loading: '正在读取快捷键…', retry: '重新读取', inactive: '取词快捷键未注册。请更换键位并保存。',
    hint: '在其他应用选中文字后按下此键，选中文字进入翻译模块。默认 Ctrl+Alt+Shift+D，可自定义；键位须包含 Control、Alt 或 Super（macOS 为 Command）。',
    preview: '浏览器预览：请在桌面应用中配置系统快捷键。' },
  en: { title: 'Selected-text input', shortcut: 'Selection shortcut', save: 'Save selection shortcut', current: 'Currently in use: ', saved: 'Selection shortcut saved.',
    loading: 'Loading shortcut…', retry: 'Reload', inactive: 'The selection shortcut is not registered. Choose another shortcut and save it.',
    hint: 'Select text in another app, then press this shortcut to send it to translation. Default: Ctrl+Alt+Shift+D; customize it with Control, Alt or Super (Command on macOS).',
    preview: 'Browser preview: configure system shortcuts in the desktop app.' },
  fr: { title: 'Capture du texte sélectionné', shortcut: 'Raccourci de sélection', save: 'Enregistrer le raccourci de sélection', current: 'Raccourci actif : ', saved: 'Raccourci de sélection enregistré.',
    loading: 'Chargement du raccourci…', retry: 'Recharger', inactive: 'Le raccourci de sélection n’est pas enregistré. Choisissez-en un autre et enregistrez-le.',
    hint: 'Sélectionnez du texte dans une autre application, puis utilisez ce raccourci pour l’envoyer à la traduction. Par défaut : Ctrl+Alt+Shift+D ; personnalisez-le avec Control, Alt ou Super (Command sur macOS).',
    preview: 'Aperçu web : configurez les raccourcis système dans l’application de bureau.' },
} as const;

const ocrMessages = {
  zh: { ...messages.zh, title: '截图 OCR', shortcut: '截图快捷键', save: '保存截图快捷键', saved: '截图快捷键已保存。',
    loading: '正在读取截图快捷键…', retry: '重新读取截图快捷键',
    inactive: '截图快捷键未注册。请更换键位并保存。', hint: '按下快捷键后框选屏幕区域，截图预览及本地识别文字进入翻译模块。默认 Ctrl+Alt+Shift+O，可自定义；键位须包含 Control、Alt 或 Super。' },
  en: { ...messages.en, title: 'Screen OCR', shortcut: 'Screenshot shortcut', save: 'Save screenshot shortcut', saved: 'Screenshot shortcut saved.',
    loading: 'Loading screenshot shortcut…', retry: 'Reload screenshot shortcut',
    inactive: 'The screenshot shortcut is not registered. Choose another shortcut and save it.', hint: 'Press the shortcut and select a screen region. The preview and locally recognized text enter translation. Default: Ctrl+Alt+Shift+O; customize it with Control, Alt or Super.' },
  fr: { ...messages.fr, title: 'Reconnaissance par capture', shortcut: 'Raccourci de capture', save: 'Enregistrer le raccourci de capture', saved: 'Raccourci de capture enregistré.',
    loading: 'Chargement du raccourci de capture…', retry: 'Recharger le raccourci de capture',
    inactive: 'Le raccourci de capture n’est pas enregistré. Choisissez-en un autre et enregistrez-le.', hint: 'Utilisez le raccourci et sélectionnez une zone. L’aperçu et le texte reconnu localement apparaissent dans la traduction. Par défaut : Ctrl+Alt+Shift+O ; personnalisez-le avec Control, Alt ou Super.' },
} as const;
type ShortcutText = { readonly [Key in keyof typeof messages.zh]: string };

function useShortcutSettings(read: RuntimeAdapter['selectionShortcut']) {
  const [status, setStatus] = useState<ShortcutStatus | null>();
  const [shortcut, setShortcut] = useState(''), [error, setError] = useState('');
  const [phase, setPhase] = useState<'loading' | 'ready' | 'saving' | 'saved'>('loading');
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let active = true;
    setPhase('loading'); setError('');
    void read().then((value) => {
      if (active) { setStatus(value); setShortcut(value?.shortcut ?? ''); setPhase('ready'); }
    }).catch((cause: unknown) => { if (active) { setError(errorMessage(cause)); setPhase('ready'); } });
    return () => { active = false; };
  }, [read, attempt]);
  async function save() {
    setPhase('saving'); setError('');
    try { const value = await read(shortcut); setStatus(value); setShortcut(value?.shortcut ?? ''); setPhase('saved'); }
    catch (cause: unknown) { setError(errorMessage(cause)); setPhase('ready'); }
  }
  return { status, shortcut, error, phase, save, setShortcut, setError, setPhase, setAttempt };
}

function ShortcutState({ settings, text }: { readonly settings: ReturnType<typeof useShortcutSettings>; readonly text: ShortcutText }) {
  const { phase, status, error } = settings;
  return <>
    {phase === 'loading' && <p role="status">{text.loading}</p>}
    <ShortcutRegistration status={status} text={text} />
    {error && <p role="alert">{error}</p>}
    {phase === 'saved' && status?.registered && <p role="status">{text.saved}</p>}
  </>;
}

function ShortcutRegistration({ status, text }: { readonly status?: ShortcutStatus | null; readonly text: ShortcutText }) {
  return <>
    {status === null && <p>{text.preview}</p>}
    {status?.registered && <p>{text.current}{status.shortcut}</p>}
    {status && !status.registered && <p role="alert">{text.inactive}</p>}
    {status?.error && <p role="alert">{status.error}</p>}
  </>;
}

export function SelectionSettings({ runtime, locale, ocr = false }: { readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly ocr?: boolean }) {
  const text = (ocr ? ocrMessages : messages)[locale], settings = useShortcutSettings(ocr ? runtime.ocrShortcut : runtime.selectionShortcut);
  const title = ocr ? 'ocr-settings-title' : 'selection-settings-title';
  const { status, shortcut, phase } = settings;
  const busy = phase === 'loading' || phase === 'saving', disabled = !status || busy;
  return <section className="selection-settings" aria-labelledby={title}>
    <h3 id={title}>{text.title}</h3><p>{text.hint}</p>
    <ShortcutState settings={settings} text={text} />
    <form onSubmit={(event) => { event.preventDefault(); void settings.save(); }}><label>{text.shortcut}
      <input value={shortcut} required maxLength={128} disabled={disabled} spellCheck={false} autoCapitalize="none"
        onChange={(event) => { settings.setShortcut(event.target.value); settings.setError(''); settings.setPhase('ready'); }} /></label>
      <button disabled={disabled}>{text.save}</button></form>
    <button disabled={busy} onClick={() => settings.setAttempt((value) => value + 1)}>{text.retry}</button>
  </section>;
}
