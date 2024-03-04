#[tarpc::service]
pub trait RpcService {
    /// Returns a greeting for name.
    async fn hello(name: String) -> String;
    async fn exit();
}
