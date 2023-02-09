use anyhow;
use backoff::{future::retry, ExponentialBackoff};
use hello_world::greeter_client::GreeterClient;
use hello_world::{HelloReply, HelloRequest};
use tonic::transport::Channel;
use tonic::{Response, Status};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

pub struct RetryClient {
    client: GreeterClient<Channel>,
}

pub struct RetryConnection {
    channel: Channel,
}

impl RetryConnection {
    pub async fn new(s: &'static str) -> anyhow::Result<Self> {
        let channel = Channel::from_static(s).connect_lazy();
        Ok(Self { channel })
    }
    pub fn client(&self) -> RetryClient {
        let client: GreeterClient<Channel> = GreeterClient::new(self.channel.clone());
        RetryClient { client }
    }
    pub async fn say_hello_retry(&self) -> Result<Response<HelloReply>, Status> {
        retry(ExponentialBackoff::default(), || async {
            let mut retry_client = self.client();
            let request = tonic::Request::new(HelloRequest {
                name: "Tonic".into(),
            });
            Ok(retry_client.client.say_hello(request).await?)
        })
        .await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let retry_connection = RetryConnection::new("http://[::1]:50051").await?;
    let response = retry_connection.say_hello_retry().await?;
    println!("RESPONSE={:?}", response);

    Ok(())
}
