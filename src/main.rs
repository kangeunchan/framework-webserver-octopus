use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

#[derive(Debug)]
struct ApiResponse {
    message: String,
    path: String,
}

impl ApiResponse {
    fn to_json(&self) -> String {
        format!(
            "{{\"message\":\"{}\",\"path\":\"{}\"}}",
            self.message, self.path
        )
    }
}

fn handle_client(mut stream: TcpStream, router: Arc<Router>) {
    let mut buffer = [0; 1024];
    if let Ok(bytes_read) = stream.read(&mut buffer) {
        if bytes_read == 0 {
            return;
        }

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let response = process_request(&request, &router);
        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Failed to send response: {}", e);
        }
    }
}

fn process_request(request: &str, router: &Router) -> String {
    let request_line = request.lines().next().unwrap_or("");
    let path = request_line.split_whitespace().nth(1).unwrap_or("");

    match router.handle_request(path) {
        Ok(response) => format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            response.to_json().len(),
            response.to_json()
        ),
        Err(message) => {
            let error_response = ApiResponse {
                message,
                path: path.to_string(),
            };
            format!(
                "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                error_response.to_json().len(),
                error_response.to_json()
            )
        }
    }
}

struct Router {
    routes: Vec<(String, Box<dyn Fn() -> ApiResponse + Send + Sync>)>,
}

impl Router {
    fn new() -> Self {
        Router { routes: Vec::new() }
    }

    fn add_route<F>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> ApiResponse + Send + Sync + 'static,
    {
        self.routes.push((path.to_string(), Box::new(handler)));
    }

    fn handle_request(&self, path: &str) -> Result<ApiResponse, String> {
        for (route, handler) in &self.routes {
            if route == path {
                return Ok(handler());
            }
        }
        Err("Route not found".to_string())
    }

    fn list_routes(&self) -> Vec<String> {
        self.routes.iter().map(|(route, _)| route.clone()).collect()
    }
}

fn setup_router() -> Router {
    let mut router = Router::new();

    router.add_route("/api/example", || ApiResponse {
        message: "Hello from example".to_string(),
        path: "/api/example".to_string(),
    });

    router.add_route("/api", || ApiResponse {
        message: "Hello from index".to_string(),
        path: "/api".to_string(),
    });

    router
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let router = Arc::new(setup_router());

    println!("Available routes: {:?}", router.list_routes());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let router = Arc::clone(&router);
                std::thread::spawn(move || {
                    handle_client(stream, router);
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}
