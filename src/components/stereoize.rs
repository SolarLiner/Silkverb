use std::ops::AddAssign;

use num_traits::Zero;

use super::{spread::Spread, Process};

pub(crate) struct Stereoize<P: Process, const N: usize, const M: usize> {
    process: P,
    spread_in: Spread<P::T, 2, N>,
    spread_out: Spread<P::T, M, 2>,
}

impl<P: Process, const N: usize, const M: usize> Stereoize<P, N, M> {
    pub fn new(process: P) -> Self {
        assert_eq!(P::NIN, N);
        assert_eq!(P::NOUT, M);

        Self {
            process,
            spread_in: Spread::default(),
            spread_out: Spread::default(),
        }
    }

    pub fn process_mut(&mut self) -> &mut P {
        &mut self.process
    }
}

impl<P: Process, const N: usize, const M: usize> Process for Stereoize<P, N, M> where P::T: Zero + AddAssign {
    type T = P::T;
    const NIN: usize = 2;
    const NOUT: usize = 2;

    fn process(&mut self, ctx: &super::AudioContext, input_frame: &[Self::T], output_frame: &mut [Self::T]) {
        let mut pinput = [P::T::zero(); N];
        let mut poutput = [P::T::zero(); M];

        self.spread_in.process(ctx, input_frame, &mut pinput);
        self.process.process(ctx, &pinput, &mut poutput);
        self.spread_out.process(ctx, &poutput, output_frame);
    }
}
