import { useHookstate, type State } from 'hookstate';
import { useCallback, useLayoutEffect, useRef, type ReactNode } from 'react';
import GlobalContext from '../context';
import { useLayoutEffectOnce } from '../hooks/effectOnce';
import { lenisInstance } from '../utils/lenisInstance';

export default function Pipeline(props: {
  children?: ReactNode;
  contentReady?: State<boolean>;
  title: string;
  debugName: string;
  parentPath: string;
  loadingPadding?: number;
  autoStartLenis?: boolean;
}) {
  return (
    <>
      <RouteHandler
        contentReady={props.contentReady}
        title={props.title}
        debugName={props.debugName}
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
  debugName: string;
  parentPath: string;
  loadingPadding?: number;
  autoStartLenis?: boolean;
}) {
  const landingTask = useRef(setInterval(() => {}));
  const pageRendered = useHookstate(GlobalContext.pageRendered);

  /**
   * A handler running in an interval, checking for errors, misconfigured context.
   * It will be disposed when the page quits.
   */
  const landingHandler = useCallback(() => {
    if (
      !pageRendered.value ||
      GlobalContext.currentParent.value !== props.parentPath
    ) {
      if (props.autoStartLenis === undefined || props.autoStartLenis) {
        lenisInstance.start();
      }

      pageRendered.set(true);
      GlobalContext.currentParent.set(props.parentPath);

      console.log(`[${props.debugName}] Renderer status: Mounted`);
    }
  }, [pageRendered, props.autoStartLenis, props.debugName, props.parentPath]);

  useLayoutEffect(() => {
    /**
     * Clear invalid task that were scheduled last effect.
     */
    clearInterval(landingTask.current);

    document.title = props.title;

    if (props.contentReady && !props.contentReady.value) {
      console.log(`[${props.debugName}] Renderer status: Loading scene`);
      return;
    }

    landingTask.current = setInterval(
      landingHandler,
      (props.loadingPadding ?? 0) * 1000 + 500
    );
  }, [
    landingHandler,
    props.contentReady,
    props.loadingPadding,
    props.debugName,
    props.title,
  ]);

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

    console.log(`[${props.debugName}] Renderer status: Unmounted`);
  });

  return null;
}
