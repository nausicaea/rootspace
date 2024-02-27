use async_std::task;

trait A {
    async fn t(&self) -> bool;
}

struct B(bool);

impl A for B {
    async fn t(&self) -> bool {
        self.0
    }
}

fn main() {
    task::block_on(B(true).t());
}
