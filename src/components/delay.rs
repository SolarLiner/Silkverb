use std::{collections::VecDeque, ops::{Deref, Index, IndexMut, Mul}};

use audio::Sample;
use num_traits::Float;

pub struct DelayLine<T> {
    data: VecDeque<T>,
}

impl<T> DelayLine<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
        }
    }
}

impl<T> DelayLine<T> {
    pub fn push_pop(&mut self, val: T) -> Option<T> {
        if self.data.len() == self.data.capacity() {
            let old = self.data.pop_front().unwrap();
            self.data.push_back(val);
            Some(old)
        } else {
            self.data.push_back(val);
            None
        }
    }
}

impl<T> Deref for DelayLine<T> {
    type Target = VecDeque<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Index<usize> for DelayLine<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T> IndexMut<usize> for DelayLine<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

pub struct Delay<T> {
    pub time_s: f64,
    delay: DelayLine<T>,
}

impl<T: Sample> Delay<T> {
    pub fn new(max_length: usize, time_s: f64) -> Self {
        Self {
            time_s,
            delay: DelayLine::new(max_length),
        }
    }
}

impl<T: Sample + Float> Delay<T> {
    pub fn interpolate(&self, srate: impl Into<T>, time_s: impl Into<T>) -> T {
        let len = self.delay.len();
        let off_s = time_s.into() * srate.into();
        let offset = off_s.floor().to_usize().unwrap() % len;
        let fract = off_s.fract();

        let i = self.delay.len() - offset;
        let x0 = self.delay.get(i % len).copied().unwrap_or(T::ZERO);
        let x1 = self.delay.get((i-1)%len).copied().unwrap_or(T::ZERO);
        x0 + fract * (x1 - x0)
    }
}

impl<T: Sample + Float> super::SingleChannelProcess for Delay<T> {
    type T = T;

    fn process_single_channel<'a>(
        &mut self,
        ctx: &super::AudioContext,
        input: audio::Channel<'a, Self::T>,
        output: audio::ChannelMut<'a, Self::T>,
    ) {
        for (inp, out) in input.iter().zip(output.iter_mut()) {
            self.delay.push_pop(inp);
            *out = self.delay.interpolate(ctx.sample_rate, self.time_s)
        }
    }
}
