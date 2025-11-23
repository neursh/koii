import { useHookstate } from 'hookstate';
import { motion } from 'motion/react';
import { useLayoutEffect, type ReactNode } from 'react';
import GlobalContext from '../context';
import { useRoute } from '../hooks/router';
import Link from '../schemas/Link';

export default function Layout(props: { children?: ReactNode }) {
  return (
    <section className="fixed w-full h-dvh bg-[#F1E3D3]">
      <div className="absolute w-full flex justify-between items-center p-2 pl-3 pr-4 text-[black]/65 z-100">
        <Link href="/">
          <h1 className="font-[Stardom] text-2xl">Koii</h1>
        </Link>
        <NavigateManager />
        <div className="flex gap-3 font-bold">
          <a
            href="https://github.com/neursh/koii"
            target="_blank"
            rel="noopener noreferrer"
          >
            G
          </a>
          <Link href="/account">A</Link>
        </div>
      </div>
      <svg className="absolute w-full h-dvh">
        <defs>
          <mask id="content">
            <rect width="100%" height="100%" fill="black" />
            <motion.rect
              style={{
                rx: '12px',
                ry: '12px',
                width: 'calc(100dvw - 24px)',
                height: 'calc(100dvh - 60px)',
                translateX: '12px',
                translateY: '48px',
              }}
              fill="white"
            />
          </mask>
        </defs>
      </svg>
      <section className="aboslute" style={{ mask: `url(#content)` }}>
        {props.children}
      </section>
    </section>
  );
}

export function NavigateManager() {
  const route = useRoute();
  const pageRendered = useHookstate(GlobalContext.pageRendered);

  useLayoutEffect(() => {
    pageRendered.set(false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [route]);

  return (
    <section className="">{pageRendered.value ? 'done' : 'loading'}</section>
  );
}
