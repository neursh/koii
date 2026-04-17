use std::time::{ Duration, SystemTime };

use axum::{ extract::{ Request, State }, middleware::Next, response::IntoResponse };

pub async fn padding(
    State(pad): State<Duration>,
    request: Request,
    next: Next
) -> impl IntoResponse {
    let start = SystemTime::now();
    let response = next.run(request).await;

    match start.elapsed() {
        Ok(finish) => {
            if finish < pad {
                tokio::time::sleep(pad - finish).await;
            } else {
                tracing::warn!(
                    "Time spent was too much for raising: {}ms > {}ms",
                    finish.as_millis(),
                    pad.as_millis()
                );
            }
        }
        Err(err) => {
            tracing::error!("System time error: {}", err);
        }
    }

    response
}
