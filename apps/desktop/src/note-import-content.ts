import { dictionaryImportMessages } from './note-import-dictionary';

export const noteImportMessages = {
  zh: { dictionary: dictionaryImportMessages.zh, open: '导入单词 CSV', select: '选择 CSV 文件', title: '单词 CSV 导入预览', columns: '规范列：headword,notes,example_fr,example_zh。一个词头，可有多行例句。',
    confirm: '确认导入', cancel: '取消导入', cancelled: '已取消，未保存任何记录。', busy: '正在处理 CSV…',
    create: '新建外部笔记', identical: '内容相同，将跳过', different: '内容有差异，本次不更新；请修改文件后重新预览。', invalid: 'CSV 内容无效',
    imported: '导入已保存。可打开记录，主动加入识义卡和听音卡。', skipped: '内容相同，已有记录保持不变。', rows: '文件行', incoming: '文件内容', current: '已有内容',
    source: '导入来源', missing: '尚未关联词典，暂无词汇摘要。', external: '外部笔记', error: 'CSV 导入失败', openRecord: '打开记录', association: '例句尚未关联词典义项；可以保存后继续整理。' },
  en: { dictionary: dictionaryImportMessages.en, open: 'Import single-word CSV', select: 'Choose CSV file', title: 'Single-word CSV import preview', columns: 'Canonical columns: headword,notes,example_fr,example_zh. One headword with ordered example rows.',
    confirm: 'Confirm import', cancel: 'Cancel import', cancelled: 'Cancelled. No records saved.', busy: 'Processing CSV…',
    create: 'Create external note', identical: 'Identical content; skip', different: 'Content differs. This import cannot update it; edit the file and preview again.', invalid: 'Invalid CSV content',
    imported: 'Import saved. Open the record to add recognition and listening cards explicitly.', skipped: 'Identical content. Existing record unchanged.', rows: 'File rows', incoming: 'File content', current: 'Saved content',
    source: 'Import source', missing: 'No dictionary link or vocabulary summary yet.', external: 'External note', error: 'CSV import failed', openRecord: 'Open record', association: 'Examples have no dictionary meaning assigned; you can save and organize them later.' },
  fr: { dictionary: dictionaryImportMessages.fr, open: 'Importer un CSV pour un mot', select: 'Choisir le fichier CSV', title: 'Aperçu de l’import CSV pour un mot', columns: 'Colonnes : headword,notes,example_fr,example_zh. Un mot avec des exemples ordonnés.',
    confirm: 'Confirmer l’import', cancel: 'Annuler l’import', cancelled: 'Import annulé. Aucun enregistrement sauvegardé.', busy: 'Traitement du CSV…',
    create: 'Créer une note externe', identical: 'Contenu identique ; ignorer', different: 'Le contenu diffère. Cet import ne le modifie pas ; corrigez le fichier puis relancez l’aperçu.', invalid: 'Contenu CSV invalide',
    imported: 'Import sauvegardé. Ouvrez la note pour ajouter les cartes de reconnaissance et d’écoute.', skipped: 'Contenu identique. La note existante est conservée.', rows: 'Lignes du fichier', incoming: 'Contenu du fichier', current: 'Contenu sauvegardé',
    source: 'Source de l’import', missing: 'Aucun lien ni résumé de dictionnaire pour le moment.', external: 'Note externe', error: 'Échec de l’import CSV', openRecord: 'Ouvrir la note', association: 'Exemples sans sens de dictionnaire associé ; vous pouvez les sauvegarder et les organiser plus tard.' },
} as const;
