import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { Router, useRoute } from './hooks/router.tsx';
import './index.css';
import Layout from './Layout/index.tsx';

const destinations = new Router({
  '/apps': <p>Welcome to apps</p>,
});

export function Container() {
  const route = useRoute(500);

  return (
    <main>
      <Layout />
      {destinations.match(route.href)}
    </main>
  );
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Container />
  </StrictMode>
);
