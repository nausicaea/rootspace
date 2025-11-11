use tokio::sync::oneshot::Sender;

use crate::resources::statistics::Statistics;
use crate::systems::rpc::graphics_info::{GraphicsInfo, GraphicsInfoCategory};

#[derive(Debug)]
pub enum RpcMessage {
    StatsRequest(Sender<Statistics>),
    GraphicsInfo {
        tx: Sender<GraphicsInfo>,
        category: GraphicsInfoCategory,
    },
    LoadScene {
        tx: Sender<Result<(), anyhow::Error>>,
        group: String,
        name: String,
    },
    Exit,
}
