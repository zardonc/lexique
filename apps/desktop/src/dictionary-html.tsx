import { useMemo } from 'react';

const tags = new Set('div span p b strong i em u s sup sub small h1 h2 h3 h4 h5 h6 br hr ul ol li dl dt dd table thead tbody tfoot tr td th blockquote pre code'.split(' '));
const discarded = new Set('script style link meta base iframe object embed img video audio source template form input button select textarea'.split(' '));
const properties = /^(font(-family|-size|-style|-weight)?|color|background-color|text-(align|decoration|indent|transform)|vertical-align|line-height|white-space|display|border(-[a-z-]+)?|margin(-[a-z]+)?|padding(-[a-z]+)?|width|max-width)$/;

function inlineStyle(element: HTMLElement) {
  return Array.from(element.style).filter((property) => properties.test(property))
    .map((property) => [property, element.style.getPropertyValue(property)])
    .filter(([, value]) => !/url\s*\(|@import|expression\s*\(/i.test(value))
    .map(([property, value]) => `${property}:${value}`).join(';');
}

function htmlDocument(html: string) {
  const template = document.createElement('template');
  template.innerHTML = html;
  for (const element of template.content.querySelectorAll('*')) {
    if (!(element instanceof HTMLElement) || discarded.has(element.localName)) { element.remove(); continue; }
    if (!tags.has(element.localName)) { element.replaceWith(...element.childNodes); continue; }
    const style = inlineStyle(element);
    for (const attribute of Array.from(element.attributes)) {
      if (!['title', 'lang', 'colspan', 'rowspan'].includes(attribute.name)) element.removeAttribute(attribute.name);
    }
    if (style) element.setAttribute('style', style);
  }
  return `<!doctype html><html lang="fr"><head><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'"><style>body{font:16px/1.6 system-ui,sans-serif;color:#222;background:#fff;margin:12px;overflow-wrap:anywhere}table{max-width:100%}pre{white-space:pre-wrap}</style></head><body>${template.innerHTML}</body></html>`;
}

export function DictionaryHtml({ html, title }: { readonly html: string; readonly title: string }) {
  const content = useMemo(() => htmlDocument(html), [html]);
  return <iframe className="dictionary-html" sandbox="" title={title} srcDoc={content} />;
}
