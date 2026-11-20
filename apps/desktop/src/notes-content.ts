import type { Locale } from './content';

export const notesMessages = {
  zh: { vocabulary: '生词', grammar: '语法', tabs: '笔记类型', create: '新建语法笔记', title: '语法标题', body: '语法正文', search: '搜索语法标题或正文', empty: '没有匹配的语法笔记。',
    sorts: ['标题正序', '标题倒序', '创建时间正序', '创建时间倒序', '修改时间正序', '修改时间倒序', '下次复习时间'],
    save: '保存笔记', makeCard: '制作问答卡', saveFirst: '先保存笔记，再制作问答卡。', question: '题面', answer: '答案', card: '语法问答卡',
    enroll: '保存并加入复习', saveCard: '保存问答卡', independent: '问答卡独立保存；修改笔记不会覆盖题面、答案或复习进度。', nameLimit: '请输入 1–200 字符的单行标题或名称。', answerRequired: '题面和答案均须填写，且各不超过 20,000 字符。',
    delete: '删除语法笔记', deleteHeading: '永久删除语法笔记？', deleteDetail: '将删除这条笔记、分类归属、关联问答卡、学习进度和评分历史。已消耗的当日额度保留。', confirmDelete: '确认删除', cancel: '取消',
    categories: '学习分类', filter: '筛选分类', all: '全部分类', uncategorized: '未分类', picker: '记录分类', none: '尚无分类，可在笔记页创建。',
    manage: '管理分类', name: '新分类名称', createCategory: '创建分类', rename: '重命名', renameLabel: '分类名称', deleteCategory: '删除分类', deleteCategoryHeading: '删除这个分类？',
    deleteCategoryDetail: '只解除分类归属，笔记、卡片和学习进度全部保留。', batchCategory: '批量整理分类', batchAdd: '添加分类', batchRemove: '移除分类', categoryError: '分类操作失败', retry: '重试分类', results: '分类整理结果',
  },
  en: { vocabulary: 'Vocabulary', grammar: 'Grammar', tabs: 'Note type', create: 'New grammar note', title: 'Grammar title', body: 'Grammar body', search: 'Search grammar titles or bodies', empty: 'No matching grammar notes.',
    sorts: ['Title ascending', 'Title descending', 'Created ascending', 'Created descending', 'Updated ascending', 'Updated descending', 'Next review'],
    save: 'Save note', makeCard: 'Make question-answer card', saveFirst: 'Save the note before making a question-answer card.', question: 'Question', answer: 'Answer', card: 'Grammar question-answer card',
    enroll: 'Save and add to review', saveCard: 'Save question-answer card', independent: 'The card is saved separately. Note edits keep its question, answer and review progress.', nameLimit: 'Enter a single-line title or name of 1–200 characters.', answerRequired: 'Question and answer are required, with at most 20,000 characters each.',
    delete: 'Delete grammar note', deleteHeading: 'Permanently delete grammar note?', deleteDetail: 'Delete this note, category memberships, card, progress and rating history. Today’s consumed quota is retained.', confirmDelete: 'Confirm deletion', cancel: 'Cancel',
    categories: 'Learning categories', filter: 'Filter category', all: 'All categories', uncategorized: 'Uncategorized', picker: 'Record categories', none: 'No categories yet. Create them on the notes page.',
    manage: 'Manage categories', name: 'New category name', createCategory: 'Create category', rename: 'Rename', renameLabel: 'Category name', deleteCategory: 'Delete category', deleteCategoryHeading: 'Delete this category?',
    deleteCategoryDetail: 'Remove memberships only. Keep all notes, cards and learning progress.', batchCategory: 'Classify selected notes', batchAdd: 'Add category', batchRemove: 'Remove category', categoryError: 'Category operation failed', retry: 'Retry classification', results: 'Classification results',
  },
  fr: { vocabulary: 'Vocabulaire', grammar: 'Grammaire', tabs: 'Type de note', create: 'Nouvelle note de grammaire', title: 'Titre de grammaire', body: 'Texte de grammaire', search: 'Rechercher dans les titres ou textes', empty: 'Aucune note de grammaire correspondante.',
    sorts: ['Titre croissant', 'Titre décroissant', 'Création croissante', 'Création décroissante', 'Modification croissante', 'Modification décroissante', 'Prochaine révision'],
    save: 'Enregistrer la note', makeCard: 'Créer une carte question-réponse', saveFirst: 'Enregistrez la note avant de créer sa carte.', question: 'Question', answer: 'Réponse', card: 'Carte de grammaire',
    enroll: 'Enregistrer et ajouter aux révisions', saveCard: 'Enregistrer la carte', independent: 'La carte est enregistrée séparément. Modifier la note conserve sa question, sa réponse et ses progrès.', nameLimit: 'Saisissez un titre ou nom de 1–200 caractères sur une ligne.', answerRequired: 'Question et réponse obligatoires, 20 000 caractères maximum chacune.',
    delete: 'Supprimer la note de grammaire', deleteHeading: 'Supprimer définitivement cette note ?', deleteDetail: 'Supprimer cette note, ses catégories, sa carte, ses progrès et son historique. Le quota déjà utilisé aujourd’hui est conservé.', confirmDelete: 'Confirmer la suppression', cancel: 'Annuler',
    categories: 'Catégories d’étude', filter: 'Filtrer par catégorie', all: 'Toutes les catégories', uncategorized: 'Sans catégorie', picker: 'Catégories de la note', none: 'Aucune catégorie. Créez-les sur la page des notes.',
    manage: 'Gérer les catégories', name: 'Nom de la nouvelle catégorie', createCategory: 'Créer la catégorie', rename: 'Renommer', renameLabel: 'Nom de la catégorie', deleteCategory: 'Supprimer la catégorie', deleteCategoryHeading: 'Supprimer cette catégorie ?',
    deleteCategoryDetail: 'Retirer uniquement les appartenances. Conserver les notes, cartes et progrès.', batchCategory: 'Classer les notes sélectionnées', batchAdd: 'Ajouter la catégorie', batchRemove: 'Retirer la catégorie', categoryError: 'Échec de l’opération', retry: 'Réessayer le classement', results: 'Résultats du classement',
  },
} as const;

export type NotesText = (typeof notesMessages)[Locale];
