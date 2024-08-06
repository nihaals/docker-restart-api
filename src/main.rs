use std::env;
use std::net::SocketAddr;

use actix_web::{post, web, App, HttpResponse, HttpServer};
use serde::Deserialize;

struct State {
    client: bollard::Docker,
}

#[derive(Deserialize)]
struct SearchBody {
    query: String,
}

#[post("/restart/search")]
async fn search(state: web::Data<State>, body: web::Json<SearchBody>) -> HttpResponse {
    let containers = state.client.list_containers::<&str>(None).await.unwrap();
    let mut found_container: Option<String> = None;
    for container in containers {
        if let Some(names) = container.names {
            for name in names {
                if name.contains(&body.query) {
                    if found_container.is_some() {
                        return HttpResponse::BadRequest()
                            .content_type("text/plain")
                            .body("Multiple containers found");
                    }
                    found_container = Some(container.id.clone().unwrap());
                }
            }
        }
    }
    if let Some(container_id) = found_container {
        state
            .client
            .restart_container(&container_id, None)
            .await
            .unwrap();
        HttpResponse::Ok()
            .content_type("text/plain")
            .body("Container restarted")
    } else {
        HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Container not found")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    let state = web::Data::new(State {
        client: bollard::Docker::connect_with_local_defaults().unwrap(),
    });

    println!("Listening on http://{}...", addr);
    HttpServer::new(move || App::new().app_data(state.clone()).service(search))
        .bind(addr)?
        .run()
        .await
}
