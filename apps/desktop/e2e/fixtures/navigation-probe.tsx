import { createRoot } from 'react-dom/client';
import { useLookupNavigation } from '../../src/conjugation-navigation';

export function navigationProbe() {
  let navigation: ReturnType<typeof useLookupNavigation>;
  const element = document.createElement('div');
  document.body.append(element);
  const root = createRoot(element);
  function Probe() {
    navigation = useLookupNavigation(action => action());
    return null;
  }
  const rendered = () => new Promise<void>(done => requestAnimationFrame(() => requestAnimationFrame(() => done())));
  root.render(<Probe />);
  return {
    ready: rendered(),
    state: () => navigation,
    act: async (action: (value: typeof navigation) => void) => { action(navigation); await rendered(); },
    close: () => { root.unmount(); element.remove(); },
  };
}
