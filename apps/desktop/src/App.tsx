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

const themes: readonly Theme[] = ['white', 'sepia', 'dark'];
const locales: readonly Locale[] = ['zh', 'en', 'fr'];
const localeNames = ['中文', 'English', 'Français'] as const;
const languageTags = { zh: 'zh-CN', en: 'en', fr: 'fr' } as const;

interface HeaderProps {
  readonly text: Messages;
  readonly locale: Locale;
  readonly theme: Theme;
  readonly onLocale: (locale: Locale) => void;
  readonly onTheme: (theme: Theme) => void;
}

function Header({ text, locale, theme, onLocale, onTheme }: HeaderProps) {
  return <header className="topbar"><span className="preview-label">{text.preview}</span><div className="preferences">
    <select aria-label={text.language} value={locale} onChange={(event) => {
      const value = locales.find((candidate) => candidate === event.target.value);
      if (value) onLocale(value);
    }}>{locales.map((value, index) => <option key={value} value={value}>{localeNames[index]}</option>)}</select>
    <select aria-label={text.theme} value={theme} onChange={(event) => {
      const value = themes.find((candidate) => candidate === event.target.value);
      if (value) onTheme(value);
    }}>{themes.map((value, index) => <option key={value} value={value}>{text.themes[index]}</option>)}</select>
  </div></header>;
}

export function App({ runtime }: { readonly runtime: RuntimeAdapter }) {
  const [locale, setLocale] = useState<Locale>('zh');
  const [theme, setTheme] = useState<Theme>('white');
  const [page, setPage] = useState(0);
  const [collapsed, setCollapsed] = useState(false);
  const text = messages[locale];
  const notebookText = notebookMessages[locale];
  const notebook = useNotebook(runtime);
  const guard = useLeaveGuard(runtime, notebook);
  function changePage(next: number) { guard.request(() => { notebook.clear(); setPage(next); }); }
  useEffect(() => { document.documentElement.lang = languageTags[locale]; }, [locale]);
  return <div className="app" data-theme={theme} data-collapsed={collapsed}>
    <a className="skip-link" href="#main-content">{text.skip}</a>
    <Navigation text={text} selected={page} collapsed={collapsed} onSelect={changePage} onToggle={() => setCollapsed((value) => !value)} />
    <div className="workspace"><Header text={text} locale={locale} theme={theme} onLocale={setLocale} onTheme={setTheme} />
      <main id="main-content" tabIndex={-1}>
        <p className="eyebrow">{text.eyebrow}</p><h1>{text.modules[page]}</h1>
        <section className="page-content" aria-labelledby="page-title">
          <span className="page-index" aria-hidden="true">{String(page + 1).padStart(2, '0')}</span>
          <h2 id="page-title">{page === 7 ? text.settingsTitle : text.title}</h2><p className="intro">{page === 7 ? text.settingsIntro : text.intro}</p>
          {page === 0 ? <DictionaryPanel runtime={runtime} text={text.dictionary} notebookText={notebookText} version={notebook.version}
            onCollect={(result) => guard.request(() => notebook.collect(result))}
            onOpen={(record) => guard.request(() => notebook.open(record))}
            beforeSearch={(action) => guard.request(() => { notebook.clear(); action(); })} /> : page === 2 ?
            <NotebookPanel runtime={runtime} version={notebook.version} text={notebookText}
              onOpen={(record) => guard.request(() => notebook.open(record))} /> : page === 3 ? <ReviewQueuePanel runtime={runtime} locale={locale} text={notebookText} /> : page === 7 ?
            <><ReviewSettingsPanel runtime={runtime} locale={locale} /><SpeechSettings runtime={runtime} locale={locale} /></> :
            <div className="empty-state"><p>{text.pending[page]}</p><small>{text.scaffold}</small></div>}
          <VocabularyEditor notebook={notebook} text={notebookText} runtime={runtime} viewExisting={() => {
            if (notebook.existing) { const record = notebook.existing; guard.request(() => notebook.open(record)); }
          }} />
          {page === 2 && notebook.editor && <RecordOriginal headword={notebook.editor.snapshot.reference.lookup_key}
            runtime={runtime} text={text.dictionary} title={notebookText.original} />}
          {guard.error && <p role="alert">{notebookText.closeError}: {guard.error}</p>}
        </section>
        <RuntimeStatusPanel runtime={runtime} text={text} />
      </main>
    </div>
    <LeaveDialog guard={guard} notebook={notebook} text={notebookText} />
  </div>;
}
