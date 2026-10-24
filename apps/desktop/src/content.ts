export type Locale = 'zh' | 'en' | 'fr';
export type Theme = 'white' | 'sepia' | 'dark';

export const messages = {
  zh: {
    subtitle: '法语词典与复习', preview: '开发预览', language: '界面语言', theme: '显示主题',
    themes: ['纯白', '暖色', '深色'], collapse: '收起导航', expand: '展开导航',
    skip: '跳到主要内容', eyebrow: '你的法语学习空间',
    title: '从一个词开始。', intro: '查词、记录与复习，留在同一个安静的空间里。',
    statusLoading: '正在检查本地存储…', statusReady: '本地存储已就绪。',
    statusPreview: '浏览器预览：本地数据功能仅在桌面应用可用。', retry: '重试',
    scaffold: '当前为开发骨架；学习功能将按模块逐步接入。',
    modules: ['词典', '动词变位', '生词本', '闪卡复习', '主题词汇', '翻译', '练习', '设置'],
    pending: ['词典资源尚未接入。', '离线变位数据尚未接入。', '笔记与分类功能尚未接入。',
      'FSRS 调度尚未接入。', '分级与主题词库尚未接入。', '翻译与 OpenAI 兼容服务尚未接入。',
      '练习将在后续阶段接入。', '偏好、额外资源及服务凭据将在此管理。'],
  },
  en: {
    subtitle: 'French dictionary & review', preview: 'Development preview', language: 'Interface language', theme: 'Display theme',
    themes: ['White', 'Warm', 'Dark'], collapse: 'Collapse navigation', expand: 'Expand navigation',
    skip: 'Skip to main content', eyebrow: 'A space for your French',
    title: 'Start with a word.', intro: 'Look up, collect and review in one quiet space.',
    statusLoading: 'Checking local storage…', statusReady: 'Local storage is ready.',
    statusPreview: 'Browser preview: local data is available in the desktop app only.', retry: 'Retry',
    scaffold: 'This is a development scaffold. Learning features will be connected module by module.',
    modules: ['Dictionary', 'Conjugation', 'Notebook', 'Flashcards', 'Topics', 'Translation', 'Practice', 'Settings'],
    pending: ['Dictionary resources are not connected yet.', 'Offline conjugation data is not connected yet.',
      'Notes and categories are not connected yet.', 'FSRS scheduling is not connected yet.',
      'Level and topic lists are not connected yet.', 'Translation and OpenAI-compatible services are not connected yet.',
      'Practice is planned for a later stage.', 'Preferences, extra resources and service credentials will be managed here.'],
  },
  fr: {
    subtitle: 'Dictionnaire et révision', preview: 'Aperçu de développement', language: 'Langue de l’interface', theme: 'Thème d’affichage',
    themes: ['Blanc', 'Chaud', 'Sombre'], collapse: 'Réduire la navigation', expand: 'Développer la navigation',
    skip: 'Aller au contenu principal', eyebrow: 'Un espace pour votre français',
    title: 'Tout commence par un mot.', intro: 'Chercher, noter et réviser dans un même espace calme.',
    statusLoading: 'Vérification du stockage local…', statusReady: 'Le stockage local est prêt.',
    statusPreview: 'Aperçu web : les données locales sont disponibles uniquement dans l’application de bureau.', retry: 'Réessayer',
    scaffold: 'Il s’agit du socle de développement. Les fonctions d’apprentissage seront ajoutées progressivement.',
    modules: ['Dictionnaire', 'Conjugaison', 'Carnet', 'Cartes mémoire', 'Thèmes', 'Traduction', 'Exercices', 'Réglages'],
    pending: ['Les dictionnaires ne sont pas encore intégrés.', 'Les conjugaisons hors ligne ne sont pas encore intégrées.',
      'Les notes et catégories ne sont pas encore intégrées.', 'La planification FSRS n’est pas encore intégrée.',
      'Les listes par niveau et par thème ne sont pas encore intégrées.', 'La traduction et les services compatibles OpenAI ne sont pas encore intégrés.',
      'Les exercices sont prévus pour une étape ultérieure.', 'Les préférences, ressources et identifiants seront gérés ici.'],
  },
} as const;

export type Messages = (typeof messages)[Locale];
