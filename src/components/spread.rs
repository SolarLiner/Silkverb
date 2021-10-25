use num_traits::Zero;

use crate::AudioContext;
use std::{marker::PhantomData, ops::AddAssign};

use super::Process;

#[derive(Default)]
pub struct Spread<T, const In: usize, const Out: usize> {
    __phantom: PhantomData<T>,
}

impl<T: audio::Sample + Zero + AddAssign, const In: usize, const Out: usize> Process for Spread<T, In, Out> {
    type T = T;
    const NIN: usize = In;
    const NOUT: usize = Out;

    #[inline(always)]
    fn process(
        &mut self,
        _: &AudioContext,
        inputs: &[<Self as Process>::T],
        outputs: &mut [<Self as Process>::T],
    ) {
        outputs.iter_mut().for_each(T::set_zero);
        for i in 0..(In.max(Out)) {
            outputs[i % Out] += inputs[i % In];
        }
    }
}
