use std::{ sync::Arc, thread };

use argon2::{ Argon2, PasswordHash, PasswordVerifier };
use tokio::sync::{ Mutex, mpsc, oneshot };

pub struct VerifyPassRequest {
  pub password: String,
  pub hash: String,
}

pub fn launch(
  rx: mpsc::Receiver<(VerifyPassRequest, Option<oneshot::Sender<Option<bool>>>)>,
  threads: usize
) {
  let rx_wraped = Arc::new(Mutex::new(rx));

  for _ in 0..threads {
    let rx_branch = rx_wraped.clone();
    thread::spawn(|| { worker(rx_branch) });
  }
}

fn worker(
  rx: Arc<Mutex<mpsc::Receiver<(VerifyPassRequest, Option<oneshot::Sender<Option<bool>>>)>>>
) {
  let argon2 = Argon2::default();
  loop {
    let retrieve = { rx.blocking_lock().blocking_recv().unwrap() };
    if let (request, Some(sender)) = retrieve {
      match PasswordHash::new(&request.hash) {
        Ok(hash) => {
          let _ = sender.send(
            Some(argon2.verify_password(request.password.as_bytes(), &hash).is_ok())
          );
        }
        Err(_) => {
          let _ = sender.send(None);
        }
      }
    }
  }
}
