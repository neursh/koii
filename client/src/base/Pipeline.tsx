import { useHookstate, type State } from 'hookstate';
import { useCallback, useLayoutEffect, useRef, type ReactNode } from 'react';
import GlobalContext from '../context';
import { useLayoutEffectOnce } from '../hooks/effectOnce';
import { lenisInstance } from '../utils/lenisInstance';

export default function Pipeline(props: {
  children?: ReactNode;
  contentReady?: State<boolean>;
  title: string;
  name: string;
  parentPath: string;
  loadingPadding?: number;
  autoStartLenis?: boolean;
}) {
  return (
    <>
      <RouteHandler
        contentReady={props.contentReady}
        title={props.title}
        name={props.name}
        parentPath={props.parentPath}
        loadingPadding={props.loadingPadding}
        autoStartLenis={props.autoStartLenis}
      />
      {props.children}
    </>
  );
}

function RouteHandler(props: {
  contentReady?: State<boolean>;
  title: string;
  name: string;
  parentPath: string;
  loadingPadding?: number;
  autoStartLenis?: boolean;
}) {
  const landingTask = useRef(setInterval(() => {}));
  const pageRendered = useHookstate(GlobalContext.pageRendered);
  const currentParent = useHookstate(GlobalContext.currentParent);

  /**
   * A handler running in an interval, checking for errors, misconfigured context.
   * It will be disposed when the page quits.
   */
  const landingHandler = useCallback(() => {
    if (!pageRendered.value) {
      if (props.autoStartLenis === undefined || props.autoStartLenis) {
        lenisInstance.start();
      }

      document.title = props.title;
      pageRendered.set(true);

      currentParent.set(props.parentPath);

      console.log(`[${props.name}] Renderer status: Mounted`);
    }
  }, [
    currentParent,
    pageRendered,
    props.autoStartLenis,
    props.name,
    props.parentPath,
    props.title,
  ]);

  useLayoutEffect(() => {
    /**
     * Clear invalid task that were scheduled last effect.
     */
    clearInterval(landingTask.current);

    if (props.contentReady && !props.contentReady.value) {
      console.log(`[${props.name}] Renderer status: Loading scene`);
      return;
    }

    landingTask.current = setInterval(
      landingHandler,
      (props.loadingPadding ?? 0) * 1000 + 500
    );
  }, [landingHandler, props.contentReady, props.loadingPadding, props.name]);

  useLayoutEffectOnce(() => () => {
    /**
     * Clear invalid task that were scheduled from the instance.
     */
    clearInterval(landingTask.current);

    /**
     * When unmounted, stop lenis to pass control to another Pipline instance.
     *
     * The unmount process only happens when the `<LoadingFallback />` kicks in,
     * so any visual glitches can happen in here, we can now set the scroll position to 0.
     */
    lenisInstance.stop();
    lenisInstance.scrollTo(0, { immediate: true, force: true });

    console.log(`[${props.name}] Renderer status: Unmounted`);
  });

  return null;
}
