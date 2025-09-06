use anyhow::Result;

#[async_trait::async_trait]
pub trait Updatable: Send + Sync + 'static {
    async fn download<C, D>(&self, on_chunk: C, on_finish: D) -> Result<Vec<u8>>
    where
        C: FnMut(usize, Option<u64>) + Send + 'static,
        D: FnOnce() + Send + 'static;

    fn install(&self, data: Vec<u8>) -> Result<()>;
}

#[async_trait::async_trait]
pub trait UpdateProvider<U: Updatable>: Send + Sync + 'static {
    fn setup(&self) -> Result<Self> where Self: Sized; 
    async fn check(&mut self) -> Result<Option<U>>;
}