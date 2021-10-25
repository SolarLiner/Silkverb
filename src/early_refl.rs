use crate::{components::{Process, allpass::Allpass}, seq, seqdef};

pub struct EarlyReflections<const N: usize> {
    delays: seq!(f32, Allpass<N>; Allpass<N>; Allpass<N>; Allpass<N>),
}

impl<const N: usize> EarlyReflections<N> {
    pub fn new(sample_rate: f32) -> Self {
        let delays = seqdef!(Allpass::new((sample_rate * 2.0) as _); Allpass::new((sample_rate * 2.0) as _); Allpass::new((sample_rate * 2.0) as _); Allpass::new((sample_rate * 2.0) as _));
        Self { delays }
    }

    pub fn set_delay_fract(&mut self, f: f32) {
        self.delays.pa.update(f);
        self.delays.pb.pa.update(f);
        self.delays.pb.pb.pa.update(f);
        self.delays.pb.pb.pb.update(f);
    }
}

impl<const N: usize> Process for EarlyReflections<N> {
    type T = f32;
    const NIN: usize = N;
    const NOUT: usize = N;

    #[inline(always)]
    fn process(&mut self, ctx: &crate::components::AudioContext, input_frame: &[Self::T], output_frame: &mut [Self::T]) {
        self.delays.process(ctx, input_frame, output_frame)
    }
}
