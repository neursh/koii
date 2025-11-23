import { useEffect, useLayoutEffect } from 'react';

export function useEffectOnce(callback: React.EffectCallback) {
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(callback, []);
}

export function useLayoutEffectOnce(callback: React.EffectCallback) {
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useLayoutEffect(callback, []);
}
