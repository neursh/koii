import { motion } from 'motion/react';
import type { ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';

export default function Link(props: { to: string; children: ReactNode }) {
  const navigator = useNavigate();

  return (
    <motion.a
      href={props.to}
      onClick={(event) => {
        event.preventDefault();
        navigator(props.to);
      }}
    >
      {props.children}
    </motion.a>
  );
}
