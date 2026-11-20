import type { NoteEnrichment } from './platform/note-enrichment';
import type { VocabularyRecord } from './platform/vocabulary';

export const dictionaryImportMessages = {
  zh: { linked: '已精确关联本地词典', not_found: '已安装词典未收录，保存为外部笔记。', error: '词典查询未完成，尚未关联词典。', retained: '保留已有词典信息', continue: '继续保存这条未关联词典的笔记' },
  en: { linked: 'Exact local dictionary match', not_found: 'Not found in installed dictionaries; save as an external note.', error: 'Dictionary lookup could not complete; no dictionary link.', retained: 'Keep saved dictionary information', continue: 'Continue saving this note without a dictionary link' },
  fr: { linked: 'Correspondance exacte dans un dictionnaire local', not_found: 'Absent des dictionnaires installés ; sauvegarder comme note externe.', error: 'Recherche incomplète ; aucun lien de dictionnaire.', retained: 'Conserver les informations du dictionnaire enregistrées', continue: 'Continuer à sauvegarder cette note sans lien de dictionnaire' },
} as const;

export function NoteImportDictionary({ enrichment, existing, messages, allowUnlinked, onContinue, disabled }: {
  readonly enrichment: NoteEnrichment; readonly existing: VocabularyRecord | null;
  readonly messages: typeof dictionaryImportMessages[keyof typeof dictionaryImportMessages];
  readonly allowUnlinked: boolean; readonly onContinue: (value: boolean) => void; readonly disabled: boolean;
}) {
  const reference = enrichment.snapshot?.reference ?? existing?.reference;
  return <div>
    <p>{messages[enrichment.status]}{reference && ` · ${reference.dictionary} · ${reference.version}`}</p>
    {enrichment.warnings.map((warning, index) => <p role="alert" key={index}>{warning.dictionary_id}: {warning.code} — {warning.message}</p>)}
    {enrichment.status === 'error' && <label><input type="checkbox" checked={allowUnlinked} disabled={disabled}
      onChange={(event) => onContinue(event.target.checked)} />{messages.continue}</label>}
  </div>;
}
