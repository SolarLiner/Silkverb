use audio::Sample;

pub(crate) mod allpass;
pub(crate) mod delay;
pub(crate) mod hadamard;
pub(crate) mod parallel;
pub(crate) mod seq;
pub(crate) mod spread;
pub(crate) mod stereoize;

pub(crate) struct AudioContext {
    pub(crate) sample_rate: f32,
    pub(crate) sample_count: u128,
}

pub(crate) trait Process {
    type T: Sample;
    const NIN: usize;
    const NOUT: usize;

    fn process(
        &mut self,
        ctx: &AudioContext,
        input_frame: &[Self::T],
        output_frame: &mut [Self::T],
    );
}

pub(crate) trait SingleChannelProcess {
    type T: Sample;

    fn process_single_channel(&mut self, ctx: &AudioContext, value: Self::T) -> Self::T;
}

impl<P: SingleChannelProcess> Process for P {
    type T = P::T;
    const NIN: usize = 1;
    const NOUT: usize = 1;

    fn process(
        &mut self,
        ctx: &AudioContext,
        input_frame: &[Self::T],
        output_frame: &mut [Self::T],
    ) {
        output_frame[0] = self.process_single_channel(ctx, input_frame[0])
    }
}
