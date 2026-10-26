import { useEffect, useState } from 'react';
import type { Locale } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import type { SpeechPreferences } from './platform/speech';

const messages = {
  zh: { title: '法语语音 · Piper', voice: '法语声线', female: '法语女声', male: '法语男声',
    speed: '朗读语速', normal: '正常', slow: '慢速', save: '保存语音偏好', saved: '语音偏好已保存。',
    loading: '正在读取语音偏好…', failed: '语音偏好操作失败', retry: '重新读取',
    preview: '浏览器预览：请在桌面应用中保存语音偏好。',
    pending: '偏好可保存并在重启后恢复；离线朗读功能尚未接入。' },
  en: { title: 'French speech · Piper', voice: 'French voice', female: 'French female voice', male: 'French male voice',
    speed: 'Speaking speed', normal: 'Normal', slow: 'Slow', save: 'Save speech preferences', saved: 'Speech preferences saved.',
    loading: 'Loading speech preferences…', failed: 'Speech preference operation failed', retry: 'Reload',
    preview: 'Browser preview: save speech preferences in the desktop app.',
    pending: 'Preferences are saved across restarts. Offline speech is not connected yet.' },
  fr: { title: 'Voix française · Piper', voice: 'Voix française', female: 'Voix féminine française', male: 'Voix masculine française',
    speed: 'Vitesse de lecture', normal: 'Normale', slow: 'Lente', save: 'Enregistrer les préférences vocales', saved: 'Préférences vocales enregistrées.',
    loading: 'Chargement des préférences vocales…', failed: 'Échec de l’opération sur les préférences vocales', retry: 'Recharger',
    preview: 'Aperçu web : enregistrez les préférences dans l’application de bureau.',
    pending: 'Les préférences sont conservées après redémarrage. La lecture hors ligne n’est pas encore intégrée.' },
} as const;

export function SpeechSettings({ runtime, locale }: { readonly runtime: RuntimeAdapter; readonly locale: Locale }) {
  const text = messages[locale];
  const [choices, setChoices] = useState<SpeechPreferences>({ voice: 'female', speed: 'normal' });
  const [state, setState] = useState<'loading' | 'ready' | 'preview' | 'saving' | 'saved' | 'failed'>('loading');
  const [error, setError] = useState('');
  const [loaded, setLoaded] = useState(false);
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let active = true;
    runtime.speechPreferences().then((value) => {
      if (!active) return;
      if (value) { setChoices(value); setLoaded(true); setState('ready'); }
      else setState('preview');
    }).catch((reason: unknown) => { if (active) { setError(String(reason)); setState('failed'); } });
    return () => { active = false; };
  }, [runtime, attempt]);
  async function save() {
    setState('saving'); setError('');
    try {
      const value = await runtime.speechPreferences(choices);
      if (value) { setChoices(value); setState('saved'); }
      else setState('preview');
    } catch (reason) { setError(String(reason)); setState('failed'); }
  }
  const disabled = !loaded || state === 'saving';
  return <section className="speech-settings" aria-labelledby="speech-title">
    <h3 id="speech-title">{text.title}</h3><p>{text.pending}</p>
    <div className="speech-options"><label>{text.voice}<select value={choices.voice} disabled={disabled}
      onChange={(event) => { const voice = event.target.value; if (voice === 'female' || voice === 'male') { setChoices({ ...choices, voice }); setState('ready'); setError(''); } }}>
      <option value="female">{text.female}</option><option value="male">{text.male}</option>
    </select></label>
    <label>{text.speed}<select value={choices.speed} disabled={disabled}
      onChange={(event) => { const speed = event.target.value; if (speed === 'normal' || speed === 'slow') { setChoices({ ...choices, speed }); setState('ready'); setError(''); } }}>
      <option value="normal">{text.normal}</option><option value="slow">{text.slow}</option>
    </select></label></div>
    <div className="actions"><button disabled={disabled} onClick={() => void save()}>{text.save}</button></div>
    <p role="status">{state === 'loading' || state === 'saving' ? text.loading : state === 'preview' ? text.preview : state === 'saved' ? text.saved : ''}</p>
    {error && <div role="alert">{text.failed}: {error}{!loaded && <button onClick={() => { setError(''); setState('loading'); setAttempt((value) => value + 1); }}>{text.retry}</button>}</div>}
  </section>;
}
