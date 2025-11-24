import DelayedOutlet from '../base/DelayedOutlet';
import NavBar from './components/NavBar';

export default function Layout() {
  return (
    <>
      <NavBar />
      <DelayedOutlet delay={500} />
    </>
  );
}
