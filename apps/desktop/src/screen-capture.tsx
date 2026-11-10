import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';
import { errorMessage } from './platform/vocabulary';
import type { Locale } from './content';

const captureMessages = {
  zh: { label: '屏幕框选', image: '冻结屏幕', cancel: '取消截图', loading: '正在读取冻结屏幕…', empty: '请选择一个区域。', invalid: '截图请求无效。',
    help: '拖动框选；Esc 取消。键盘：方向键移动，空格定起点，Shift+方向键调终点，Enter 确认。' },
  en: { label: 'Screen selection', image: 'Frozen screen', cancel: 'Cancel capture', loading: 'Reading frozen screen…', empty: 'Select an area.', invalid: 'Invalid capture request.',
    help: 'Drag to select; Esc to cancel. Keyboard: arrows move, Space sets the start, Shift+arrows adjust the end, Enter confirms.' },
  fr: { label: 'Sélection de l’écran', image: 'Écran figé', cancel: 'Annuler la capture', loading: 'Lecture de l’écran figé…', empty: 'Sélectionnez une zone.', invalid: 'Demande de capture invalide.',
    help: 'Faites glisser pour sélectionner ; Échap pour annuler. Clavier : flèches pour déplacer, Espace pour fixer le début, Maj+flèches pour ajuster la fin, Entrée pour confirmer.' },
};

const screenSchema = z.object({ width: z.number().int().min(1).max(8192), height: z.number().int().min(1).max(8192),
  data_url: z.string().max(89_478_510).regex(/^data:image\/png;base64,[A-Za-z0-9+/]+={0,2}$/) }).strict();
interface Point { readonly x: number; readonly y: number }

function useScreen(request: number, text: typeof captureMessages.zh) {
  const [screen, setScreen] = useState<z.infer<typeof screenSchema>>(), [error, setError] = useState('');
  useEffect(() => {
    let active = true;
    if (!Number.isSafeInteger(request) || request < 1) { setError(text.invalid); return; }
    void invoke('ocr_screen', { request }).then((value) => {
      if (active) setScreen(screenSchema.parse(value));
    }).catch((cause: unknown) => { if (active) setError(errorMessage(cause)); });
    return () => { active = false; };
  }, [request, text]);
  function cancel() { void invoke('ocr_abort', { request }).catch((cause: unknown) => setError(errorMessage(cause))); }
  async function submit(start: Point | undefined, end: Point) {
    if (!screen || !start) return;
    if (start.x === end.x || start.y === end.y) { setError(text.empty); return; }
    const x = Math.floor(Math.min(start.x, end.x));
    const y = Math.floor(Math.min(start.y, end.y));
    const right = Math.min(screen.width, Math.ceil(Math.max(start.x, end.x)));
    const bottom = Math.min(screen.height, Math.ceil(Math.max(start.y, end.y)));
    const region = { x, y, width: right - x, height: bottom - y };
    try { await invoke('ocr_region', { request, region }); }
    catch (cause) { setError(errorMessage(cause)); }
  }
  return { screen, error, cancel, submit };
}

export function ScreenCapture({ request: identity, locale }: { readonly request: string; readonly locale: Locale }) {
  const text = captureMessages[locale], capture = useScreen(Number(identity), text), root = useRef<HTMLDivElement>(null);
  const [start, setStart] = useState<Point>(), [position, setCursor] = useState<Point>();
  const { width, height } = capture.screen ?? { width: 1, height: 1 };
  const cursor = position ?? { x: width / 2, y: height / 2 };
  useEffect(() => { root.current?.focus(); }, []);
  // Bitmap points preserve the chosen pixels when the viewport or DPI changes during a drag.
  function point(x: number, y: number) { return { x: Math.max(0, Math.min(width, x * width / window.innerWidth)), y: Math.max(0, Math.min(height, y * height / window.innerHeight)) }; }
  const rectangle = start ? { left: `${Math.min(start.x, cursor.x) * 100 / width}%`, top: `${Math.min(start.y, cursor.y) * 100 / height}%`,
    width: `${Math.abs(start.x - cursor.x) * 100 / width}%`, height: `${Math.abs(start.y - cursor.y) * 100 / height}%` } : undefined;
  return <div ref={root} className="screen-capture" role="application" aria-label={text.label} lang={locale} tabIndex={0}
    onPointerDown={(event) => {
      if (event.button !== 0 || !capture.screen) return;
      const value = point(event.clientX, event.clientY); setStart(value); setCursor(value); event.currentTarget.setPointerCapture(event.pointerId);
    }} onPointerMove={(event) => { if (event.buttons === 1) setCursor(point(event.clientX, event.clientY)); }}
    onPointerUp={(event) => { if (event.button === 0) void capture.submit(start, point(event.clientX, event.clientY)); }}
    onKeyDown={(event) => {
      if (event.key === 'Escape') { event.preventDefault(); capture.cancel(); return; }
      if (event.target !== event.currentTarget) return;
      if (event.key === ' ') { event.preventDefault(); setStart(cursor); return; }
      if (event.key === 'Enter') { event.preventDefault(); void capture.submit(start, cursor); return; }
      const step = event.shiftKey ? 20 : 1;
      const move = ({ ArrowLeft: [-step, 0], ArrowRight: [step, 0], ArrowUp: [0, -step], ArrowDown: [0, step] } as Record<string, number[]>)[event.key];
      if (move) { event.preventDefault(); setCursor(point(cursor.x * window.innerWidth / width + move[0]!, cursor.y * window.innerHeight / height + move[1]!)); }
    }}>
    {capture.screen && <img className="frozen-screen" src={capture.screen.data_url} alt={text.image} draggable={false} />}
    {rectangle ? <div className="capture-rectangle" style={rectangle} /> : <div className="capture-cursor" style={{ left: `${cursor.x * 100 / width}%`, top: `${cursor.y * 100 / height}%` }} />}
    <div className="capture-controls" onPointerDown={(event) => event.stopPropagation()} onPointerUp={(event) => event.stopPropagation()}>
      <p>{text.help}</p>
      <button onClick={capture.cancel}>{text.cancel}</button>
      {!capture.screen && !capture.error && <p role="status">{text.loading}</p>}
      {capture.error && <p role="alert">{capture.error}</p>}
    </div>
  </div>;
}
