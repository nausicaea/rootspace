use std::net::{IpAddr, Ipv6Addr};

use clap::{Parser, Subcommand};
use rootspace::RpcServiceClient;
use rootspace::systems::rpc::graphics_info::GraphicsInfoCategory;
use tarpc::tokio_serde::formats::Bincode;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short = 'H', long, help = "The host (name or IP) to connect to", default_value_t = IpAddr::V6(Ipv6Addr::LOCALHOST))]
    host: IpAddr,
    #[arg(short, long, help = "The port to connect to", default_value_t = 7919)]
    port: u16,
    #[arg(long, help = "The maximum packet size", default_value_t = 8388608)]
    max_frame_length: usize,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Exit,
    Perf,
    Gfx {
        category: GraphicsInfoCategory,
    },
    LoadScene {
        #[arg(short, long, help = "The asset group", default_value = "scenes")]
        group: String,
        name: String,
    },
}

fn main() -> anyhow::Result<()> {
    smol::block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();
    let server_addr = (args.host, args.port);
    let mut connection = tarpc::serde_transport::tcp::connect(server_addr, Bincode::default);
    connection.config_mut().max_frame_length(args.max_frame_length);

    let client = RpcServiceClient::new(tarpc::client::Config::default(), connection.await?).spawn();
    let context = tarpc::context::current();

    match args.command {
        Command::Exit => client.exit(context).await??,
        Command::Perf => println!("{}", client.perf(context).await??),
        Command::Gfx { category } => {
            println!("{}", client.graphics_info(context, category).await??);
        }
        Command::LoadScene { group, name } => client.load_scene(context, group, name).await??,
    }

    Ok(())
}
