//! A Torch tensor.
use crate::{Device, Kind};
use failure::Fallible;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

mod iter;
mod npy;

pub use super::wrappers::tensor::{no_grad, no_grad_guard, NoGradGuard, Reduction, Tensor};

macro_rules! impl_op {
    ($trait:ident, $rhs:ident, $func:ident, $op:ident) => {
        impl $trait<$rhs> for Tensor {
            type Output = Tensor;

            fn $func(self, rhs: $rhs) -> Self::Output {
                self.$op(&rhs)
            }
        }

        impl $trait<&$rhs> for Tensor {
            type Output = Tensor;

            fn $func(self, rhs: &$rhs) -> Self::Output {
                self.$op(rhs)
            }
        }

        impl<'a> $trait<&$rhs> for &'a Tensor {
            type Output = Tensor;

            fn $func(self, rhs: &$rhs) -> Self::Output {
                self.$op(rhs)
            }
        }

        impl $trait<$rhs> for &Tensor {
            type Output = Tensor;

            fn $func(self, rhs: $rhs) -> Self::Output {
                self.$op(&rhs)
            }
        }
    };
}

macro_rules! impl_op_basic {
    /* rev such that rev(op(b, a)) = op(a, b) */
    ($trait:ident, $func:ident, $op:ident, $rev:ident) => {
        impl $trait<i64> for Tensor {
            type Output = Tensor;

            fn $func(self, rhs: i64) -> Self::Output {
                self.$op(rhs)
            }
        }

        impl $trait<f64> for Tensor {
            type Output = Tensor;

            fn $func(self, rhs: f64) -> Self::Output {
                self.$op(rhs)
            }
        }

        impl $trait<i64> for &Tensor {
            type Output = Tensor;

            fn $func(self, rhs: i64) -> Self::Output {
                self.$op(rhs)
            }
        }

        impl $trait<f64> for &Tensor {
            type Output = Tensor;

            fn $func(self, rhs: f64) -> Self::Output {
                self.$op(rhs)
            }
        }

        impl $trait<Tensor> for i64 {
            type Output = Tensor;

            fn $func(self, rhs: Tensor) -> Self::Output {
                $rev(rhs.$op(self))
            }
        }

        impl $trait<Tensor> for f64 {
            type Output = Tensor;

            fn $func(self, rhs: Tensor) -> Self::Output {
                $rev(rhs.$op(self))
            }
        }

        impl $trait<&Tensor> for i64 {
            type Output = Tensor;

            fn $func(self, rhs: &Tensor) -> Self::Output {
                $rev(rhs.$op(self))
            }
        }

        impl $trait<&Tensor> for f64 {
            type Output = Tensor;

            fn $func(self, rhs: &Tensor) -> Self::Output {
                $rev(rhs.$op(self))
            }
        }
    };
}

macro_rules! impl_op_assign {
    ($trait:ident, $rhs:ident, $func:ident, $op:ident) => {
        impl $trait<$rhs> for Tensor {
            fn $func(&mut self, rhs: $rhs) {
                let _ = self.$op(&rhs);
            }
        }

        impl $trait<&$rhs> for Tensor {
            fn $func(&mut self, rhs: &$rhs) {
                let _ = self.$op(rhs);
            }
        }
    };
}

macro_rules! impl_op_assign_basic {
    ($trait:ident, $func:ident, $op:ident) => {
        impl $trait<i64> for Tensor {
            fn $func(&mut self, rhs: i64) {
                let _ = self.$op(rhs);
            }
        }
        impl $trait<f64> for Tensor {
            fn $func(&mut self, rhs: f64) {
                let _ = self.$op(rhs);
            }
        }
    };
}

fn id<T>(v: T) -> T {
    v
}

fn neg(t: Tensor) -> Tensor {
    t.neg()
}

fn inv(t: Tensor) -> Tensor {
    t.pow(-1)
}

impl_op!(Add, Tensor, add, g_add);
impl_op_basic!(Add, add, g_add1, id);
impl_op_assign!(AddAssign, Tensor, add_assign, g_add_);
impl_op_assign_basic!(AddAssign, add_assign, g_add_1);

impl_op!(Mul, Tensor, mul, g_mul);
impl_op_basic!(Mul, mul, g_mul1, id);
impl_op_assign!(MulAssign, Tensor, mul_assign, g_mul_);
impl_op_assign_basic!(MulAssign, mul_assign, g_mul_1);

impl_op!(Div, Tensor, div, g_div);
impl_op_basic!(Div, div, g_div1, inv);
impl_op_assign!(DivAssign, Tensor, div_assign, g_div_);
impl_op_assign_basic!(DivAssign, div_assign, g_div_1);

impl_op!(Sub, Tensor, sub, g_sub);
impl_op_basic!(Sub, sub, g_sub1, neg);
impl_op_assign!(SubAssign, Tensor, sub_assign, g_sub_);
impl_op_assign_basic!(SubAssign, sub_assign, g_sub_1);

impl Neg for Tensor {
    type Output = Tensor;

    fn neg(self) -> Tensor {
        self.f_neg().unwrap()
    }
}

impl Neg for &Tensor {
    type Output = Tensor;

    fn neg(self) -> Tensor {
        self.f_neg().unwrap()
    }
}

impl<T: crate::kind::T> From<&[T]> for Tensor {
    fn from(v: &[T]) -> Tensor {
        Tensor::of_slice(v)
    }
}

impl<T: crate::kind::T> From<T> for Tensor {
    fn from(v: T) -> Tensor {
        Tensor::of_slice(&[v]).view(&[])
    }
}

impl std::fmt::Debug for Tensor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (is_int, is_float) = match self.kind() {
            Kind::Int | Kind::Int8 | Kind::Uint8 | Kind::Int16 | Kind::Int64 => (true, false),
            Kind::Half | Kind::Float | Kind::Double => (false, true),
            Kind::ComplexHalf | Kind::ComplexFloat | Kind::ComplexDouble => (false, false),
        };
        match (self.size().as_slice(), is_int, is_float) {
            ([], true, false) => write!(f, "[{}]", i64::from(self)),
            ([s], true, false) if *s < 10 => write!(f, "{:?}", Vec::<i64>::from(self)),
            ([], false, true) => write!(f, "[{}]", f64::from(self)),
            ([s], false, true) if *s < 10 => write!(f, "{:?}", Vec::<f64>::from(self)),
            _ => write!(f, "Tensor[{:?}, {:?}]", self.size(), self.kind()),
        }
    }
}

impl Tensor {
    /// Casts a tensor to a specified kind.
    pub fn to_kind(&self, kind: Kind) -> Tensor {
        self.totype(kind)
    }

    pub fn f_to_kind(&self, kind: Kind) -> Fallible<Tensor> {
        self.f_totype(kind)
    }

    pub fn nll_loss(&self, targets: &Tensor) -> Tensor {
        self.g_nll_loss::<Tensor>(targets, None, Reduction::Mean, -100)
    }
}

macro_rules! from_tensor {
    ($typ:ident, $zero:expr, $kind:ident) => {
        impl From<&Tensor> for Vec<$typ> {
            fn from(tensor: &Tensor) -> Vec<$typ> {
                let numel = tensor.numel();
                let mut vec = vec![$zero; numel as usize];
                tensor.to_kind(Kind::$kind).copy_data(&mut vec, numel);
                vec
            }
        }

        impl From<&Tensor> for Vec<Vec<$typ>> {
            fn from(tensor: &Tensor) -> Vec<Vec<$typ>> {
                let first_dim = tensor.size()[0];
                (0..first_dim)
                    .map(|i| Vec::<$typ>::from(tensor.get(i)))
                    .collect()
            }
        }

        impl From<&Tensor> for Vec<Vec<Vec<$typ>>> {
            fn from(tensor: &Tensor) -> Vec<Vec<Vec<$typ>>> {
                let first_dim = tensor.size()[0];
                (0..first_dim)
                    .map(|i| Vec::<Vec<$typ>>::from(tensor.get(i)))
                    .collect()
            }
        }

        impl From<&Tensor> for $typ {
            fn from(tensor: &Tensor) -> $typ {
                let numel = tensor.numel();
                if numel != 1 {
                    panic!("expected exactly one element, got {}", numel)
                }
                Vec::from(tensor)[0]
            }
        }

        impl From<Tensor> for Vec<$typ> {
            fn from(tensor: Tensor) -> Vec<$typ> {
                Vec::<$typ>::from(&tensor)
            }
        }

        impl From<Tensor> for Vec<Vec<$typ>> {
            fn from(tensor: Tensor) -> Vec<Vec<$typ>> {
                Vec::<Vec<$typ>>::from(&tensor)
            }
        }

        impl From<Tensor> for Vec<Vec<Vec<$typ>>> {
            fn from(tensor: Tensor) -> Vec<Vec<Vec<$typ>>> {
                Vec::<Vec<Vec<$typ>>>::from(&tensor)
            }
        }

        impl From<Tensor> for $typ {
            fn from(tensor: Tensor) -> $typ {
                $typ::from(&tensor)
            }
        }
    };
}

from_tensor!(f64, 0f64, Double);
from_tensor!(f32, 0f32, Float);
from_tensor!(i64, 0i64, Int64);
from_tensor!(i32, 0i32, Int);
from_tensor!(i8, 0i8, Int8);
from_tensor!(u8, 0u8, Uint8);

impl Tensor {
    /// Computes the cross-entropy loss based on some logits and targets.
    pub fn cross_entropy_for_logits(&self, targets: &Tensor) -> Tensor {
        self.log_softmax(-1).nll_loss(&targets)
    }

    /// Returns the average accuracy for some given logits assuming that
    /// targets represent ground-truth.
    pub fn accuracy_for_logits(&self, targets: &Tensor) -> Tensor {
        self.argmax(-1, false)
            .eq1(&targets)
            .to_kind(Kind::Float)
            .mean()
    }

    pub fn random_batch(&self, batch_size: i64) -> Tensor {
        let len: i64 = self.size()[0];
        let index = Tensor::randint(len, &[batch_size], crate::wrappers::kind::INT64_CPU);
        self.index_select(0, &index)
    }

    pub fn random_batch2(
        t1: &Tensor,
        t2: &Tensor,
        batch_size: i64,
        device: Device,
    ) -> (Tensor, Tensor) {
        let len1: i64 = t1.size()[0];
        let len2: i64 = t2.size()[0];
        if len1 != len2 {
            panic!(
                "random_batch2: shape mismatch {:?} {:?}",
                t1.size(),
                t2.size()
            )
        }
        let index = Tensor::randint(len1, &[batch_size], crate::wrappers::kind::INT64_CPU);
        let batch1 = t1.index_select(0, &index).to_device(device);
        let batch2 = t2.index_select(0, &index).to_device(device);
        (batch1, batch2)
    }

    /// Moves a tensor to a specified device.
    pub fn to_device(&self, device: Device) -> Tensor {
        self.to(device)
    }

    pub fn f_to_device(&self, device: Device) -> Fallible<Tensor> {
        self.f_to(device)
    }

    pub fn avg_pool2d_default(&self, ksize: i64) -> Tensor {
        self.avg_pool2d(&[ksize, ksize], &[ksize, ksize], &[0, 0], false, true)
    }

    pub fn max_pool2d_default(&self, ksize: i64) -> Tensor {
        self.max_pool2d(&[ksize, ksize], &[ksize, ksize], &[0, 0], &[1, 1], false)
    }

    /// Flattens a tensor.
    ///
    /// This returns a flattened version of the given tensor. The first dimension
    /// is preserved as it is assumed to be the mini-batch dimension.
    pub fn flat_view(&self) -> Tensor {
        let batch_size = self.size()[0] as i64;
        self.view(&[batch_size, -1])
    }

    /// Converts a tensor to a one-hot encoded version.
    ///
    /// If the input has a size [N1, N2, ..., Nk], the returned tensor has a size
    /// [N1, ..., Nk, labels]. The returned tensor uses float values.
    /// Elements of the input vector are expected to be between 0 and labels-1.
    pub fn onehot(&self, labels: i64) -> Tensor {
        Tensor::zeros(
            &[self.size(), vec![labels]].concat(),
            crate::wrappers::kind::FLOAT_CPU,
        )
        .scatter_(
            -1,
            &self.unsqueeze(-1).to_kind(Kind::Int64),
            &Tensor::ones(&[], crate::wrappers::kind::FLOAT_CPU),
        )
    }

    /// Copies a tensor to a newly allocated tensor using the same shape and device.
    pub fn copy(&self) -> Tensor {
        let mut result = self.zeros_like();
        result.copy_(&self);
        result
    }
}

impl std::iter::Sum for Tensor {
    fn sum<I: Iterator<Item = Tensor>>(mut iter: I) -> Tensor {
        match iter.next() {
            None => Tensor::from(0.),
            Some(t) => iter.fold(t, |acc, x| x + acc),
        }
    }
}

impl<'a> std::iter::Sum<&'a Tensor> for Tensor {
    fn sum<I: Iterator<Item = &'a Tensor>>(mut iter: I) -> Tensor {
        match iter.next() {
            None => Tensor::from(0.),
            Some(t) => iter.fold(t.shallow_clone(), |acc, x| x + acc),
        }
    }
}

impl PartialEq for Tensor {
    fn eq(&self, other: &Tensor) -> bool {
        if self.size() != other.size() {
            return false;
        }
        match self.f_eq1(&other) {
            Err(_) => false,
            Ok(v) => match v.f_all() {
                Err(_) => false,
                Ok(v) => i64::from(v) > 0,
            },
        }
    }
}
