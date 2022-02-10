use crate::core::model::Model;

/// FedAdam algorithm based on Reddi et al. ADAPTIVE FEDERATED OPTIMIZATION
/// (https://arxiv.org/pdf/2003.00295.pdf)
pub trait FedAdam
where
    Self: Clone + Send + Sync + 'static,
{
    fn adapt(&mut self) -> Model;
}

/// FedAdaGrad algorithm based on Reddi et al. ADAPTIVE FEDERATED OPTIMIZATION
/// (https://arxiv.org/pdf/2003.00295.pdf)
pub trait FedAdaGrad
where
    Self: Clone + Send + Sync + 'static,
{
    fn adapt(&mut self) -> Model;
}
/// FedYogi algorithm based on Reddi et al. ADAPTIVE FEDERATED OPTIMIZATION
/// (https://arxiv.org/pdf/2003.00295.pdf)
pub trait FedYogi
where
    Self: Clone + Send + Sync + 'static,
{
    fn adapt(&mut self) -> Model;
}
