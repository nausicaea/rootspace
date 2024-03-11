use crate::engine::resources::statistics::Statistics;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub(crate) enum RpcMessage {
    StatsRequest(Sender<Statistics>),
    LoadScene {
        tx: Sender<Result<(), anyhow::Error>>,
        group: String,
        name: String,
    },
    Exit,
}
