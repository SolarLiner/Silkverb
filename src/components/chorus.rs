use audio::Sample;
use num_traits::{Float, FloatConst, FromPrimitive};
use std::fmt::Debug;

use super::{delay::Delay, ext::DurationFloatExt, SingleChannelProcess};

#[derive(Copy, Clone, Debug)]
pub struct Hz<T>(T);

impl<T> Hz<T> {
    pub fn from_frequency(freq: T) -> Self {
        Self(freq)
    }

    pub fn to_freq(self) -> T {
        self.0
    }
}

impl<T: Float> Hz<T> {
    pub fn phase(&self, t: T) -> T {
        (t * self.0).fract()
    }
}

impl<T: Float> Hz<T> {
    pub fn from_seconds(secs: T) -> Self {
        Self(secs.recip())
    }

    pub fn to_seconds(self) -> T {
        self.0.recip()
    }
}

pub struct Chorus<T> {
    amplitude: T,
    freq: Hz<T>,
    pos: T,
    delay: Delay<T>,
}

impl<T> Chorus<T> {
    pub fn set_amplitude(&mut self, t: T) {
        self.amplitude = t
    }

    pub fn set_frequency(&mut self, freq: impl Into<Hz<T>>) {
        self.freq = freq.into();
    }

    pub fn set_pos(&mut self, p: T) {
        self.pos = p;
    }
}

impl<T: Debug + Sample + Float + FromPrimitive> Chorus<T> {
    pub fn new(max_delay: usize) -> Self {
        Self {
            amplitude: T::zero(),
            freq: Hz::from_frequency(T::one()),
            pos: T::zero(),
            delay: Delay::new(max_delay),
        }
    }

    fn tick(&mut self, ctx: &super::AudioContext) {
        self.pos = self.pos + ctx.tick_length().as_seconds::<T>() * self.freq.to_freq();
    }
}

impl<T: Debug + Sample + Float + FloatConst + FromPrimitive> SingleChannelProcess for Chorus<T> {
    type T = T;

    fn process_single_channel(&mut self, ctx: &super::AudioContext, value: Self::T) -> Self::T {
        let two = T::one().add(T::one());
        let half = two.recip();
        self.delay.pos = self.pos.fract().mul(T::TAU()).sin() * self.amplitude / two + half;
        self.tick(ctx);
        self.delay.process_single_channel(ctx, value)
    }
}
