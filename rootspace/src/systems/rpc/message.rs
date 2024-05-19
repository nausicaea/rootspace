use tokio::sync::oneshot::Sender;

use crate::resources::statistics::Statistics;

#[derive(Debug)]
pub enum RpcMessage {
    StatsRequest(Sender<Statistics>),
    LoadScene {
        tx: Sender<Result<(), anyhow::Error>>,
        group: String,
        name: String,
    },
    Exit,
}
