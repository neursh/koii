use std::{ sync::Arc, thread };

use argon2::{ Argon2, password_hash::{ PasswordHasher, SaltString, rand_core::OsRng } };
use tokio::sync::{ Mutex, mpsc, oneshot };

pub fn launch(
  rx: mpsc::Receiver<(String, Option<oneshot::Sender<Option<String>>>)>,
  threads: usize
) {
  let rx_wraped = Arc::new(Mutex::new(rx));

  for _ in 0..threads {
    let rx_branch = rx_wraped.clone();
    thread::spawn(|| { worker(rx_branch) });
  }
}

fn worker(rx: Arc<Mutex<mpsc::Receiver<(String, Option<oneshot::Sender<Option<String>>>)>>>) {
  let argon2 = Argon2::default();
  loop {
    let retrieve = { rx.blocking_lock().blocking_recv().unwrap() };
    if let (password, Some(sender)) = retrieve {
      let salt = SaltString::generate(&mut OsRng);
      match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hashed) => {
          let _ = sender.send(Some(hashed.to_string()));
        }
        Err(_) => {
          let _ = sender.send(None);
        }
      }
    }
  }
}
