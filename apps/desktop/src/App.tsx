import { useEffect, useState } from 'react';
import { messages } from './content';
import type { Locale, Messages, Theme } from './content';
import { Navigation } from './navigation';
import { RuntimeStatusPanel } from './runtime-status';
import type { RuntimeAdapter } from './platform/runtime';

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
  useEffect(() => { document.documentElement.lang = languageTags[locale]; }, [locale]);
  return <div className="app" data-theme={theme} data-collapsed={collapsed}>
    <a className="skip-link" href="#main-content">{text.skip}</a>
    <Navigation text={text} selected={page} collapsed={collapsed} onSelect={setPage} onToggle={() => setCollapsed((value) => !value)} />
    <div className="workspace"><Header text={text} locale={locale} theme={theme} onLocale={setLocale} onTheme={setTheme} />
      <main id="main-content" tabIndex={-1}>
        <p className="eyebrow">{text.eyebrow}</p><h1>{text.modules[page]}</h1>
        <section className="page-content" aria-labelledby="page-title">
          <span className="page-index" aria-hidden="true">{String(page + 1).padStart(2, '0')}</span>
          <h2 id="page-title">{text.title}</h2><p className="intro">{text.intro}</p>
          <div className="empty-state"><p>{text.pending[page]}</p><small>{text.scaffold}</small></div>
        </section>
        <RuntimeStatusPanel runtime={runtime} text={text} />
      </main>
    </div>
  </div>;
}
