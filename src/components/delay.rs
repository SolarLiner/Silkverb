use std::{
    collections::VecDeque,
    iter::FromIterator,
    ops::{Deref, Index, IndexMut},
};

use audio::Sample;
use num_traits::{Float, FromPrimitive, One};

use super::SingleChannelProcess;

pub struct DelayLine<T> {
    data: VecDeque<T>,
}

impl<T: Sample> DelayLine<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::from_iter(std::iter::repeat(T::ZERO).take(capacity)),
        }
    }
}

impl<T: Sample> DelayLine<T> {
    pub fn push_pop(&mut self, val: T) -> T {
        let old = self.data.pop_front().unwrap_or(T::ZERO);
        self.data.push_back(val);
        old
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
    pub pos: T,
    delay: DelayLine<T>,
}

impl<T: Sample + One> Delay<T> {
    pub fn new(max_length: usize) -> Self {
        Self {
            pos: T::one(),
            delay: DelayLine::new(max_length),
        }
    }
}

impl<T: Sample + Float + FromPrimitive> Delay<T> {
    pub fn interpolate(&self, f: T) -> T {
        let len = self.delay.len();
        let v: T = T::from_usize(len).unwrap() * f;
        let offset = v.floor().to_usize().unwrap() % len;
        let fract = v.fract();

        let i = len - offset;
        let x0 = self.delay.get(i % len).copied().unwrap_or(T::ZERO);
        let x1 = self.delay.get((i - 1) % len).copied().unwrap_or(T::ZERO);
        x0 + fract * (x1 - x0)
    }
}

impl<T: Sample + Float + FromPrimitive> SingleChannelProcess for Delay<T> {
    type T = T;

    fn process_single_channel(&mut self, ctx: &super::AudioContext, value: Self::T) -> Self::T {
        self.delay.push_pop(value);
        self.interpolate(self.pos)
    }
}

pub struct FeedbackDelay<T> {
    pub time_s: T,
    pub feedback: T,
    delay: DelayLine<T>,
}

impl<T: Sample> FeedbackDelay<T> {
    pub fn new(max_length: usize, time_s: T, feedback: T) -> Self {
        Self {
            time_s,
            feedback,
            delay: DelayLine::new(max_length),
        }
    }

    pub fn len(&self) -> usize {
        self.delay.len()
    }
}

impl<T: Sample + Float> FeedbackDelay<T> {
    pub fn interpolate(&self, srate: impl Into<T>, time_s: impl Into<T>) -> T {
        let len = self.delay.len();
        let off_s = time_s.into() * srate.into();
        let offset = off_s.floor().to_usize().unwrap() % len;
        let fract = off_s.fract();

        let i = self.delay.len() - offset;
        let x0 = self.delay.get(i % len).copied().unwrap_or(T::ZERO);
        let x1 = self.delay.get((i - 1) % len).copied().unwrap_or(T::ZERO);
        x0 + fract * (x1 - x0)
    }
}

impl<T: Sample + Float + From<f32>> super::SingleChannelProcess for FeedbackDelay<T> {
    type T = T;

    fn process_single_channel(&mut self, ctx: &super::AudioContext, value: Self::T) -> Self::T {
        let v = value + self.feedback * self.interpolate(ctx.sample_rate, self.time_s);
        self.delay.push_pop(v);
        return v;
    }
}
