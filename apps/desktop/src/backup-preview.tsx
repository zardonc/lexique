import { useEffect, useRef, useState } from 'react';
import type { Locale } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import type { BackupCounts, BackupPreview, BackupPreviewStatus } from './platform/backup';
import { errorMessage } from './platform/vocabulary';

const messages = {
  zh: { title: '检查备份', choose: '选择备份', ready: '备份已验证，当前学习数据未改变。', failed: '检查备份失败',
    check: '检查预览状态', cancel: '取消检查', cancelled: '备份检查已取消。', cancelling: '正在取消备份检查…', retry: '重试',
    choosing: '请选择要检查的备份文件。', phases: { validating: '正在检查备份内容，可继续学习。', staging: '正在准备隔离学习数据和音频，可继续学习。' },
    working: '正在打开备份选择器…', desktop: '备份检查仅在桌面应用中可用。',
    contents: '选择备份，在隔离区域完整检查并准备学习数据和音频。检查不会改变当前学习数据。',
    resources: '词典文件和模型需在目标设备另行提供。缺少资源不会抹掉笔记和卡片进度。',
    restart: '恢复需要重启。', filename: '备份文件', size: '文件大小', bytes: '字节', created: '创建时间', version: '来源应用版本', pending: '待补齐词典' },
  en: { title: 'Check a backup', choose: 'Choose backup', ready: 'Backup verified. Current learning data is unchanged.', failed: 'Backup check failed',
    check: 'Check preview status', cancel: 'Cancel check', cancelled: 'Backup check cancelled.', cancelling: 'Cancelling the backup check…', retry: 'Retry',
    choosing: 'Choose the backup file to check.', phases: { validating: 'Checking the backup contents. You can continue learning.', staging: 'Preparing isolated learning data and audio. You can continue learning.' },
    working: 'Opening the backup picker…', desktop: 'Backup checks are available in the desktop app only.',
    contents: 'Choose a backup to fully check and prepare its learning data and audio in an isolated area. Checking leaves current learning data unchanged.',
    resources: 'Provide dictionary files and models separately on the destination device. Missing resources preserve notes and card progress.',
    restart: 'Restoring requires a restart.', filename: 'Backup file', size: 'File size', bytes: 'bytes', created: 'Created at', version: 'Source app version', pending: 'Dictionaries to provide' },
  fr: { title: 'Vérifier une sauvegarde', choose: 'Choisir une sauvegarde', ready: 'Sauvegarde vérifiée. Les données actuelles sont conservées.', failed: 'Échec de la vérification',
    check: 'Vérifier l’état de la préparation', cancel: 'Annuler la vérification', cancelled: 'Vérification annulée.', cancelling: 'Annulation de la vérification…', retry: 'Réessayer',
    choosing: 'Choisissez le fichier à vérifier.', phases: { validating: 'Vérification du contenu. Vous pouvez continuer à étudier.', staging: 'Préparation des données et audios isolés. Vous pouvez continuer à étudier.' },
    working: 'Ouverture du sélecteur de sauvegarde…', desktop: 'La vérification est disponible uniquement dans l’application de bureau.',
    contents: 'Choisissez une sauvegarde pour vérifier et préparer ses données et audios dans un espace isolé. Les données actuelles sont conservées.',
    resources: 'Les fichiers de dictionnaire et les modèles doivent être fournis sur l’appareil cible. Les notes et progrès sont conservés en cas de ressource manquante.',
    restart: 'La restauration nécessite un redémarrage.', filename: 'Fichier de sauvegarde', size: 'Taille du fichier', bytes: 'octets', created: 'Date de création', version: 'Version de l’application source', pending: 'Dictionnaires à fournir' },
} as const;

function activePreview(status: BackupPreviewStatus | null) { return status?.state === 'choosing' || status?.state === 'running'; }
function canReadPreview(status: BackupPreviewStatus | null) { return activePreview(status) || status?.state === 'preview_ready'; }
function progressOrder(status: BackupPreviewStatus) { return status.state === 'running' ? status.phase === 'validating' ? 1 : 2 : status.state === 'choosing' ? 0 : 3; }
function regresses(previous: BackupPreviewStatus | null, value: BackupPreviewStatus) {
  if (previous?.operation !== value.operation) return false;
  if (previous.state === 'cancelled' || previous.state === 'failed') return value.state !== previous.state;
  return progressOrder(value) < progressOrder(previous);
}

function usePreviewStatus(runtime: RuntimeAdapter) {
  const [status, setStatus] = useState<BackupPreviewStatus | null>(null), [error, setError] = useState('');
  const current = useRef<BackupPreviewStatus | null>(null), updates = useRef(0), live = useRef(true);
  const unlisten = useRef<(() => void) | undefined>(undefined);
  function update(value: BackupPreviewStatus) {
    if (!live.current || regresses(current.current, value)) return;
    current.current = value; updates.current += 1; setStatus(value); setError('');
    if (value.state === 'cancelled' || value.state === 'failed') { unlisten.current?.(); unlisten.current = undefined; }
  }
  function receive(value: BackupPreviewStatus) {
    if (current.current?.operation === value.operation) update(value);
  }
  function reset() {
    unlisten.current?.(); unlisten.current = undefined;
    current.current = null; updates.current += 1; setStatus(null); setError('');
  }
  function failed(cause: unknown) { if (live.current) setError(errorMessage(cause)); }
  async function refresh() {
    const operation = current.current?.operation;
    if (!operation) return;
    const version = updates.current;
    try { const value = await runtime.backupPreviewStatus(operation); if (version === updates.current) update(value); }
    catch (cause) { if (version === updates.current) failed(cause); }
  }
  async function cancel() {
    const operation = current.current?.operation;
    if (!operation) return;
    const version = updates.current;
    try { const value = await runtime.cancelBackupPreview(operation); if (current.current?.operation === operation) update(value); }
    catch (cause) { if (version === updates.current) failed(cause); }
  }
  useEffect(() => {
    live.current = true;
    return () => { live.current = false; unlisten.current?.(); };
  }, []);
  return { status, error, current, live, unlisten, update, receive, reset, refresh, cancel, failed };
}

export function useBackupPreview(runtime: RuntimeAdapter) {
  const operation = usePreviewStatus(runtime);
  const [starting, setStarting] = useState(false), [cancelOperation, setCancelOperation] = useState<string | null>(null);
  const startPending = useRef(false), cancelPending = useRef<string | null>(null);
  const cancelling = cancelOperation !== null && cancelOperation === operation.status?.operation && canReadPreview(operation.status);
  async function start() {
    if (startPending.current || activePreview(operation.current.current) || !runtime.backupAvailable()) return;
    startPending.current = true; setStarting(true); operation.reset();
    try {
      const stop = await runtime.onBackupPreviewProgress(operation.receive, operation.failed);
      if (!operation.live.current) { stop(); return; }
      operation.unlisten.current = stop;
      const value = await runtime.startBackupPreview();
      operation.update(value);
      if (activePreview(value)) void operation.refresh();
    } catch (cause) {
      operation.failed(cause);
      if (!canReadPreview(operation.current.current)) { operation.unlisten.current?.(); operation.unlisten.current = undefined; }
    }
    finally { startPending.current = false; setStarting(false); }
  }
  async function cancel() {
    const id = operation.current.current?.operation;
    if (!id || !canReadPreview(operation.current.current) || cancelPending.current === id) return;
    cancelPending.current = id; setCancelOperation(id);
    try { await operation.cancel(); }
    finally { if (cancelPending.current === id) { cancelPending.current = null; setCancelOperation(null); } }
  }
  return { ...operation, start, cancel, starting, cancelling, pending: starting || cancelling, active: starting || cancelling || activePreview(operation.status),
    retry: () => canReadPreview(operation.current.current) ? operation.refresh() : start() };
}

function PreviewFacts({ preview, locale, countsText }: {
  readonly preview: BackupPreview; readonly locale: Locale; readonly countsText: Readonly<Record<keyof BackupCounts, string>>;
}) {
  const text = messages[locale], language = locale === 'zh' ? 'zh-CN' : locale;
  const keys = Object.keys(countsText) as (keyof BackupCounts)[];
  return <><dl className="backup-result">
    <div><dt>{text.filename}</dt><dd>{preview.filename}</dd></div>
    <div><dt>{text.size}</dt><dd>{preview.byte_length.toLocaleString(language)} {text.bytes}</dd></div>
    <div><dt>{text.created}</dt><dd>{new Date(preview.created_at * 1000).toLocaleString(language)}</dd></div>
    <div><dt>{text.version}</dt><dd>{preview.application_version}</dd></div>
    {keys.map((key) => <div key={key}><dt>{countsText[key]}：</dt><dd>{preview.counts[key].toLocaleString(language)}</dd></div>)}
  </dl>{preview.pending_dictionaries.length > 0 && <div><h4>{text.pending}</h4><ul>
    {preview.pending_dictionaries.map((dictionary) => <li key={dictionary.id}>{dictionary.title}</li>)}
  </ul></div>}<p>{text.restart}</p></>;
}

function PreviewProgress({ preview, locale }: { readonly preview: ReturnType<typeof useBackupPreview>; readonly locale: Locale }) {
  const text = messages[locale], { status } = preview;
  let label = preview.starting ? text.working : status?.state === 'choosing' ? text.choosing : status?.state === 'running' ? text.phases[status.phase] : '';
  if (preview.cancelling && label) label = text.cancelling;
  return label ? <><p role="status">{label}</p><progress aria-label={label} /></> : null;
}

function PreviewFeedback({ preview, locale, visible, blocked }: {
  readonly preview: ReturnType<typeof useBackupPreview>; readonly locale: Locale; readonly visible: boolean; readonly blocked: boolean;
}) {
  const text = messages[locale], { status } = preview, state = status?.state;
  const failure = preview.error || (status?.state === 'failed' ? status.error : '');
  const feedback = useRef<HTMLParagraphElement | null>(null);
  useEffect(() => {
    if (visible && (failure || state === 'preview_ready' || state === 'cancelled')) feedback.current?.focus();
  }, [visible, failure, state]);
  if (failure) return <p role="alert" tabIndex={-1} ref={feedback}>{text.failed}: {failure} <button disabled={blocked || preview.pending} onClick={() => { void preview.retry(); }}>{text.retry}</button></p>;
  if (state === 'preview_ready') return <p role="status" tabIndex={-1} ref={feedback}>{preview.cancelling ? text.cancelling : text.ready}</p>;
  if (state === 'cancelled') return <p role="status" tabIndex={-1} ref={feedback}>{text.cancelled}</p>;
  return null;
}

function PreviewActions({ preview, runtime, locale, blocked }: {
  readonly preview: ReturnType<typeof useBackupPreview>; readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly blocked: boolean;
}) {
  const text = messages[locale];
  return <div className="actions"><button disabled={preview.active || blocked || !runtime.backupAvailable()} onClick={() => { if (!blocked) void preview.start(); }}>{text.choose}</button>
    {preview.status && <button disabled={preview.cancelling} onClick={() => { void preview.refresh(); }}>{text.check}</button>}
    {canReadPreview(preview.status) && <button disabled={preview.cancelling} onClick={() => { void preview.cancel(); }}>{text.cancel}</button>}
  </div>;
}

export function BackupPreviewSettings({ runtime, preview, locale, countsText, visible, blocked }: {
  readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly countsText: Readonly<Record<keyof BackupCounts, string>>;
  readonly preview: ReturnType<typeof useBackupPreview>;
  readonly visible: boolean; readonly blocked: boolean;
}) {
  const text = messages[locale];
  const { status } = preview;
  return <section className="speech-settings backup-settings" aria-labelledby="backup-preview-heading" hidden={!visible && !preview.active}>
    <h3 id="backup-preview-heading">{text.title}</h3><p>{text.contents}</p><p>{text.resources}</p>
    {!runtime.backupAvailable() && <p>{text.desktop}</p>}
    <PreviewActions preview={preview} runtime={runtime} locale={locale} blocked={blocked} />
    <PreviewProgress preview={preview} locale={locale} />
    <PreviewFeedback preview={preview} locale={locale} visible={visible} blocked={blocked} />
    {status?.state === 'preview_ready' && <PreviewFacts preview={status.preview} locale={locale} countsText={countsText} />}
  </section>;
}
