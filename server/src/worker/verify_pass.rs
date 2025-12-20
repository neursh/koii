use std::thread;

use argon2::{ Argon2, PasswordHash, PasswordVerifier };
use tokio::sync::oneshot;

pub struct VerifyPassRequest {
    pub password: String,
    pub hash: String,
}

pub fn launch(
    rx: kanal::AsyncReceiver<(VerifyPassRequest, Option<oneshot::Sender<Option<bool>>>)>,
    threads: usize
) {
    for _ in 0..threads {
        let rx_branch = rx.clone().to_sync();
        thread::spawn(|| { worker(rx_branch) });
    }
}

fn worker(rx: kanal::Receiver<(VerifyPassRequest, Option<oneshot::Sender<Option<bool>>>)>) {
    let argon2 = Argon2::default();
    while let (request, Some(sender)) = rx.recv().unwrap() {
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
