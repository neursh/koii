import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import Pipeline from './base/Pipeline';
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

const router = createBrowserRouter([
  {
    path: '/',
    element: <Layout />,
    children: [
      {
        index: true,
        element: (
          <Pipeline
            title="Koii"
            debugName="Landing"
            parentPath="/"
            key="anding"
          >
            <p>Hello</p>
          </Pipeline>
        ),
      },
      {
        path: '/account',
        element: (
          <Pipeline
            title="Koii - Account"
            debugName="Account"
            parentPath="/account"
            key="acc"
          >
            <p>Hello 2</p>
          </Pipeline>
        ),
      },
    ],
  },
]);

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <RouterProvider router={router} />
  </StrictMode>
);
