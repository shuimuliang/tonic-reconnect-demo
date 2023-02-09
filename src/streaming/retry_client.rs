pub mod pb {
    tonic::include_proto!("grpc.examples.echo");
}

use pb::{echo_client::EchoClient, EchoRequest, EchoResponse};

use anyhow;
use backoff::{future::retry, ExponentialBackoff};
use futures::stream::Stream;
use std::time::Duration;
use tokio_stream::StreamExt;
use tonic::{codec::Streaming, transport::Channel, Response};

pub struct RetryClient {
    client: EchoClient<Channel>,
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
        let client: EchoClient<Channel> = EchoClient::new(self.channel.clone());
        RetryClient { client }
    }
    pub async fn bidirectional_streaming_echo_throttle_retry(&self) -> anyhow::Result<()> {
        retry(ExponentialBackoff::default(), || async {
            println!("retrying");
            let mut retry_client = self.client();
            bidirectional_streaming_echo_throttle(&mut retry_client.client, Duration::from_secs(2))
                .await?;
            Ok(())
        })
        .await
    }
}

fn echo_requests_iter() -> impl Stream<Item = EchoRequest> {
    tokio_stream::iter(1..usize::MAX).map(|i| EchoRequest {
        message: format!("msg {:02}", i),
    })
}

async fn bidirectional_streaming_echo_throttle(
    client: &mut EchoClient<Channel>,
    dur: Duration,
) -> anyhow::Result<()> {
    let in_stream = echo_requests_iter().throttle(dur);

    // Result < tonic::Response < tonic::codec::Streaming < super::EchoResponse >>, tonic::Status >

    let response: Response<Streaming<EchoResponse>> =
        client.bidirectional_streaming_echo(in_stream).await?;
    // .unwrap();

    let mut resp_stream: Streaming<EchoResponse> = response.into_inner();

    while let Some(received) = resp_stream.next().await {
        match received {
            Err(e) => return Err(anyhow::anyhow!("Error: {:?}", e)),
            Ok(received) => println!("\treceived message: `{}`", received.message),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let retry_connection = RetryConnection::new("http://[::1]:50051").await?;

    println!("Streaming echo:");
    println!("\r\nBidirectional stream echo (kill client with CTLR+C):");
    retry_connection
        .bidirectional_streaming_echo_throttle_retry()
        .await?;

    Ok(())
}
