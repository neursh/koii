use std::{ collections::HashMap, thread, time::Duration };
use resend_rs::{ Resend, types::{ CreateEmailBaseOptions, EmailTemplate } };
use tokio::sync::oneshot;

pub struct VerifyEmailRequest {
    pub email: String,
    pub verify_code: String,
}

// The oneshot param is required by design for each services, but we don't use it.
pub fn launch(
    rx: kanal::AsyncReceiver<(VerifyEmailRequest, Option<oneshot::Sender<Option<()>>>)>,
    threads: usize
) {
    if threads > 1 {
        eprintln!("But sire, there can only be 1 email worker, I can't do this.");
    }

    let rx = rx.to_sync();
    thread::spawn(|| { worker(rx) });
}

fn worker(rx: kanal::Receiver<(VerifyEmailRequest, Option<oneshot::Sender<Option<()>>>)>) {
    // DO NOT MOVE THIS UP TO THE LAUNCHER FUNCTION.
    // Resend uses `reqwest` under the hood.
    // And if it's defined as blocking, it can't be initialized inside of tokio context.
    let resend_token = std::env
        ::var("RESEND_TOKEN")
        .expect("RESEND_TOKEN must be set in .env file");
    let resend = Resend::new(&resend_token);

    let mut requests = Vec::with_capacity(100);

    loop {
        let waiting = rx.len();
        if waiting > 1 {
            rx.drain_into(&mut requests).unwrap();

            let batch: Vec<CreateEmailBaseOptions> = requests
                .iter()
                .map(|request| { create_verify_base(&request.0) })
                .collect();

            let _ = resend.batch.send(batch);
            continue;
        }

        if waiting == 1 {
            let request = rx.recv().unwrap();
            let _ = resend.emails.send(create_verify_base(&request.0));
        }

        thread::sleep(Duration::from_millis(5000));
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
