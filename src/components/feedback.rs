use std::ops::AddAssign;

use audio::Sample;

use super::Process;

pub(crate) struct Feedback<P: Process, Q: Process<T = P::T>, const N: usize> {
    forward: P,
    backward: Q,
    fb_buffer: [P::T; N],
}

impl<P: Process, Q: Process<T = P::T>, const N: usize> Feedback<P, Q, N>
where
    P::T: Sample,
{
    pub fn new(forward: P, backward: Q) -> Self {
        debug_assert_eq!(P::NOUT, Q::NIN);
        debug_assert_eq!(P::NIN, Q::NOUT);
        debug_assert_eq!(N, P::NIN);

        Self {
            forward,
            backward,
            fb_buffer: [P::T::ZERO; N],
        }
    }

    pub fn forward_mut(&mut self) -> &mut P {
        &mut self.forward
    }

    pub fn backward_mut(&mut self) -> &mut Q {
        &mut self.backward
    }
}

impl<P: Process, Q: Process<T = P::T>, const N: usize> Process for Feedback<P, Q, N>
where
    P::T: Sample + AddAssign,
{
    type T = P::T;
    const NIN: usize = P::NIN;
    const NOUT: usize = P::NOUT;

    fn process(
        &mut self,
        ctx: &super::AudioContext,
        input_frame: &[Self::T],
        output_frame: &mut [Self::T],
    ) {
        self.fb_buffer
            .iter_mut()
            .enumerate()
            .for_each(|(i, o)| *o += input_frame[i]);

        self.forward.process(ctx, &self.fb_buffer, output_frame);
        self.backward
            .process(ctx, output_frame, &mut self.fb_buffer);
    }
}
