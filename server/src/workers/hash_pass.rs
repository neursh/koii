use std::thread;

use argon2::{ Argon2, password_hash::{ PasswordHasher, SaltString, rand_core::OsRng } };
use tokio::sync::{ oneshot };

pub fn launch(
    rx: kanal::AsyncReceiver<(String, Option<oneshot::Sender<Option<String>>>)>,
    threads: usize
) {
    for _ in 0..threads {
        let rx_branch = rx.clone().to_sync();
        thread::spawn(|| { worker(rx_branch) });
    }
}

fn worker(rx: kanal::Receiver<(String, Option<oneshot::Sender<Option<String>>>)>) {
    let argon2 = Argon2::default();
    while let Ok((password, Some(sender))) = rx.recv() {
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
