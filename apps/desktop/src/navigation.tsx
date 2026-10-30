import { BookOpen, BookMarked, Layers, Languages, Library, ListChecks, Settings, SpellCheck, PanelLeftClose, PanelLeftOpen } from 'lucide-react';
import type { Messages } from './content';

const icons = [BookOpen, SpellCheck, BookMarked, Layers, Library, Languages, ListChecks, Settings] as const;

interface NavigationProps {
  readonly text: Messages;
  readonly selected: number;
  readonly collapsed: boolean;
  readonly disabled?: boolean;
  readonly onSelect: (page: number) => void;
  readonly onToggle: () => void;
}

export function Navigation({ text, selected, collapsed, disabled, onSelect, onToggle }: NavigationProps) {
  const ToggleIcon = collapsed ? PanelLeftOpen : PanelLeftClose;
  const toggleLabel = collapsed ? text.expand : text.collapse;
  return <aside className="sidebar">
    <div className="brand"><span className="brand-mark" aria-hidden="true">L.</span>
      <div className="brand-copy"><strong>Lexique</strong><span>{text.subtitle}</span></div>
    </div>
    <nav aria-label={text.subtitle}>{text.modules.map((label, index) => {
      const Icon = icons[index];
      return <button key={label} type="button" title={label} aria-label={label} disabled={disabled}
        aria-current={selected === index ? 'page' : undefined} onClick={() => onSelect(index)}>
        {Icon && <Icon size={19} aria-hidden="true" />}<span className="nav-label">{label}</span>
      </button>;
    })}</nav>
    <button className="collapse" type="button" aria-label={toggleLabel} title={toggleLabel} onClick={onToggle}>
      <ToggleIcon size={18} aria-hidden="true" /><span className="nav-label">{toggleLabel}</span>
    </button>
  </aside>;
}
