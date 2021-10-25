use audio::Sample;

use super::{AudioContext, Process};

pub(crate) struct Sequence<T, P, Q> {
    pub pa: P,
    pub pb: Q,
    tmp_buf: Vec<T>,
}

impl<T: Sample, P: Process<T=T>, Q: Process<T=T>> Sequence<T, P, Q> {
    pub fn new(pa: P, pb: Q) -> Self {
        debug_assert_eq!(P::NOUT, Q::NIN);
        Self {
            pa,
            pb,
            tmp_buf: vec![T::ZERO; P::NOUT],
        }
    }
}

impl<T: Sample, P: Process<T = T>, Q: Process<T = T>> Process for Sequence<T, P, Q> {
    type T = T;
    const NIN: usize = P::NIN;
    const NOUT: usize = Q::NOUT;

    fn process(
        &mut self,
        ctx: &AudioContext,
        input_frame: &[<Self as Process>::T],
        output_frame: &mut [<Self as Process>::T],
    ) {
        debug_assert_eq!(P::NOUT, Q::NIN);
        self.pa.process(ctx, input_frame, &mut self.tmp_buf);
        self.pb.process(ctx, &self.tmp_buf, output_frame);
    }
}

#[macro_export]
macro_rules! seq {
    ($st:tt, $t:ty) => {
        $t
    };
    ($st:tt, $t:ty; $($ts:ty);*) => {
        $crate::components::seq::Sequence<$st, $t, seq!($st, $($ts);*)>
    }
}

#[macro_export]
macro_rules! seqdef {
    ($e:expr) => {
        $e
    };
    ($e:expr; $($es:expr);*) => {
        $crate::components::seq::Sequence::new($e, seqdef!($($es);*))
    }
}
