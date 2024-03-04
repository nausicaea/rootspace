use crate::engine::resources::statistics::Statistics;

#[tarpc::service]
pub trait RpcService {
    /// Returns a greeting for name.
    async fn hello(name: String) -> String;
    /// Requests rootspace to exit
    async fn exit();
    /// Requests performance statistics
    async fn perf() -> Statistics;
}
