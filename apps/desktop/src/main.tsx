import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { runtime } from './platform/runtime';
import { ScreenCapture } from './screen-capture';
import './styles.css';

const root = document.getElementById('root');
if (!root) throw new Error('Application root is missing');
const parameters = new URL(window.location.href).searchParams;
const capture = parameters.get('capture'), locale = parameters.get('locale');
createRoot(root).render(capture === null ? <StrictMode><App runtime={runtime} /></StrictMode> : <ScreenCapture request={capture} locale={locale === 'en' || locale === 'fr' ? locale : 'zh'} />);
