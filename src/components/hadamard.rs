use std::ops::Neg;

use nalgebra::ClosedDiv;
use nalgebra::{ClosedAdd, ClosedMul, DMatrix, SMatrix, SVectorSlice, SVectorSliceMut, Scalar};
use num_traits::{Float, One, Zero};

use super::AudioContext;

use super::Process;

pub struct Hadamard<T, const N: usize> {
    transfer: SMatrix<T, N, N>,
}

impl<
        T: Scalar
            + Float
            + ClosedAdd
            + ClosedMul
            + ClosedDiv
            + Neg<Output = T>
            + std::ops::Add
            + num_traits::Zero,
        const N: usize,
    > Hadamard<T, N>
{
    pub fn new() -> Self {
        Self {
            transfer: SMatrix::from_column_slice(hadamard(N).as_slice()),
        }
    }
}

impl<T: audio::Sample + Scalar + Float + ClosedAdd + ClosedMul + ClosedDiv, const N: usize> Process
    for Hadamard<T, N>
{
    type T = T;
    const NIN: usize = 2usize.pow(N as _);
    const NOUT: usize = 2usize.pow(N as _);

    #[inline(always)]
    fn process(
        &mut self,
        _: &AudioContext,
        inputs: &[<Self as Process>::T],
        outputs: &mut [<Self as Process>::T],
    ) {
        let invec = SVectorSlice::<T, N>::from_slice(inputs);
        let mut outvec = SVectorSliceMut::<T, N>::from_slice(outputs);
        self.transfer.mul_to(&invec, &mut outvec);
        outvec
            .iter_mut()
            .for_each(|x| *x /= T::from(N).unwrap().sqrt());
    }
}

// FIXME: Wait for const int operations to replace this with a proper generic function
fn hadamard<T: Scalar + One + Neg<Output = T>>(n: usize) -> DMatrix<T> {
    if n == 1 {
        // DMatrix::from_row_slice(2, 2, &[T::one(), T::one(), T::one(), T::one().neg()])
        DMatrix::from_fn(1, 1, |_, _| T::one())
    } else {
        debug_assert_eq!(n, n.next_power_of_two());
        let mut result = DMatrix::from_fn(n, n, |_, _| T::one());
        let base = hadamard(n / 2);
        let inverted = base.map(T::neg);
        result.slice_mut((0, 0), (n / 2, n / 2)).copy_from(&base);
        result
            .slice_mut((0, n / 2), (n / 2, n / 2))
            .copy_from(&base);
        result
            .slice_mut((n / 2, 0), (n / 2, n / 2))
            .copy_from(&base);
        result
            .slice_mut((n / 2, n / 2), (n / 2, n / 2))
            .copy_from(&inverted);
        result
    }
}

#[test]
fn test_h1() {
    let h1 = hadamard::<f32>(1);
    println!("{:?}", h1);
    assert_eq!(h1.as_slice(), &[1.0]);
}

#[test]
fn test_h2() {
    let h2 = hadamard::<f32>(2);
    println!("{:?}", h2);
    assert_eq!(h2.as_slice(), &[1.0, 1.0, 1.0, -1.0]);
}

#[test]
#[should_panic]
fn test_h3() {
    let _ = hadamard::<f32>(3);
}

#[test]
#[rustfmt::skip]
fn test_h4() {
    let h4 = hadamard::<f32>(4);
    println!("{:?}", h4);

    assert_eq!(h4.as_slice(), &[
               1., 1., 1., 1.,
               1.,-1., 1.,-1.,
               1., 1.,-1.,-1.,
               1.,-1.,-1., 1.,
    ]);
}
