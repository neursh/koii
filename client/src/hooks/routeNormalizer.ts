import { useCallback, useLayoutEffect } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';

export function useRouteNormalizer(options: { autoReplace: boolean }) {
  const { pathname } = useLocation();
  const navigate = useNavigate();

  const normalizer = useCallback(
    (input: string) => input.replace(/\/+\//g, () => '/'),
    []
  );

  useLayoutEffect(() => {
    const result = normalizer(pathname);
    if (options.autoReplace && result !== pathname) {
      window.location.replace(result);
    }
  }, [options.autoReplace, pathname, navigate, normalizer]);

  return {
    pathname,
    normalizer,
  };
}
