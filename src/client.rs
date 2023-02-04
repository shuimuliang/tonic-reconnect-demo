use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let request1 = tonic::Request::new(HelloRequest {
        name: "Tonic1".into(),
    });
    let response1 = client.say_hello(request1).await?;
    println!("RESPONSE={:?}", response1);

    let request2 = tonic::Request::new(HelloRequest {
        name: "Tonic2".into(),
    });

    // let response2 = client.say_hello(request2).await?;
    let response2 = client.say_hello(request2).await;
    if response2.is_ok() {
        println!("RESPONSE={:?}", response2);
    } else {
        println!("Connect ERROR={:?}", response2.err());
    }

    Ok(())
}
