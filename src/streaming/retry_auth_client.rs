pub mod pb {
    tonic::include_proto!("grpc.examples.echo");
}

use pb::{echo_client::EchoClient, EchoRequest, EchoResponse};

use anyhow;
use backoff::{future::retry, ExponentialBackoff};
use futures::stream::Stream;
use std::time::Duration;
use tokio_stream::StreamExt;
use tonic::{
    codec::Streaming,
    transport::Channel,
    Request,
    Response,
    Status,
    metadata::MetadataValue,
    metadata::Ascii,
    service::interceptor::InterceptedService,

};
// use thiserror::Error;
//
// #[derive(Debug, Error)]
// pub enum Error {
//     #[error("AccessToken: {0}")]
//     AccessToken(String),
//
//     #[error("Certificate: {0}")]
//     Certificate(String),
//
//     #[error("I/O: {0}")]
//     Io(std::io::Error),
//
//     #[error("Transport: {0}")]
//     Transport(tonic::transport::Error),
//
//     #[error("Invalid URI {0}: {1}")]
//     InvalidUri(String, String),
//
//     #[error("Row not found")]
//     RowNotFound,
//
//     #[error("Row write failed")]
//     RowWriteFailed,
//
//     #[error("Row delete failed")]
//     RowDeleteFailed,
//
//     #[error("Object not found: {0}")]
//     ObjectNotFound(String),
//
//     #[error("Object is corrupt: {0}")]
//     ObjectCorrupt(String),
//
//     #[error("RPC: {0}")]
//     Rpc(tonic::Status),
//
//     #[error("Timeout")]
//     Timeout,
// }
//
// impl std::convert::From<std::io::Error> for Error {
//     fn from(err: std::io::Error) -> Self {
//         Self::Io(err)
//     }
// }
//
// impl std::convert::From<tonic::transport::Error> for Error {
//     fn from(err: tonic::transport::Error) -> Self {
//         Self::Transport(err)
//     }
// }
//
// impl std::convert::From<tonic::Status> for Error {
//     fn from(err: tonic::Status) -> Self {
//         Self::Rpc(err)
//     }
// }

// pub type Result<T> = std::result::Result<T, Error>;
type InterceptedRequestResult = std::result::Result<Request<()>, Status>;

pub struct RetryClient<F: FnMut(Request<()>) -> InterceptedRequestResult> {
    // client: EchoClient<Channel>,
    // client: InterceptedService<Channel, F>,
    #[allow(dead_code)]
    token: Option<MetadataValue<Ascii>>, // for refresh
    client: EchoClient<InterceptedService<Channel, F>>,
}

impl<F: FnMut(Request<()>) -> InterceptedRequestResult> RetryClient<F> {
    async fn bidirectional_streaming_echo_throttle(
        &mut self,
        dur: Duration,
    ) -> anyhow::Result<()> {
        let in_stream = echo_requests_iter().throttle(dur);

        // Result < tonic::Response < tonic::codec::Streaming < super::EchoResponse >>, tonic::Status >

        let response: Response<Streaming<EchoResponse>> =
            self.client.bidirectional_streaming_echo(in_stream).await?;

        let mut resp_stream: Streaming<EchoResponse> = response.into_inner();

        while let Some(received) = resp_stream.next().await {
            match received {
                Err(e) => return Err(anyhow::anyhow!("Error: {:?}", e)),
                Ok(received) => println!("\treceived message: `{}`", received.message),
            }
        }
        Ok(())
    }
}

pub struct RetryConnection {
    token: Option<MetadataValue<Ascii>>,
    channel: Channel,
}

impl RetryConnection {
    pub async fn new(s: &'static str) -> anyhow::Result<Self> {
        let channel = Channel::from_static(s).connect_lazy();
        let token: MetadataValue<Ascii> = "Bearer some-auth-token".parse().unwrap();
        Ok(Self { token: Some(token), channel })
    }
    pub fn client(&self) -> RetryClient<impl FnMut(Request<()>) -> InterceptedRequestResult + '_> {
        let client = EchoClient::with_interceptor(
            self.channel.clone(),
            move |mut req: tonic::Request<()>| {
                if let Some(token) = &self.token {
                    req.metadata_mut().insert("authorization", token.clone());
                }
                Ok(req)
            },
        );
        RetryClient { token: self.token.clone(), client }
    }
    pub async fn bidirectional_streaming_echo_throttle_retry(&self) -> anyhow::Result<()> {
        retry(ExponentialBackoff::default(), || async {
            println!("retrying");
            let mut client = self.client();
            client.bidirectional_streaming_echo_throttle(Duration::from_secs(2))
                .await?;
            Ok(())
        })
            .await
    }

}

fn echo_requests_iter() -> impl Stream<Item=EchoRequest> {
    tokio_stream::iter(1..usize::MAX).map(|i| EchoRequest {
        message: format!("msg {:02}", i),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let retry_connection = RetryConnection::new("http://[::1]:50051").await?;

    println!("Streaming echo:");
    println!("\r\nBidirectional stream echo (kill client with CTLR+C):");
    retry_connection
        .bidirectional_streaming_echo_throttle_retry()
        .await?;

    Ok(())
}
