use actix_web::{post, web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

// ── Memos webhook payload ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct MemosWebhook {
    #[serde(rename = "activityType")]
    #[allow(dead_code)]
    activity_type: String,
    #[serde(rename = "creatorId")]
    #[allow(dead_code)]
    creator_id: i64,
    #[serde(rename = "createdTs")]
    #[allow(dead_code)]
    created_ts: i64,
    memo: Memo,
}

#[derive(Debug, Deserialize)]
struct Memo {
    #[allow(dead_code)]
    id: String,
    content: String,
    #[allow(dead_code)]
    visibility: String,
    #[allow(dead_code)]
    #[serde(rename = "createdTs")]
    created_ts: i64,
    #[allow(dead_code)]
    #[serde(rename = "updatedTs")]
    updated_ts: i64,
    #[allow(dead_code)]
    #[serde(rename = "rowStatus")]
    row_status: String,
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
    body: web::Json<MemosWebhook>,
    config: web::Data<Config>,
) -> HttpResponse {
    let payload = ProcessingPayload {
        source: "memos".into(),
        content: body.memo.content.clone(),
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