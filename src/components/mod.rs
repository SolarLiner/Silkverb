use std::time::Duration;

use audio::{Channel, ChannelMut, Channels, ChannelsMut, Sample};

pub mod delay;

pub(crate) struct AudioContext {
    pub(crate) sample_rate: f64,
    pub(crate) sample_count: u128,
}

pub(crate) trait Process {
    type T: Sample;
    const NIN: usize;
    const NOUT: usize;

    fn process<'a, BIn: Channels<Self::T> + 'a, BOut: ChannelsMut<Self::T> + 'a>(
        &mut self,
        ctx: &AudioContext,
        input: &'a BIn,
        output: &'a mut BOut,
    );
}

pub(crate) trait SingleChannelProcess {
    type T: Sample;

    fn process_single_channel<'a>(
        &mut self,
        ctx: &AudioContext,
        input: Channel<'a, Self::T>,
        output: ChannelMut<'a, Self::T>,
    );
}

impl<P: SingleChannelProcess> Process for P {
    type T = P::T;
    const NIN: usize = 1;
    const NOUT: usize = 1;

    fn process<'a, BIn: Channels<Self::T> + 'a, BOut: ChannelsMut<Self::T> + 'a>(
        &mut self,
        ctx: &AudioContext,
        input: &'a BIn,
        output: &'a mut BOut,
    ) {
        P::process_single_channel(self, ctx, input.channel(0), output.channel_mut(0))
    }
}

pub struct Parallel<P, const N: usize> {
    data: Vec<P>,
}

impl<P, const N: usize> Parallel<P, N> {
    pub fn new(mut init: impl FnMut() -> P) -> Self {
        Self {
            data: (0..N).map(|_| init()).collect()
        }
    }
}

impl<P: SingleChannelProcess, const N: usize> Process for Parallel<P, N> {
    type T = P::T;
    const NIN: usize = N;
    const NOUT: usize = N;

    fn process<'a, BIn: Channels<Self::T> + 'a, BOut: ChannelsMut<Self::T> + 'a>(
        &mut self,
        ctx: &AudioContext,
        input: &'a BIn,
        output: &'a mut BOut,
    ) {
        for n in 0..N {
            self.data[n].process_single_channel(ctx, input.channel(n), output.channel_mut(n));
        }
    }
}
