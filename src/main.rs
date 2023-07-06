use tonic::{transport::Server, Request, Response, Status};

use hello_world::{HelloResponse, HelloRequest, hello_world_server::{HelloWorld, HelloWorldServer}};

pub mod hello_world {
    tonic::include_proto!("pool"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
struct MyHelloWorld {}

#[tonic::async_trait]
impl HelloWorld for MyHelloWorld {
    async fn hello(
        &self,
        request: Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<HelloResponse>, Status> { // Return an instance of type HelloReply
        println!("Got a request:");
        let inner_request = request.into_inner();
        println!("{} from {}", inner_request.message, inner_request.name);

        let reply = hello_world::HelloResponse {
            message: format!("I am great, thanks for asking, {}!", inner_request.name).into(), // We must use .into_inner() as the fields of gRPC requests and responses are private
            name: format!("Server"),
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:8080".parse()?;
    let greeter = MyHelloWorld::default();

    Server::builder()
        .add_service(HelloWorldServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}