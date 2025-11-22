import { StrictMode, useMemo, type ReactElement } from 'react';
import { createRoot } from 'react-dom/client';
import { URLPattern } from 'urlpattern-polyfill/urlpattern';
import { useRoute } from './hooks/router.tsx';
import './index.css';
import Layout from './Layout/index.tsx';

const destinations: { [key: string]: ReactElement } = {
  '/': <></>,
  '/apps': <p>Welcome to apps</p>,
};

export function Container() {
  const route = useRoute(500);
  const destinationsParse: [URLPattern, ReactElement][] = useMemo(
    () =>
      Object.entries(destinations).map((destination) => [
        new URLPattern(destination[0], window.location.origin),
        destination[1],
      ]),
    []
  );

  const outlet = useMemo(() => {
    const destination = destinationsParse.find((destination) =>
      destination[0].test(route.href)
    );
    return destination ? destination[1] : null;
  }, [destinationsParse, route.href]);

  return (
    <main>
      <Layout />
      {outlet}
    </main>
  );
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Container />
  </StrictMode>
);
