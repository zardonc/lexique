import { useEffect, useRef, useState } from 'react';
import type { Locale } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import type { BatchStatus } from './platform/runtime';
import type { BackupCounts, BackupResult, BackupStatus } from './platform/backup';
import { errorMessage } from './platform/vocabulary';
import { resolvePendingReviews } from './pending-review-requests';

const messages = {
  zh: { title: '手动备份', create: '创建备份', saved: '备份已保存', working: '正在准备备份…', error: '备份失败', choosing: '请选择新备份文件的位置。', check: '检查备份状态', cancel: '取消备份', cancelled: '备份已取消。', cancelling: '已请求取消，正在等待备份停止…',
    paused: '学习写入和后台音频维护暂时暂停。采集完成后恢复使用。',
    phases: { draining: '正在等待已确认操作完成…', collecting: '正在采集学习数据和音频…', packaging: '正在打包备份，可继续学习。', validating: '正在检查备份文件，可继续学习。', publishing: '正在保存完整备份文件，可继续学习。' },
    preview: '手动备份仅在桌面应用中可用。',
    contents: '备份保存已提交的笔记、例句、卡片进度、评分历史和可迁移偏好，以及当前和仍在保留期的上一版单词音频。',
    resources: '自定义 MDX 仅保存身份与顺序清单，词典正文和模型需在目标设备另行提供。',
    privacy: '备份包含个人笔记与学习历史，文件不加密。请选择新文件位置并妥善保管。',
    destination: '保存位置', size: '文件大小', bytes: '字节', created: '创建时间',
    counts: { vocabulary: '生词', cards: '卡片', ratings: '评分', batches: '批量请求', custom_dictionaries: '自定义词典', legacy_audio: '旧音频关联',
      included_audio_versions: '已包含音频版本', included_legacy_audio: '已包含旧音频', audio_files: '音频文件', source_missing_audio: '原音频缺失', source_invalid_audio: '原音频损坏' } },
  en: { title: 'Manual backup', create: 'Create backup', saved: 'Backup saved', working: 'Preparing backup…', error: 'Backup failed', choosing: 'Choose a location for a new backup file.', check: 'Check backup status', cancel: 'Cancel backup', cancelled: 'Backup cancelled.', cancelling: 'Cancel requested. Waiting for backup to stop…',
    paused: 'Learning changes and background audio maintenance are temporarily paused. They resume when collection finishes.',
    phases: { draining: 'Waiting for accepted operations to finish…', collecting: 'Collecting learning data and audio…', packaging: 'Packaging the backup. You can continue learning.', validating: 'Checking the backup file. You can continue learning.', publishing: 'Saving the complete backup file. You can continue learning.' },
    preview: 'Manual backup is available in the desktop app only.',
    contents: 'Back up committed notes, examples, card progress, rating history and portable preferences, with current word audio and the previous version still within its retention period.',
    resources: 'Custom MDX dictionaries keep their identity and order only. Provide dictionary files and models separately on the destination device.',
    privacy: 'Backups contain personal notes and learning history and are not encrypted. Choose a new file location and keep it private.',
    destination: 'Saved location', size: 'File size', bytes: 'bytes', created: 'Created at',
    counts: { vocabulary: 'Vocabulary', cards: 'Cards', ratings: 'Ratings', batches: 'Batch requests', custom_dictionaries: 'Custom dictionaries', legacy_audio: 'Legacy audio links',
      included_audio_versions: 'Included audio versions', included_legacy_audio: 'Included legacy audio', audio_files: 'Audio files', source_missing_audio: 'Source audio missing', source_invalid_audio: 'Source audio damaged' } },
  fr: { title: 'Sauvegarde manuelle', create: 'Créer une sauvegarde', saved: 'Sauvegarde enregistrée', working: 'Préparation de la sauvegarde…', error: 'Échec de la sauvegarde', choosing: 'Choisissez l’emplacement d’un nouveau fichier.', check: 'Vérifier la sauvegarde', cancel: 'Annuler la sauvegarde', cancelled: 'Sauvegarde annulée.', cancelling: 'Annulation demandée. Attente de l’arrêt de la sauvegarde…',
    paused: 'Les modifications et la maintenance audio sont temporairement suspendues. Elles reprennent après la collecte.',
    phases: { draining: 'Attente de la fin des opérations acceptées…', collecting: 'Collecte des données et des audios…', packaging: 'Création du fichier. Vous pouvez continuer à étudier.', validating: 'Vérification du fichier. Vous pouvez continuer à étudier.', publishing: 'Enregistrement du fichier complet. Vous pouvez continuer à étudier.' },
    preview: 'La sauvegarde manuelle est disponible uniquement dans l’application de bureau.',
    contents: 'La sauvegarde conserve les notes, exemples, progrès des cartes, évaluations et préférences portables enregistrés, ainsi que l’audio actuel et la version précédente encore conservée.',
    resources: 'Seuls l’identité et l’ordre des dictionnaires MDX sont conservés. Les fichiers et modèles doivent être fournis séparément sur l’appareil cible.',
    privacy: 'Les sauvegardes contiennent des notes personnelles et l’historique d’apprentissage sans chiffrement. Choisissez un nouveau fichier et conservez-le en lieu sûr.',
    destination: 'Emplacement', size: 'Taille du fichier', bytes: 'octets', created: 'Date de création',
    counts: { vocabulary: 'Vocabulaire', cards: 'Cartes', ratings: 'Évaluations', batches: 'Demandes groupées', custom_dictionaries: 'Dictionnaires personnalisés', legacy_audio: 'Liens audio anciens',
      included_audio_versions: 'Versions audio incluses', included_legacy_audio: 'Audios anciens inclus', audio_files: 'Fichiers audio', source_missing_audio: 'Audio source manquant', source_invalid_audio: 'Audio source endommagé' } },
} as const;

function activeBackup(status: BackupStatus | null) { return status?.state === 'choosing' || status?.state === 'running'; }
function pausesLearning(status: BackupStatus | null) { return status?.state === 'running' && (status.phase === 'draining' || status.phase === 'collecting'); }
const phaseOrder = { draining: 1, collecting: 2, packaging: 3, validating: 4, publishing: 5 } as const;
function progressOrder(status: BackupStatus) { return status.state === 'running' ? phaseOrder[status.phase] : status.state === 'choosing' ? 0 : 6; }
function regresses(previous: BackupStatus | null, value: BackupStatus) {
  return previous?.operation === value.operation && progressOrder(value) < progressOrder(previous);
}

function useBackupStatus(runtime: RuntimeAdapter) {
  const [status, setStatus] = useState<BackupStatus | null>(null), [error, setError] = useState('');
  const current = useRef<BackupStatus | null>(null), updates = useRef(0), live = useRef(true);
  const unlisten = useRef<(() => void) | undefined>(undefined);
  function update(value: BackupStatus) {
    if (!live.current || regresses(current.current, value)) return;
    current.current = value; updates.current += 1; setStatus(value); setError('');
    if (!activeBackup(value)) { unlisten.current?.(); unlisten.current = undefined; }
  }
  function receive(value: BackupStatus) {
    const previous = current.current;
    if (previous && value.operation === previous.operation && activeBackup(previous)) update({ ...value, cancel_requested: value.cancel_requested || previous.cancel_requested });
  }
  function reset() { current.current = null; updates.current += 1; setStatus(null); setError(''); }
  async function request(read: (operation: string) => Promise<BackupStatus>) {
    const operation = current.current?.operation;
    if (!operation) return;
    const version = updates.current, value = await read(operation);
    if (version === updates.current) update(value);
  }
  useEffect(() => {
    live.current = true;
    return () => { live.current = false; unlisten.current?.(); };
  }, []);
  return { status, error, current, live, unlisten, update, receive, reset, refresh: () => request(runtime.backupStatus), cancel: () => request(runtime.cancelBackup), failed: (cause: unknown) => setError(errorMessage(cause)) };
}

function useBackupExport(runtime: RuntimeAdapter, onResolved: (batch: BatchStatus | null) => void) {
  const operation = useBackupStatus(runtime);
  const [preparing, setPreparing] = useState(false), [cancelling, setCancelling] = useState(false), starting = useRef(false);
  async function start() {
    if (starting.current || activeBackup(operation.current.current) || !runtime.backupAvailable()) return;
    starting.current = true; setPreparing(true); operation.reset();
    try {
      onResolved(await resolvePendingReviews(runtime));
      const stop = await runtime.onBackupProgress(operation.receive, operation.failed);
      if (!operation.live.current) { stop(); return; }
      operation.unlisten.current = stop;
      const value = await runtime.startBackup();
      operation.update(value); setPreparing(false);
      if (activeBackup(value)) await operation.refresh();
    } catch (cause) {
      operation.failed(cause);
      if (!activeBackup(operation.current.current)) { operation.unlisten.current?.(); operation.unlisten.current = undefined; }
    } finally { starting.current = false; setPreparing(false); }
  }
  async function cancel() {
    setCancelling(true);
    try { await operation.cancel(); }
    catch (cause) { operation.failed(cause); }
    finally { setCancelling(false); }
  }
  return { ...operation, start, cancel, preparing, cancelling, active: preparing || activeBackup(operation.status), paused: preparing || pausesLearning(operation.status) };
}

function Progress({ status, preparing, locale }: { readonly status: BackupStatus | null; readonly preparing: boolean; readonly locale: Locale }) {
  const text = messages[locale];
  const label = preparing ? text.working : status?.state === 'choosing' ? text.choosing : status?.state === 'running' ? text.phases[status.phase] : '';
  return label ? <><p role="status">{label}{status?.cancel_requested && ` ${text.cancelling}`}{pausesLearning(status) && ` ${text.paused}`}</p><progress aria-label={label} /></> : null;
}

function Result({ result, locale }: { readonly result: BackupResult; readonly locale: Locale }) {
  const text = messages[locale];
  const language = locale === 'zh' ? 'zh-CN' : locale;
  const keys = Object.keys(text.counts) as (keyof BackupCounts)[];
  return <dl className="backup-result">
    <div><dt>{text.destination}</dt><dd>{result.destination}</dd></div>
    <div><dt>{text.size}</dt><dd>{result.byte_length.toLocaleString(language)} {text.bytes}</dd></div>
    <div><dt>{text.created}</dt><dd>{new Date(result.created_at * 1000).toLocaleString(language)}</dd></div>
    {keys.map((key) => <div key={key}><dt>{text.counts[key]}：</dt><dd>{result.counts[key].toLocaleString(language)}</dd></div>)}
  </dl>;
}

function BackupActions({ backup, locale, beforeStart }: {
  readonly backup: ReturnType<typeof useBackupExport>; readonly locale: Locale; readonly beforeStart: (action: () => void) => void;
}) {
  const text = messages[locale];
  return <div className="actions"><button disabled={backup.active} onClick={() => beforeStart(() => { void backup.start(); })}>{text.create}</button>
    {activeBackup(backup.status) && <><button disabled={backup.cancelling} onClick={() => { void backup.refresh().catch(backup.failed); }}>{text.check}</button>
      <button disabled={backup.cancelling || backup.status?.cancel_requested} onClick={() => { void backup.cancel(); }}>{text.cancel}</button></>}</div>;
}

export function BackupSettings({ runtime, locale, beforeStart, onPause, onResolved, visible }: {
  readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly beforeStart: (action: () => void) => void;
  readonly onPause: (paused: boolean) => void; readonly visible: boolean;
  readonly onResolved: (batch: BatchStatus | null) => void;
}) {
  const text = messages[locale];
  const backup = useBackupExport(runtime, onResolved), { status, error } = backup;
  useEffect(() => onPause(backup.paused), [backup.paused, onPause]);
  return <section className="speech-settings backup-settings" aria-labelledby="backup-heading" hidden={!visible && !backup.paused}>
    <h3 id="backup-heading">{text.title}</h3>
    <p>{text.contents}</p><p>{text.resources}</p><p>{text.privacy}</p>
    {!runtime.backupAvailable() && <p>{text.preview}</p>}
    <fieldset className="backup-controls" disabled={!runtime.backupAvailable()}><BackupActions backup={backup} locale={locale} beforeStart={beforeStart} /></fieldset>
    <Progress status={status} preparing={backup.preparing} locale={locale} />
    {error && <p role="alert">{text.error}: {error}</p>}
    {status?.state === 'completed' && <><p role="status">{text.saved}</p><Result result={status.result} locale={locale} /></>}
    {status?.state === 'failed' && <p role="alert">{text.error}: {status.error}</p>}
    {status?.state === 'cancelled' && <p role="status">{text.cancelled}</p>}
  </section>;
}
