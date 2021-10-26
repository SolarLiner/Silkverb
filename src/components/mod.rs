use std::time::Duration;

use audio::Sample;
use num_traits::ToPrimitive;

pub(crate) mod allpass;
pub mod chorus;
pub(crate) mod delay;
pub(crate) mod feedback;
pub(crate) mod hadamard;
pub(crate) mod parallel;
pub(crate) mod seq;
pub(crate) mod spread;
pub(crate) mod stereoize;

mod ext {
    use std::time::Duration;

    use num_traits::{Float, FromPrimitive};

    pub trait DurationFloatExt: Sized {
        fn from_seconds<T: Float>(&self, secs: T) -> Self;
        fn as_seconds<T: Float + FromPrimitive>(&self) -> T;
    }

    impl DurationFloatExt for Duration {
        fn from_seconds<T: Float>(&self, secs: T) -> Self {
            Duration::from_nanos((secs * T::one().powi(9)).to_u64().unwrap())
        }

        fn as_seconds<T: Float + FromPrimitive>(&self) -> T {
            T::from_u64(self.as_secs()).unwrap()
                + T::from_u32(self.subsec_nanos()).unwrap() / T::one().powi(9)
        }
    }
}

pub(crate) struct AudioContext {
    pub(crate) sample_rate: f32,
    pub(crate) sample_count: u128,
}

impl AudioContext {
    pub fn get_runtime(&self) -> Duration {
        let nanos = self.sample_count as u64 / (self.sample_rate * 1e9) as u64;
        Duration::from_nanos(nanos)
    }

    pub fn tick_length(&self) -> Duration {
        let tick = (1e9 / self.sample_rate as f64).floor() as u64;
        Duration::from_nanos(tick)
    }
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
