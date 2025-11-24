import { type ReactNode, useLayoutEffect, useRef, useState } from 'react';
import { useOutlet } from 'react-router-dom';
import GlobalContext from '../context';
import { useRouteNormalizer } from '../hooks/routeNormalizer';

export default function DelayedOutlet(props: { delay: number }) {
  // Middleware to detect and replace invalid url.
  useRouteNormalizer({ autoReplace: true });

  const renderPageTask = useRef(setTimeout(() => {}));
  const routerOutlet = useOutlet();

  const [renderer, setRenderer] = useState<ReactNode>(null);

  // Handle and update new parent page.
  useLayoutEffect(() => {
    /**
     * `pageRendered` should be outside of `renderPageTask`,
     * we alrady have `Pipeline` to correct errors.
     */
    GlobalContext.pageRendered.set(false);

    renderPageTask.current = setTimeout(() => {
      setRenderer(routerOutlet!);
    }, props.delay);

    return () => {
      /**
       * Clear invalid task that were scheduled last effect.
       */
      clearInterval(renderPageTask.current);
    };
  }, [props.delay, routerOutlet]);

  return renderer;
}
