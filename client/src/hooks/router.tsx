import { hookstate, useHookstate } from 'hookstate';
import { useLayoutEffect, useRef, useState } from 'react';

const currentHrefGlobal = hookstate(window.location.href);

window.addEventListener('popstate', () => {
  currentHrefGlobal.set(window.location.href);
});

/// Notify route changes, with optionally wait a bit before fire.
export function useRoute(delay?: number) {
  const currentHref = useHookstate(currentHrefGlobal);
  const [route, setRoute] = useState(new URL(currentHrefGlobal.value));
  const timeoutHolder = useRef(setTimeout(() => {}, 0));

  useLayoutEffect(() => {
    if (delay) {
      clearTimeout(timeoutHolder.current);
      timeoutHolder.current = setTimeout(() => {
        setRoute(new URL(currentHref.value));
      }, delay);
    } else {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setRoute(new URL(currentHref.value));
    }

    return () => {
      clearTimeout(timeoutHolder.current);
    };
  }, [currentHref, delay]);

  return route;
}

export function navigate(to: string) {
  history.pushState(undefined, '', to);
  currentHrefGlobal.set(window.location.href);
}
