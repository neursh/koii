import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { Router, useRoute } from './hooks/router.tsx';
import './index.css';
import Layout from './Layout/index.tsx';
import { activateLenis } from './utils/lenisInstance.ts';

/**
 * Avoid the save scroll progress feature to break the flow of the sites.
 */
window.scrollTo({ top: 0 });

/**
 * Activate the normal Lenis implementation to control the underlying instance
 * for ease of access.
 */
activateLenis();

const destinations = new Router({
  '/': <div className="w-full h-dvh bg-[white]/80"></div>,
  '/apps': <p>Welcome to apps</p>,
});

export function Container() {
  const route = useRoute(500);

  return (
    <main>
      <Layout>{destinations.match(route.href)}</Layout>
    </main>
  );
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Container />
  </StrictMode>
);
