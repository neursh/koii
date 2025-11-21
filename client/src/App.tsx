import { Turnstile } from '@marsidev/react-turnstile';
import { useRef } from 'react';

export default function App() {
  const email = useRef<HTMLInputElement>(null);
  const password = useRef<HTMLInputElement>(null);
  return (
    <>
      <input ref={email} className="border" type="email" />
      <input ref={password} className="border" type="password" />
      <Turnstile
        siteKey="0x4AAAAAACB-ek7cQvTEJ5Ll"
        onSuccess={(token) =>
          fetch('https://auth.koii.space/user', {
            method: 'POST',
            credentials: 'include',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({
              email: email.current!.value,
              password: password.current!.value,
              clientstile: token,
            }),
          })
        }
      />
    </>
  );
}
