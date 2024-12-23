use octopus::{Controller, GetMapping};

#[Controller("/api")]
trait ExampleController {
    #[GetMapping("/example")]
    fn example_return();
}

impl ExampleController for () {
    fn example_return() {
        println!("Hello from /api/example");
    }
}

fn main() {
    let router = setup_router();
    router.handle_request("/api/example");
    router.handle_request("/api/unknown");
}