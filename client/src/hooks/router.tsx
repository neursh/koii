import { hookstate, useHookstate } from 'hookstate';
import {
  Fragment,
  useLayoutEffect,
  useRef,
  useState,
  type ReactNode,
} from 'react';
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

    if (!destination) return undefined;

    return <Fragment key={`router-${location}`}>{destination[1]}</Fragment>;
  }
}

const currentHrefGlobal = hookstate(window.location.href);

window.addEventListener('popstate', () => {
  currentHrefGlobal.set(window.location.href);
});

/// Notify route changes, with optionally wait a bit before fire.
export function useRoute(delay?: number): URL {
  const currentHref = useHookstate(currentHrefGlobal);
  const [route, setRoute] = useState(new URL(currentHrefGlobal.value));
  const timeoutHolder = useRef(setTimeout(() => {}));

  useLayoutEffect(() => {
    clearTimeout(timeoutHolder.current);
    timeoutHolder.current = setTimeout(() => {
      if (route.href !== currentHref.value) {
        setRoute(new URL(currentHref.value));
      }
    }, delay);

    return () => {
      clearTimeout(timeoutHolder.current);
    };
  }, [currentHref, delay, route.href]);

  return route;
}

export function navigate(to: string) {
  if (to === window.location.href.substring(window.location.origin.length)) {
    return;
  }

  history.pushState(undefined, '', to);
  currentHrefGlobal.set(window.location.href);
}
