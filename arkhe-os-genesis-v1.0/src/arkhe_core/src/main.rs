use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Node {
    id: String,
    coherence: f64,
    satoshi: f64,
    handovers: u64,
}

struct AppState {
    node: Mutex<Node>,
}

#[derive(Deserialize)]
struct HandoverRequest {
    #[serde(alias = "nodeId")]
    to: String,
    #[serde(alias = "coherence")]
    payload: serde_json::Value,
}

#[derive(Serialize)]
struct HandoverResponse {
    success: bool,
    message: String,
    new_coherence: f64,
}

async fn status(state: web::Data<AppState>) -> impl Responder {
    let node = state.node.lock().unwrap();
    HttpResponse::Ok().json(&*node)
}

async fn handover(req: web::Json<HandoverRequest>, state: web::Data<AppState>) -> impl Responder {
    let mut node = state.node.lock().unwrap();
    node.handovers += 1;
    node.satoshi += 0.01; // pequeno ganho por handover
    node.coherence *= 0.999; // ligeira perda

    // Simula verificação ZK (Nó 22)
    let proof = format!("zk_{}_{}", node.id, node.handovers);
    log::info!("Handover para {} com prova {}", req.to, proof);

    HttpResponse::Ok().json(HandoverResponse {
        success: true,
        message: "Handover processado".to_string(),
        new_coherence: node.coherence,
    })
}

async fn anticipate() -> impl Responder {
    // Nó 23: retorna previsão de próxima coerência
    let future_coherence = 0.99; // exemplo
    HttpResponse::Ok().json(serde_json::json!({ "predicted_coherence": future_coherence }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let node_id = std::env::var("NODE_ID").unwrap_or_else(|_| Uuid::new_v4().to_string());
    let app_state = web::Data::new(AppState {
        node: Mutex::new(Node {
            id: node_id,
            coherence: 0.99,
            satoshi: 0.0,
            handovers: 0,
        }),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/status", web::get().to(status))
            .route("/handover", web::post().to(handover))
            .route("/anticipate", web::get().to(anticipate))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
