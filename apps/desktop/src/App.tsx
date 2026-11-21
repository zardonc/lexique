import { ExampleSpeechGroup } from './example-speech';
import { useEffect, useState } from 'react';
import { messages } from './content';
import type { Locale, Messages, Theme } from './content';
import { Navigation } from './navigation';
import { DictionaryPanel, RecordOriginal } from './dictionary';
import { RuntimeStatusPanel } from './runtime-status';
import type { RuntimeAdapter } from './platform/runtime';
import { useNotebook } from './use-notebook';
import { NotebookPanel } from './notebook';
import { VocabularyEditor } from './vocabulary-view';
import { notebookMessages } from './notebook-content';
import { LeaveDialog, useLeaveGuard } from './leave-guard';
import { SpeechSettings } from './speech-settings';
import { ReviewSettingsPanel } from './review-settings';
import { ReviewQueuePanel } from './review-queue';
import { useBatchReview } from './use-batch-review';
import { BatchProgress } from './batch-progress';
import { ConjugationPanel } from './conjugation';
import { useLookupNavigation } from './conjugation-navigation';
import { LookupInputPanel } from './lookup-input';
import { useSelectionInput } from './use-selection-input';
import { SelectionSettings } from './selection-settings';
import { lookupInputMessages } from './lookup-input-content';
import { DictionarySettings } from './dictionary-settings';
import { DictionaryResources } from './dictionary-resources';
import { useInterfacePreferences } from './use-interface-preferences';
import { BackupSettings } from './backup-settings';
import { StartupFeedback, StartupScreen, useStartup } from './startup';
import type { StartupStatus } from './platform/backup';
import { RestoreScreen, useFormalRestore } from './backup-restore';
import type { FormalRestore } from './backup-restore';
import { useGrammarNotebook } from './use-grammar-notebook';
import { GrammarEditor } from './grammar-editor';
import { useLearningCategories } from './learning-categories';
import type { NotebookMutation } from './learning-categories';
import { errorMessage } from './platform/vocabulary';

const themes: readonly Theme[] = ['white', 'sepia', 'dark'];
const locales: readonly Locale[] = ['zh', 'en', 'fr'];
const localeNames = ['中文', 'English', 'Français'] as const;
const languageTags = { zh: 'zh-CN', en: 'en', fr: 'fr' } as const;

interface HeaderProps {
  readonly text: Messages;
  readonly preferences: ReturnType<typeof useInterfacePreferences>;
  readonly onLocal: () => void;
  readonly selectionError: string;
}

function Header({ text, preferences, onLocal, selectionError }: HeaderProps) {
  const { locale, theme } = preferences.value;
  const description = preferences.message || preferences.error ? 'interface-preferences-state' : undefined;
  return <header className="topbar"><span className="preview-label">{text.preview}</span><div className="preferences">
    <button type="button" onMouseDown={(event) => event.preventDefault()} onClick={onLocal}>{lookupInputMessages[locale].local}</button>
    <select aria-label={text.language} aria-describedby={description} value={locale} disabled={preferences.disabled} onChange={(event) => {
      const value = locales.find((candidate) => candidate === event.target.value);
      if (value) void preferences.choose({ ...preferences.value, locale: value });
    }}>{locales.map((value, index) => <option key={value} value={value}>{localeNames[index]}</option>)}</select>
    <select aria-label={text.theme} aria-describedby={description} value={theme} disabled={preferences.disabled} onChange={(event) => {
      const value = themes.find((candidate) => candidate === event.target.value);
      if (value) void preferences.choose({ ...preferences.value, theme: value });
    }}>{themes.map((value, index) => <option key={value} value={value}>{text.themes[index]}</option>)}</select>
  </div>{(preferences.message || preferences.error) && <div className="interface-preferences-state" id="interface-preferences-state">
    <p aria-live="polite">{preferences.message}</p>
    {preferences.error && <div role="alert">{preferences.text.failed}: {preferences.error}
      <button disabled={preferences.busy} onClick={preferences.retry}>{preferences.text.retry}</button></div>}
  </div>}{selectionError && <p role="alert">{selectionError}</p>}</header>;
}

function PageIntroduction({ page, text }: { readonly page: number; readonly text: Messages }) {
  if (page === 3 || page === 1) return null;
  return <><span className="page-index" aria-hidden="true">{String(page + 1).padStart(2, '0')}</span>
    <h2 id="page-title">{page === 7 ? text.settingsTitle : text.title}</h2><p className="intro">{page === 7 ? text.settingsIntro : text.intro}</p></>;
}

function pageTitleId(page: number) { return page === 3 || page === 1 ? 'main-page-heading' : 'page-title'; }

function SettingsPanels({ runtime, locale }: { readonly runtime: RuntimeAdapter; readonly locale: Locale }) {
  return <><ReviewSettingsPanel runtime={runtime} locale={locale} /><SpeechSettings runtime={runtime} locale={locale} />
    <SelectionSettings runtime={runtime} locale={locale} /><SelectionSettings runtime={runtime} locale={locale} ocr />
    <DictionarySettings runtime={runtime} locale={locale} /><DictionaryResources runtime={runtime} locale={locale} /></>;
}

export function App({ runtime }: { readonly runtime: RuntimeAdapter }) {
  const startup = useStartup(runtime), restore = useFormalRestore(runtime, startup);
  if (restore.active) return <RestoreScreen runtime={runtime} restore={restore} startup={startup} />;
  return startup.status?.state === 'ready' && !startup.error && !startup.pending ? <LearningApp runtime={runtime} startup={startup.status} restore={restore} /> : <StartupScreen runtime={runtime} startup={startup} restore={restore} />;
}

function useLearningState(runtime: RuntimeAdapter) {
  const preferences = useInterfacePreferences(runtime);
  const { locale, theme } = preferences.value;
  const [collapsed, setCollapsed] = useState(false), [backupPaused, setBackupPaused] = useState(false);
  const [notesBusy, setNotesBusy] = useState(false), [categoryVersion, setCategoryVersion] = useState(0), [mutationError, setMutationError] = useState('');
  const batch = useBatchReview(runtime);
  const [reviewVersion, setReviewVersion] = useState(0);
  const reviewChanged = () => setReviewVersion((value) => value + 1);
  const text = messages[locale], notebookText = notebookMessages[locale];
  const notebook = useNotebook(runtime);
  const grammar = useGrammarNotebook(runtime), categories = useLearningCategories(runtime, categoryVersion);
  const clearEditors = () => { notebook.clear(); grammar.clear(); };
  const editor = { ...(grammar.editor ? grammar : notebook), busy: notebook.busy || grammar.busy, clear: clearEditors };
  const guard = useLeaveGuard(runtime, editor, batch.working || notesBusy);
  const scopeChange = (action: () => void) => guard.request(() => { clearEditors(); action(); });
  const navigation = useLookupNavigation(scopeChange);
  const mutate: NotebookMutation = (action) => {
    const refreshEditor = !editor.dirty;
    guard.request(() => {
      setNotesBusy(true); setMutationError('');
      void action().then(async (changed) => {
        if (!changed) return;
        setCategoryVersion((value) => value + 1); reviewChanged();
        if (refreshEditor) await Promise.all([notebook.refresh(), grammar.refresh()]);
      }).catch((cause: unknown) => setMutationError(errorMessage(cause))).finally(() => setNotesBusy(false));
    });
  };
  const page = navigation.page;
  const selection = useSelectionInput(runtime, { page, locale }, navigation.openInput);
  useEffect(() => { document.documentElement.lang = languageTags[locale]; }, [locale]);
  return { preferences, locale, theme, collapsed, backupPaused, notesBusy, mutationError, reviewVersion, reviewChanged, text, notebookText,
    notebook, grammar, categories, editor, guard, navigation, page, selection, mutate, scopeChange, batch, setBackupPaused,
    toggleCollapsed: () => setCollapsed((value) => !value) };
}

function LearningPage({ runtime, state }: { readonly runtime: RuntimeAdapter; readonly state: ReturnType<typeof useLearningState> }) {
  const { page, locale, text, notebookText, notebook, grammar, categories, navigation, selection, mutate, scopeChange, guard, reviewVersion, batch } = state;
  if (page === 0) return <DictionaryPanel runtime={runtime} text={text.dictionary} notebookText={notebookText} version={notebook.version}
    preserveExcerpt={notebook.preservesExcerpt} locale={locale} initial={navigation.dictionary} onConjugation={navigation.openConjugation}
    onExcerpt={(result, record) => { const start = () => { grammar.clear(); notebook.beginExcerpt(result, record); }; if (notebook.editor?.snapshot.headword === result.entries[0]?.key && !notebook.excerpt && !notebook.busy) start(); else guard.request(start); }} onCollect={(result) => scopeChange(() => notebook.collect(result))}
    onOpen={(record) => scopeChange(() => notebook.open(record))} beforeSearch={navigation.beforeSearch} />;
  if (page === 1) return <ConjugationPanel runtime={runtime} locale={locale} initial={navigation.conjugation} onChange={navigation.setConjugation} onDictionary={navigation.openDictionary} />;
  if (page === 2) return <NotebookPanel runtime={runtime} version={notebook.version + grammar.version + reviewVersion} text={notebookText} batch={batch}
    categories={categories} onMutation={mutate} onScopeChange={scopeChange} onCreateGrammar={() => scopeChange(grammar.create)}
    onOpen={(record) => scopeChange(() => { if ('title' in record) grammar.open(record); else notebook.open(record); })} />;
  if (page === 3) return <ReviewQueuePanel runtime={runtime} locale={locale} text={notebookText} />;
  if (page === 5) return <LookupInputPanel runtime={runtime} locale={locale} selection={selection.input} onCancel={selection.cancel} onDictionary={navigation.openDictionaryResult} />;
  if (page === 7) return <SettingsPanels runtime={runtime} locale={locale} />;
  return <div className="empty-state"><p>{text.pending[page]}</p><small>{text.scaffold}</small></div>;
}

function LearningApp({ runtime, startup, restore }: { readonly runtime: RuntimeAdapter; readonly startup: StartupStatus; readonly restore: FormalRestore }) {
  const state = useLearningState(runtime);
  const { preferences, locale, theme, collapsed, backupPaused, notesBusy, text, notebookText, notebook, grammar, categories,
    editor, guard, navigation, page, selection, reviewChanged, mutationError, scopeChange, batch } = state;
  return <ExampleSpeechGroup><div className="app" data-theme={theme} data-collapsed={collapsed}>
    <a className="skip-link" href="#main-content">{text.skip}</a>
    <Navigation text={text} selected={page} collapsed={collapsed} disabled={backupPaused || notesBusy} onSelect={navigation.changePage} onToggle={state.toggleCollapsed} />
    <div className="workspace"><fieldset className="backup-controls" disabled={backupPaused}><Header text={text} preferences={preferences} onLocal={selection.local} selectionError={selection.error} /></fieldset>
      <main id="main-content" tabIndex={-1}>
        <p className="eyebrow">{text.eyebrow}</p><h1 id="main-page-heading">{text.modules[page]}</h1><StartupFeedback runtime={runtime} status={startup} locale={locale} />
        <section className="page-content" aria-labelledby={pageTitleId(page)}>
          <BackupSettings runtime={runtime} locale={locale} beforeStart={scopeChange} onPause={state.setBackupPaused} onResolved={batch.confirmPending} visible={page === 7} restore={restore} />
          <fieldset className="backup-controls" disabled={backupPaused || notesBusy}>
          <PageIntroduction page={page} text={text} />
          <LearningPage runtime={runtime} state={state} />
          <VocabularyEditor notebook={notebook} categories={categories} text={notebookText} runtime={runtime} onReviewChanged={reviewChanged} viewExisting={() => {
            if (notebook.existing) { const record = notebook.existing; scopeChange(() => notebook.open(record)); }
          }} />
          <GrammarEditor notebook={grammar} categories={categories} runtime={runtime} text={notebookText} onChanged={reviewChanged} />
          {page === 2 && notebook.editor?.snapshot.reference && <RecordOriginal reference={notebook.editor.snapshot.reference}
            runtime={runtime} text={text.dictionary} title={notebookText.original} />}
          {guard.error && <p role="alert">{notebookText.closeError}: {guard.error}</p>}
          {mutationError && <p role="alert">{notebookText.notesText.categoryError}: {mutationError}</p>}
          <BatchProgress batch={batch} text={notebookText} /></fieldset>
        </section>
        <RuntimeStatusPanel runtime={runtime} text={text} />
      </main>
    </div>
    <LeaveDialog guard={guard} notebook={editor} text={notebookText} />
  </div></ExampleSpeechGroup>;
}
