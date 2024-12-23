use actix_web::{web, App, HttpResponse, HttpServer};
use octopus::{Controller, GetMapping};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
struct ApiResponse {
    message: String,
    path: String,
}

#[Controller("/api")]
trait ExampleController {
    #[GetMapping("/example")]
    fn example() -> HttpResponse;

    #[GetMapping]
    fn index() -> HttpResponse;
}

impl ExampleController for () {
    fn example() -> HttpResponse {
        let response = ApiResponse {
            message: "Hello from example".to_string(),
            path: "/api/example".to_string(),
        };
        HttpResponse::Ok().json(response)
    }

    fn index() -> HttpResponse {
        let response = ApiResponse {
            message: "Hello from index".to_string(),
            path: "/api".to_string(),
        };
        HttpResponse::Ok().json(response)
    }
}

async fn handle_request(req: actix_web::HttpRequest, router: web::Data<Arc<Router>>) -> HttpResponse {
    let path = req.path().to_string();
    match router.handle_request(&path) {
        Ok(_) => HttpResponse::Ok().json(ApiResponse {
            message: "Success".to_string(),
            path,
        }),
        Err(e) => HttpResponse::NotFound().json(ApiResponse {
            message: e,
            path,
        }),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let router = Arc::new(setup_router());
    println!("Available routes: {:?}", router.list_routes());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(router.clone()))
            .default_service(web::get().to(handle_request))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}