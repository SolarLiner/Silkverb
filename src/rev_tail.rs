use std::iter::Rev;

use audio::Sample;
use nalgebra::ClosedMul;

use crate::components::{
    allpass::Allpass, feedback::Feedback, parallel::Parallel, seq::Sequence, Process,
    SingleChannelProcess,
};

struct Gain<T> {
    gain: T,
}

impl<T: Sample + ClosedMul> SingleChannelProcess for Gain<T> {
    type T = T;

    fn process_single_channel(
        &mut self,
        _: &crate::components::AudioContext,
        value: Self::T,
    ) -> Self::T {
        value * self.gain
    }
}

pub(crate) struct ReverbTail<const N: usize> {
    tank: Feedback<Sequence<f32, Allpass<N>, Allpass<N>>, Parallel<Gain<f32>, N>, N>,
}

impl<const N: usize> ReverbTail<N> {
    pub fn new(samplerate: f32) -> Self {
        Self {
            tank: Feedback::new(
                Sequence::new(
                    Allpass::new((0.3 * samplerate) as _),
                    Allpass::new((1.4 * samplerate) as _),
                ),
                Parallel::new(|_| Gain { gain: 0.4 }),
            ),
        }
    }

    pub fn update_feedback(&mut self, feedback: f32) {
        self.tank.backward_mut().update(|g| g.gain = feedback);
    }

    pub fn update_size(&mut self, size: f32) {
        let seq = self.tank.forward_mut();
        seq.pa.update(size);
        seq.pb.update(size);
    }
}

impl<const N: usize> Process for ReverbTail<N> {
    type T = f32;
    const NIN: usize = N;
    const NOUT: usize = N;

    fn process(
        &mut self,
        ctx: &crate::components::AudioContext,
        input_frame: &[Self::T],
        output_frame: &mut [Self::T],
    ) {
        self.tank.process(ctx, input_frame, output_frame)
    }
}
