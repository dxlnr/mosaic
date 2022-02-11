use crate::core::model::Model;

/// FedAvg algorithm based on McMahan et al. Communication-Efficient Learning of Deep Networks
/// from Decentralized Data (https://arxiv.org/abs/1602.05629)
pub trait FedAvg
where
    Self: Clone + Send + Sync + 'static,
{
    fn aggregate(&mut self) -> Model;
}

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

// pub trait Strategy: FedAvg + FedAdam + FedAdaGrad + FedYogi {}
