use std::thread;

use argon2::{ Argon2, PasswordHash, PasswordVerifier };
use tokio::sync::oneshot;

use crate::consts::{ ARGON2_MEMORY_COST, ARGON2_OUTPUT_LENGTH, ARGON2_PARALLELISM_COST };

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
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::ParamsBuilder
            ::new()
            .m_cost(ARGON2_MEMORY_COST)
            .p_cost(ARGON2_PARALLELISM_COST)
            .t_cost(ARGON2_MEMORY_COST)
            .output_len(ARGON2_OUTPUT_LENGTH)
            .build()
            .unwrap()
    );

    while let Ok((request, Some(sender))) = rx.recv() {
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
