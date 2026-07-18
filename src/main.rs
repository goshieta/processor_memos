use actix_web::{post, web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

// ── Memos webhook payload ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct MemosWebhook {
    #[allow(dead_code)]
    url: String,
    #[serde(rename = "activityType")]
    #[allow(dead_code)]
    activity_type: String,
    #[allow(dead_code)]
    creator: String,
    memo: Memo,
}

#[derive(Debug, Deserialize)]
struct Memo {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    state: i32,
    #[allow(dead_code)]
    creator: String,
    #[serde(rename = "create_time")]
    #[allow(dead_code)]
    create_time: Option<Timestamp>,
    #[serde(rename = "update_time")]
    #[allow(dead_code)]
    update_time: Option<Timestamp>,
    content: String,
    #[allow(dead_code)]
    visibility: i32,
    #[allow(dead_code)]
    property: serde_json::Value,
    #[allow(dead_code)]
    snippet: String,
}

#[derive(Debug, Deserialize)]
struct Timestamp {
    #[allow(dead_code)]
    seconds: i64,
}

// ── Processing server payload ──────────────────────────────────────────

#[derive(Debug, Serialize)]
struct ProcessingPayload {
    source: String,
    content: String,
}

// ── Configuration ──────────────────────────────────────────────────────

struct Config {
    listen_addr: String,
    processing_server_url: String,
}

impl Config {
    fn from_env() -> Self {
        Self {
            listen_addr: std::env::var("LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".into()),
            processing_server_url: std::env::var("PROCESSING_SERVER_URL")
                .expect("PROCESSING_SERVER_URL must be set"),
        }
    }
}

// ── Handler ────────────────────────────────────────────────────────────

#[post("/webhook")]
async fn webhook_handler(
    body: String,
    config: web::Data<Config>,
) -> HttpResponse {
    // デバッグ用: 受信したリクエストボディをそのまま出力
    eprintln!("=== Received webhook body ===");
    eprintln!("{body}");
    eprintln!("=== End of webhook body ===");

    let webhook: MemosWebhook = match serde_json::from_str(&body) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Failed to deserialize webhook body: {e}");
            return HttpResponse::BadRequest()
                .body(format!("Invalid webhook payload: {e}"));
        }
    };

    let payload = ProcessingPayload {
        source: "memos".into(),
        content: webhook.memo.content.clone(),
    };

    let client = match reqwest::Client::builder().build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to build HTTP client: {e}");
            return HttpResponse::InternalServerError()
                .body("Failed to build HTTP client");
        }
    };

    match client
        .post(&config.processing_server_url)
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                HttpResponse::Ok().body("OK")
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                eprintln!(
                    "Processing server returned {status}: {body}"
                );
                HttpResponse::BadGateway().body("Processing server error")
            }
        }
        Err(e) => {
            eprintln!("Failed to forward to processing server: {e}");
            HttpResponse::BadGateway()
                .body("Failed to forward to processing server")
        }
    }
}

// ── Main ───────────────────────────────────────────────────────────────

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env();
    println!(
        "Starting processor_memos on {}, forwarding to {}",
        config.listen_addr, config.processing_server_url
    );

    let listen_addr = config.listen_addr.clone();
    let config = web::Data::new(config);

    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .service(webhook_handler)
    })
    .bind(&listen_addr)?
    .run()
    .await
}