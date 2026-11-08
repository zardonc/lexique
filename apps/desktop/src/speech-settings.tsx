import { useEffect, useState } from 'react';
import type { Locale } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import type { SpeechPreferences } from './platform/speech';
import { SpeechButton } from './speech-button';
import { notebookMessages } from './notebook-content';

const audition = 'Bonjour, nous apprenons le français ensemble.';

const messages = {
  zh: { title: '法语语音', voice: '法语声线', female: '法语女声', male: '法语男声',
    speed: '朗读语速', unit: 'WPM：每分钟朗读的词数', normal: '正常', slow: '慢速', fast: '快速', presets: '语速预设', audition: '试听短句', save: '保存语音偏好', saved: '语音偏好已保存。',
    loading: '正在读取语音偏好…', failed: '语音偏好操作失败', retry: '重新读取',
    preview: '浏览器预览：请在桌面应用中保存语音偏好。',
    pending: '朗读速度为近似值。试听使用当前选择；保存后用于下次播放，当前播放速度不变。新语音使用所选声线，已选单词音频保持不变。' },
  en: { title: 'French speech', voice: 'French voice', female: 'French female voice', male: 'French male voice',
    speed: 'Speaking speed', unit: 'WPM: words read per minute', normal: 'Normal', slow: 'Slow', fast: 'Fast', presets: 'Speed presets', audition: 'Preview sentence', save: 'Save speech preferences', saved: 'Speech preferences saved.',
    loading: 'Loading speech preferences…', failed: 'Speech preference operation failed', retry: 'Reload',
    preview: 'Browser preview: save speech preferences in the desktop app.',
    pending: 'Speaking speed is approximate. Preview uses your current choices; saved settings apply to the next playback. New speech uses this voice. Selected word audio is preserved.' },
  fr: { title: 'Voix française', voice: 'Voix française', female: 'Voix féminine française', male: 'Voix masculine française',
    speed: 'Vitesse de lecture', unit: 'WPM : mots lus par minute', normal: 'Normale', slow: 'Lente', fast: 'Rapide', presets: 'Vitesses prédéfinies', audition: 'Écouter la phrase', save: 'Enregistrer les préférences vocales', saved: 'Préférences vocales enregistrées.',
    loading: 'Chargement des préférences vocales…', failed: 'Échec de l’opération sur les préférences vocales', retry: 'Recharger',
    preview: 'Aperçu web : enregistrez les préférences dans l’application de bureau.',
    pending: 'La vitesse est approximative. L’essai utilise vos choix actuels ; les préférences enregistrées s’appliquent à la prochaine lecture. Les nouveaux enregistrements utilisent cette voix. L’audio choisi des mots est conservé.' },
} as const;

export function SpeechSettings({ runtime, locale }: { readonly runtime: RuntimeAdapter; readonly locale: Locale }) {
  const text = messages[locale];
  const [choices, setChoices] = useState<SpeechPreferences>({ voice: 'female', wpm: 180 });
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
  function chooseWpm(wpm: number) { setChoices({ ...choices, wpm }); setState('ready'); setError(''); }
  return <section className="speech-settings" aria-labelledby="speech-title">
    <h3 id="speech-title">{text.title}</h3><p>{text.pending}</p>
    <div className="speech-options"><label>{text.voice}<select value={choices.voice} disabled={disabled}
      onChange={(event) => { const voice = event.target.value; if (voice === 'female' || voice === 'male') { setChoices({ ...choices, voice }); setState('ready'); setError(''); } }}>
      <option value="female">{text.female}</option><option value="male">{text.male}</option>
    </select></label>
    <div className="speech-wpm"><label htmlFor="speech-wpm">{text.speed}</label><output htmlFor="speech-wpm">{choices.wpm} WPM</output>
      <input id="speech-wpm" type="range" min="80" max="240" step="20" value={choices.wpm} disabled={disabled}
        aria-describedby="speech-wpm-help" aria-valuetext={`${choices.wpm} WPM`}
        onChange={(event) => chooseWpm(Number(event.target.value))} />
      <div className="speech-wpm-endpoints"><span>80 WPM</span><span>240 WPM</span></div>
      <p id="speech-wpm-help">{text.unit}</p>
    </div></div>
    <div className="actions" role="group" aria-label={text.presets}>
      {([[100, text.slow], [180, text.normal], [240, text.fast]] as const).map(([wpm, label]) =>
        <button key={wpm} disabled={disabled} aria-pressed={choices.wpm === wpm} onClick={() => chooseWpm(wpm)}>{label} {wpm} WPM</button>)}
    </div>
    <p lang="fr">{audition} <SpeechButton word={audition} runtime={runtime} preferences={choices} disabled={disabled}
      text={{ ...notebookMessages[locale], playWord: text.audition, preview: text.preview }} /></p>
    <div className="actions"><button disabled={disabled} onClick={() => void save()}>{text.save}</button></div>
    <p role="status">{state === 'loading' || state === 'saving' ? text.loading : state === 'preview' ? text.preview : state === 'saved' ? text.saved : ''}</p>
    {error && <div role="alert">{text.failed}: {error}{!loaded && <button onClick={() => { setError(''); setState('loading'); setAttempt((value) => value + 1); }}>{text.retry}</button>}</div>}
  </section>;
}
