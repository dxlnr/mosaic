use tonic::async_trait;

#[async_trait]
pub trait Msflp {
    fn handle(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}