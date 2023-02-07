use tonic::transport::Channel;
use tower::ServiceBuilder;
use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

use futures_util::future;
use tower::retry::{Policy, Retry};
use tonic::transport::Endpoint;
use std::marker::PhantomData;

// #[derive(Clone)]
// struct LimitPolicy<Req, Res, E> {
//     count: usize,
//     _pd: PhantomData<fn(Req, Res, E)>
// }
//
// impl<Req, Res, E> Policy<Req, Res, E> for LimitPolicy<Req, Res, E> {
//     type Future = future::Ready<Self>;
//     fn retry(&self, _: &Req, result: Result<&Res, &E>) -> Option<Self::Future> {
//         if result.is_err() && self.count > 0 {
//             Some(future::ready(LimitPolicy{count: self.count - 1, _pd: PhantomData}))
//         } else {
//             None
//         }
//     }
//
//     fn clone_request(&self, req: &Req) -> Option<Req> {
//         Some(*req)
//     }
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let channel = Endpoint::from_static("http://[::1]:50051").connect_lazy();
    // let policy = LimitPolicy{count:2, _pd: PhantomData};
    // let service = ServiceBuilder::new()
    //     .retry(policy)
    //     .service(channel);
    // let client = GreeterClient::new(service);

    let channel = Endpoint::from_static("http://[::1]:50051").connect_lazy();
    let mut client = GreeterClient::new(channel);

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
