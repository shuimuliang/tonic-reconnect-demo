# tonic-reconnect-demo
tonic client with reconnect policy

## Project Setup

For this tutorial, we will start by creating a new Rust project with Cargo:

```shell
$ git clone <repo>
$ cd tonic-reconnect-demo
$ cargo init
```

## Defining the HelloWorld service

Our first step is to define the gRPC _service_ and the method _request_ and _response_ types using
[protocol buffers]. We will keep our `.proto` files in a directory in our project's root.
Note that Tonic does not really care where our `.proto` definitions live.

```shell
$ mkdir proto
$ touch proto/helloworld.proto
```

## Application Setup
add our required dependencies to the `Cargo.toml`

## Generating Server and Client code
At the root of your project (not /src), create a `build.rs` file and add the following code:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/helloworld.proto")?;
    Ok(())
}
```

## Writing Client And Server

## Test helloworld without Reconnect Policy
To run the server, run `cargo run --bin helloworld-server`.
To run the client, run `cargo run --bin helloworld-client` in another terminal window.