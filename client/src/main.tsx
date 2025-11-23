import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import Pipeline from './base/Pipeline';
import { Router, useRoute } from './hooks/router';
import './index.css';
import Layout from './Layout';
import { activateLenis } from './utils/lenisInstance';

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
  '/': {
    title: 'Koii',
    element: (
      <Pipeline name="Landing" parentPath="/">
        <div className="w-full h-dvh bg-[white]/80 pt-12">
          <p>Hello there</p>
        </div>
      </Pipeline>
    ),
  },
  '/account': {
    title: 'Koii - Account',
    element: (
      <Pipeline name="Aaccount" parentPath="/account">
        <p className="pt-12">Welcome to account</p>
      </Pipeline>
    ),
  },
  '/apps': {
    title: 'Koii - Apps',
    element: (
      <Pipeline name="Apps" parentPath="/apps">
        <p className="pt-12">Welcome to apps</p>
      </Pipeline>
    ),
  },
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
