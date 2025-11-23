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
  '/': (
    <Pipeline title="Koii" name="Landing" parentPath="/">
      <div className="w-full h-dvh bg-[white]/80"></div>
    </Pipeline>
  ),
  '/account': (
    <Pipeline title="Koii - Account" name="Apps" parentPath="/">
      <p>Welcome to account</p>
    </Pipeline>
  ),
  '/apps': (
    <Pipeline title="Koii - Apps" name="Apps" parentPath="/">
      <p>Welcome to apps</p>
    </Pipeline>
  ),
});

export function RouteProc() {
  const route = useRoute(500);
  return destinations.match(route.href);
}

export function Container() {
  return (
    <main>
      <Layout>
        <RouteProc />
      </Layout>
    </main>
  );
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Container />
  </StrictMode>
);
