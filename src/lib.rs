#![allow(incomplete_features)]
#![feature(const_generics)]
#![feature(generic_associated_types)]

mod components;

use crate::components::AudioContext;
use baseplug::{Plugin, ProcessContext};
use components::{allpass::Allpass, stereoize::Stereoize};
use serde::{Deserialize, Serialize};

baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct PluginModel {
        #[model (min = 0.01, max = 1.0)]
        #[parameter(name = "delay")]
        delay: f32,

        #[model (min = 0.0, max = 1.0)]
        #[parameter(name = "feedback")]
        feedback: f32,
    }
}

impl Default for PluginModel {
    fn default() -> Self {
        Self {
            delay: 0.4,
            feedback: 0.4,
        }
    }
}

struct FdnPlugin {
    audio_context: AudioContext,
    delay: Stereoize<seq!(f32, Allpass<8>; Allpass<8>; Allpass<8>; Allpass<8>), 8, 8>,
}

impl Plugin for FdnPlugin {
    const NAME: &'static str = "Silkverb";
    const PRODUCT: &'static str = "Silkverb";
    const VENDOR: &'static str = "SolarLiner";

    const INPUT_CHANNELS: usize = 2;
    const OUTPUT_CHANNELS: usize = 2;

    type Model = PluginModel;

    #[inline]
    fn new(sample_rate: f32, model: &PluginModel) -> Self {
        let audio_context = AudioContext {
            sample_rate: sample_rate as _,
            sample_count: 0,
        };
        let delay = Stereoize::new(
            seqdef!(Allpass::new((sample_rate * 2.0) as _); Allpass::new((sample_rate * 2.0) as _); Allpass::new((sample_rate * 2.0) as _); Allpass::new((sample_rate * 2.0) as _)),
        );
        Self {
            audio_context,
            delay,
        }
    }

    #[inline]
    fn process<'proc>(&mut self, model: &PluginModelProcess, ctx: &'proc mut ProcessContext<Self>) {
        use components::Process;

        for i in 0..ctx.nframes {
            let inputs = [ctx.inputs[0].buffers[0][i], ctx.inputs[0].buffers[1][i]];
            let mut outputs = [0.0, 0.0];
            {
                let p = self.delay.process_mut();
                let f = model.delay[i];
                p.pa.update(f);
                p.pb.pa.update(f);
                p.pb.pb.pa.update(f);
                p.pb.pb.pb.update(f);
            }
            self.delay
                .process(&self.audio_context, &inputs, &mut outputs);

            ctx.outputs[0].buffers[0][i] = outputs[0];
            ctx.outputs[0].buffers[1][i] = outputs[1];
            self.audio_context.sample_count += 1;
        }
    }
}

#[cfg(not(test))]
baseplug::vst2!(FdnPlugin, b"S1lK");
