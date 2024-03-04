use std::net::{IpAddr, Ipv6Addr};
use tarpc::tokio_serde::formats::Json;
use rootspace::engine::systems::rpc::service::RpcServiceClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), 7919);
    let mut connection = tarpc::serde_transport::tcp::connect(server_addr, Json::default);
    connection.config_mut().max_frame_length(usize::MAX);

    let client = RpcServiceClient::new(tarpc::client::Config::default(), connection.await?).spawn();

    let response = client.hello(tarpc::context::current(), String::from("nausicaea")).await?;

    println!("{}", response);

    Ok(())
}
