use super::Process;
use audio::Sample;
use num_traits::{Float, One};

pub(crate) struct DryWet<P: Process, const N: usize> {
    pub amount: P::T,
    pub process: P,
}

impl<P: Process, const N: usize> DryWet<P, N> {
    pub fn new(process: P) -> Self {
        assert_eq!(N, P::NIN);
        assert_eq!(P::NIN, P::NOUT);
        Self {
            amount: P::T::ZERO,
            process,
        }
    }

    pub fn set_amount(&mut self, amount: P::T) {
        self.amount = amount;
    }
}

impl<P: Process, const N: usize> Process for DryWet<P, N>
where
    P::T: Float,
{
    type T = P::T;
    const NIN: usize = N;
    const NOUT: usize = N;

    fn process(
        &mut self,
        ctx: &super::AudioContext,
        input_frame: &[Self::T],
        output_frame: &mut [Self::T],
    ) {
        let mut result = [P::T::ZERO; N];
        self.process.process(ctx, input_frame, &mut result);
        for i in 0..N {
            output_frame[i] =
                input_frame[i] * (P::T::one() - self.amount) + result[i] * self.amount;
        }
    }
}
