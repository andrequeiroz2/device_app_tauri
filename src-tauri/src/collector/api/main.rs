use actix_web::{web, App, HttpServer};
use sqlx::Pool;
use sqlx::Sqlite;
use tracing::{info, error, warn};
use crate::collector::api::routes;
use std::net::TcpListener;

/// Finds an available port starting from the preferred port
fn find_available_port(preferred: u16) -> Option<u16> {
    for offset in 0..=10 {
        let port = preferred + offset;
        let addr = format!("127.0.0.1:{}", port);
        
        // Try to bind to check if port is available
        if TcpListener::bind(&addr).is_ok() {
            return Some(port);
        }
    }
    None
}

/// Starts the HTTP API server in background
/// Tries to bind to 127.0.0.1:8080, falls back to 8081, 8082, etc. if port is in use
/// Port can be configured via COLLECTOR_API_PORT environment variable
pub fn start_api_server_background(pool: Pool<Sqlite>) {
    // Get preferred port from environment or use default
    let preferred_port = std::env::var("COLLECTOR_API_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8080);
    
    // Find available port
    let port = match find_available_port(preferred_port) {
        Some(p) => {
            if p != preferred_port {
                warn!(
                    preferred_port = preferred_port,
                    actual_port = p,
                    "Preferred port was in use, using alternative port"
                );
            }
            p
        }
        None => {
            error!(
                preferred_port = preferred_port,
                "No available port found in range {}-{}",
                preferred_port,
                preferred_port + 10
            );
            return;
        }
    };
    
    let bind_addr = format!("127.0.0.1:{}", port);

    info!(
        "Collector HTTP API starting at http://127.0.0.1:{}",
        port
    );

    // Run Actix in a dedicated thread with its own runtime to avoid
    // spawn_local conflict with Tauri's Tokio runtime
    std::thread::spawn(move || {
        actix_web::rt::System::new().block_on(async move {
            let server = HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(pool.clone()))
                    .service(routes::configure_routes())
            })
            .bind(&bind_addr);

            match server {
                Ok(server) => {
                    info!(
                        "Collector HTTP API running at http://127.0.0.1:{}",
                        port
                    );
                    if let Err(e) = server.run().await {
                        error!(error = %e, bind_addr = %bind_addr, "HTTP API server error");
                    }
                }
                Err(e) => {
                    error!(error = %e, bind_addr = %bind_addr, "Failed to bind HTTP API server");
                }
            }
        });
    });
}
