import { hookstate, useHookstate } from 'hookstate';
import { useLayoutEffect, useRef, useState, type ReactNode } from 'react';
import { URLPattern } from 'urlpattern-polyfill/urlpattern';

export class Router {
  destinations: [URLPattern, ReactNode][];

  constructor(load: { [key: string]: ReactNode }) {
    this.destinations = Object.entries(load).map((destination) => [
      new URLPattern(destination[0], window.location.origin),
      destination[1],
    ]);
  }

  match(location: string) {
    const destination = this.destinations.find((destination) =>
      destination[0].test(location)
    );
    return destination ? destination[1] : undefined;
  }
}

const currentHrefGlobal = hookstate(window.location.href);

window.addEventListener('popstate', () => {
  currentHrefGlobal.set(window.location.href);
});

/// Notify route changes, with optionally wait a bit before fire.
export function useRoute(delay?: number) {
  const currentHref = useHookstate(currentHrefGlobal);
  const [route, setRoute] = useState(new URL(currentHrefGlobal.value));
  const timeoutHolder = useRef(setTimeout(() => {}));

  useLayoutEffect(() => {
    clearTimeout(timeoutHolder.current);
    timeoutHolder.current = setTimeout(() => {
      setRoute(new URL(currentHref.value));
    }, delay);

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
