use crate::components::Process;
use crate::components::SingleChannelProcess;
use crate::AudioContext;

pub struct Parallel<P, const N: usize> {
    data: Vec<P>,
}

impl<P, const N: usize> Parallel<P, N> {
    pub fn new(init: impl FnMut(usize) -> P) -> Self {
        Self {
            data: (0..N).map(init).collect(),
        }
    }

    pub fn update(&mut self, mut update: impl FnMut(&mut P)) {
        for inner in &mut self.data {
            update(inner);
        }
    }
}

impl<P: SingleChannelProcess, const N: usize> Process for Parallel<P, N> {
    type T = P::T;
    const NIN: usize = N;
    const NOUT: usize = N;

    #[inline(always)]
    fn process(
        &mut self,
        ctx: &AudioContext,
        input_frame: &[Self::T],
        output_frame: &mut [Self::T],
    ) {
        for (i, (out, inp)) in output_frame
            .iter_mut()
            .zip(input_frame.iter().copied())
            .enumerate()
        {
            *out = self.data[i].process_single_channel(ctx, inp);
        }
    }
}
