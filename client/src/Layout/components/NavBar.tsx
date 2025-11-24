import { motion } from 'motion/react';
import Link from '../../schemas/Link';

export default function NavBar() {
  return (
    <motion.section className="fixed left-0 right-0 flex justify-between items-center m-3 p-2 pl-3 pr-4 text-[white]/90 z-100">
      <div className="flex gap-6 font-bold items-center p-2 pl-3 pr-4 rounded-2xl backdrop-blur-sm border border-[white]/40">
        <Link to="/">
          <h1 className="font-[Stardom] text-2xl">Koii</h1>
        </Link>
        <Link to="/apps">Apps</Link>
        <Link to="/community">Community projects</Link>
      </div>
      <div className="flex gap-4 font-bold items-center p-2 pl-4 pr-4 rounded-2xl backdrop-blur-sm border border-[white]/40">
        <a
          href="https://github.com/neursh/koii"
          target="_blank"
          rel="noopener noreferrer"
        >
          Github
        </a>
        <Link to="/account">Account</Link>
      </div>
    </motion.section>
  );
}
