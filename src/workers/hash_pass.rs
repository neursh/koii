use std::thread;

use argon2::{
    Algorithm,
    Argon2,
    Params,
    Version,
    password_hash::{ PasswordHasher, SaltString, rand_core::OsRng },
};
use tokio::sync::{ oneshot };

use crate::consts::{
    ARGON2_MEMORY_COST,
    ARGON2_OUTPUT_LENGTH,
    ARGON2_PARALLELISM_COST,
    ARGON2_TIME_COST,
};

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
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(
            ARGON2_MEMORY_COST,
            ARGON2_TIME_COST,
            ARGON2_PARALLELISM_COST,
            Some(ARGON2_OUTPUT_LENGTH)
        ).unwrap()
    );
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
