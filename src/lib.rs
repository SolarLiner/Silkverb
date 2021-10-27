#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod components;
mod early_refl;
mod rev_tail;

use crate::components::AudioContext;
use baseplug::{Plugin, ProcessContext};
use components::{chorus::Hz, spread::Spread};
use early_refl::EarlyReflections;
use rev_tail::ReverbTail;
use serde::{Deserialize, Serialize};

baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct PluginModel {
        #[model (min = 0, max = 1.0 ,gradient="Power(0.15)")]
        #[parameter(name = "Room size", )]
        size: f32,

        #[model (min = 0.0, max = 1.0, gradient="Exponential")]
        #[parameter(name = "Tail decay")]
        feedback: f32,

        #[model (min = -90.0, max = 6.0, gradient="Power(0.15)")]
        #[parameter(name = "Dry", unit="Decibels")]
        dry_vol: f32,

        #[model (min = -90.0, max = 6.0, gradient="Power(0.15)")]
        #[parameter(name = "Early reflections", unit="Decibels")]
        er_vol: f32,

        #[model (min = -90.0, max = 6.0, gradient="Power(0.15)")]
        #[parameter (name = "Wet", unit="Decibels")]
        wet_vol: f32,

        #[model (min = 0.0, max = 1.0)]
        #[parameter (name = "Modulation Amount")]
        mod_amt: f32,

        #[model (min = 0.0, max = 1.0)]
        #[parameter (name = "Modulation Freq.")]
        mod_freq: f32,
    }
}

impl Default for PluginModel {
    fn default() -> Self {
        Self {
            size: 0.4,
            feedback: 0.6,
            dry_vol: 1.0,
            er_vol: 0.4,
            wet_vol: 0.5,
            mod_amt: 0.1,
            mod_freq: 0.1,
        }
    }
}

struct FdnPlugin {
    audio_context: AudioContext,
    early_refl: EarlyReflections<8>,
    rev_tail: ReverbTail<8>,
    fanout: Spread<f32, 2, 8>,
    fanin: Spread<f32, 8, 2>,
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
        let mut early_refl = EarlyReflections::new(sample_rate);
        let mut rev_tail = ReverbTail::new(sample_rate);
        early_refl.set_delay_fract(model.size);
        rev_tail.update_size(model.size);
        rev_tail.update_feedback(model.feedback);
        rev_tail.update_chorus(|c| {
            c.set_amplitude(0.1);
            c.set_frequency(Hz::from_frequency(model.mod_freq));
        });
        rev_tail.update_chorus_drywet(model.mod_amt);

        Self {
            audio_context,
            early_refl,
            rev_tail,
            fanout: Spread::default(),
            fanin: Spread::default(),
        }
    }

    #[inline]
    fn process<'proc>(&mut self, model: &PluginModelProcess, ctx: &'proc mut ProcessContext<Self>) {
        use components::Process;

        for i in 0..ctx.nframes {
            let inputs = [ctx.inputs[0].buffers[0][i], ctx.inputs[0].buffers[1][i]];
            let mut rev_input = [0.0; 8];
            let mut er_output_in = [0.0; 8];
            let mut rev_output_in = [0.0; 8];
            let mut er_out = [0.0; 2];
            let mut rev_out = [0.0; 2];

            self.early_refl.set_delay_fract(model.size[i]);
            self.rev_tail.update_size(model.size[i]);
            self.rev_tail.update_feedback(model.feedback[i]);
            self.rev_tail.update_chorus({
                let f = Hz::from_frequency(model.mod_freq[i]);
                move |c| {
                    c.set_frequency(f);
                }
            });
            self.rev_tail.update_chorus_drywet(model.mod_amt[i]);

            self.fanout
                .process(&self.audio_context, &inputs, &mut rev_input);
            self.early_refl
                .process(&self.audio_context, &rev_input, &mut er_output_in);
            self.rev_tail
                .process(&self.audio_context, &er_output_in, &mut rev_output_in);
            self.fanin
                .process(&self.audio_context, &er_output_in, &mut er_out);
            self.fanin
                .process(&self.audio_context, &rev_output_in, &mut rev_out);

            ctx.outputs[0].buffers[0][i] = rev_out[0] * model.wet_vol[i]
                + er_out[0] * model.er_vol[i]
                + inputs[0] * model.dry_vol[i];
            ctx.outputs[0].buffers[1][i] = rev_out[1] * model.wet_vol[i]
                + er_out[1] * model.er_vol[i]
                + inputs[1] * model.dry_vol[i];
            self.audio_context.sample_count += 1;
        }
    }
}

#[cfg(not(test))]
baseplug::vst2!(FdnPlugin, b"S1lK");
