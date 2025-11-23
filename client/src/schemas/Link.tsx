import { motion } from 'motion/react';
import type { ReactNode } from 'react';
import { navigate } from '../hooks/router';

export default function Link(props: { href: string; children: ReactNode }) {
  return (
    <motion.a
      href={props.href}
      onClick={(event) => {
        event.preventDefault();
        navigate(props.href);
      }}
    >
      {props.children}
    </motion.a>
  );
}
