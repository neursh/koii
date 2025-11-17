use std::{ collections::HashMap, thread, time::Duration };
use resend_rs::{ Resend, types::{ CreateEmailBaseOptions, EmailTemplate } };
use tokio::sync::{ mpsc, oneshot };

pub struct VerifyEmailRequest {
    pub email: String,
    pub verify_code: String,
}

// The oneshot param is required by design for each services, but we don't use it.
pub fn launch(
    rx: mpsc::Receiver<(VerifyEmailRequest, Option<oneshot::Sender<Option<()>>>)>,
    threads: usize
) {
    if threads > 1 {
        eprintln!("But sire, there can only be 1 email worker, I can't do this.");
    }

    thread::spawn(|| { worker(rx) });
}

fn worker(mut rx: mpsc::Receiver<(VerifyEmailRequest, Option<oneshot::Sender<Option<()>>>)>) {
    // DO NOT MOVE THIS UP TO THE LAUNCHER FUNCTION.
    // Resend uses `reqwest` under the hood.
    // And if it's defined as blocking, it can't be initialized inside of tokio context.
    let resend_token = std::env
        ::var("RESEND_TOKEN")
        .expect("RESEND_TOKEN must be set in .env file");
    let resend = Resend::new(&resend_token);

    loop {
        let waiting = rx.len();
        let mut requests = Vec::new();
        if waiting > 1 {
            rx.blocking_recv_many(&mut requests, std::cmp::min(waiting, 100));

            let batch: Vec<CreateEmailBaseOptions> = requests
                .iter()
                .map(|request| { create_verify_base(&request.0) })
                .collect();

            let _ = resend.batch.send(batch);
            continue;
        }

        if waiting == 1 {
            let request = rx.blocking_recv().unwrap();
            let _ = resend.emails.send(create_verify_base(&request.0));
        }

        thread::sleep(Duration::from_millis(1200));
    }
}

fn create_verify_base(request: &VerifyEmailRequest) -> CreateEmailBaseOptions {
    let mut variables = HashMap::new();
    variables.insert(
        "VERIFY_LINK".to_string(),
        serde_json::Value::String(format!("https://koii.space/verify?code={}", request.verify_code))
    );

    CreateEmailBaseOptions::new(
        "Koii Auth <auth@koii.space>",
        [&request.email],
        "Koii email verification"
    ).with_template(EmailTemplate::new("koii-verify").with_variables(variables))
}
