//! A small neural-network library based on Torch.
//!
//! This library tries to stay as close as possible to the original
//! Python and C++ implementations.
mod init;
pub use init::{init, Init};

mod var_store;
pub use var_store::{Path, VarStore};

mod module;
pub use module::{Module, ModuleT};

mod linear;
pub use linear::*;

mod conv;
pub use conv::*;

mod conv_transpose;
pub use conv_transpose::*;

mod batch_norm;
pub use batch_norm::*;

mod rnn;
pub use rnn::*;

mod func;
pub use func::*;

mod sequential;
pub use sequential::*;

mod optimizer;
pub use optimizer::{adam, rms_prop, sgd, Adam, Optimizer, OptimizerConfig, RmsProp, Sgd};

/// An identity layer. This just propagates its tensor input as output.
#[derive(Debug)]
pub struct Id();

impl ModuleT for Id {
    fn forward_t(&self, xs: &crate::Tensor, _train: bool) -> crate::Tensor {
        xs.shallow_clone()
    }
}

impl Module for crate::CModule {
    fn forward(&self, xs: &crate::Tensor) -> crate::Tensor {
        self.forward_ts(&[xs]).unwrap()
    }
}
