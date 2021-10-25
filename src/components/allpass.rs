use super::{delay::Delay, hadamard::Hadamard, parallel::Parallel, Process};

pub struct Allpass<const N: usize> {
    parallel: Parallel<Delay<f32>, N>,
    hadamard: Hadamard<f32, N>,
}

impl<const N: usize> Allpass<N> {
    pub fn new(max_samples: usize) -> Self {
        Self {
            parallel: Parallel::new(|i| {
                Delay::new((((i as f32 + 1.0) / N as f32).powi(2) * max_samples as f32) as _)
            }),
            hadamard: Hadamard::new(),
        }
    }

    pub fn update(&mut self, fac: f32) {
        self.parallel
            .update(|d| d.pos = fac);
    }
}

impl<const N: usize> Process for Allpass<N> {
    type T = f32;
    const NIN: usize = N;
    const NOUT: usize = N;

    #[inline(always)]
    fn process(
        &mut self,
        ctx: &super::AudioContext,
        input_frame: &[Self::T],
        output_frame: &mut [Self::T],
    ) {
        let mut ptemp = [0.0; N];

        self.parallel.process(ctx, input_frame, &mut ptemp);
        self.hadamard.process(ctx, &ptemp, output_frame);
    }
}
