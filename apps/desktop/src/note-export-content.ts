export const noteExportMessages = {
  zh: { open: '导出 CSV', heading: '导出笔记', scope: '导出范围', all: '全部生词', search: '当前搜索结果', single: '当前生词',
    records: '条生词', rows: '行例句', empty: '此范围没有已保存的生词，无法导出。', loading: '正在读取已保存的笔记…',
    openMarkdown: '导出 Markdown', saveMarkdown: '保存 Markdown', save: '保存 CSV', busy: '正在导出…', saved: '已导出', cancelled: '已取消，未导出文件。', close: '关闭', retry: '重新读取',
    saveZip: '保存 Markdown ZIP', detail: '仅导出已保存的词头、笔记和全部例句。' },
  en: { open: 'Export CSV', heading: 'Export notes', scope: 'Export scope', all: 'All vocabulary', search: 'Current search results', single: 'Current vocabulary',
    records: 'vocabulary records', rows: 'example rows', empty: 'No saved vocabulary in this scope to export.', loading: 'Reading saved notes…',
    openMarkdown: 'Export Markdown', saveMarkdown: 'Save Markdown', save: 'Save CSV', busy: 'Exporting…', saved: 'Exported', cancelled: 'Cancelled; no file exported.', close: 'Close', retry: 'Read again',
    saveZip: 'Save Markdown ZIP', detail: 'Exports saved headwords, notes and all examples.' },
  fr: { open: 'Exporter en CSV', heading: 'Exporter les notes', scope: 'Portée de l’export', all: 'Tout le vocabulaire', search: 'Résultats de recherche', single: 'Mot actuel',
    records: 'fiches de vocabulaire', rows: 'lignes d’exemples', empty: 'Aucun mot enregistré à exporter dans cette sélection.', loading: 'Lecture des notes enregistrées…',
    openMarkdown: 'Exporter en Markdown', saveMarkdown: 'Enregistrer le Markdown', save: 'Enregistrer le CSV', busy: 'Export en cours…', saved: 'Exporté', cancelled: 'Annulé ; aucun fichier exporté.', close: 'Fermer', retry: 'Relire',
    saveZip: 'Enregistrer le ZIP Markdown', detail: 'Exporte les mots, notes et tous les exemples enregistrés.' },
};
export type NoteExportText = (typeof noteExportMessages)[keyof typeof noteExportMessages];
