import { navigate } from '../hooks/router';

export default function Layout() {
  return (
    <>
      <button onClick={() => navigate('/apps')}>yo</button>
      <button onClick={() => navigate('/apps?uf86f=8ygyguy')}>ya</button>
    </>
  );
}
