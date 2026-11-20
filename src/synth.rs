use crate::allpass::Allpass;
use crate::biquad::Biquad;
use crate::dc_block::DcBlock;
use crate::integrator::Integrator;
use crate::matrix::Matrix;
use crate::shared_params::{PointWriter, SharedParam};
use web_sys::console;
// TODO not using rusts random here update to use rand::random
use web_sys::js_sys::Math::random;

const INTEGRATOR_COEF: f64 = 0.98;

fn to_audio_f32(sample: f64) -> f32 {
    // sample as f32
    sample.clamp(-1.0, 1.0) as f32
}

const ALLPASS_CHANNELS: u32 = 4;

pub struct Channel {
    dc_block: DcBlock,
    integrator: Integrator,
    allpass: Vec<Allpass>,
    filters: Vec<Biquad>,
}

impl Channel {
    pub fn new(sample_rate: f64) -> Channel {
        let mut allpass: Vec<Allpass> = Vec::new();
        let allpass_phase = random() * 2.0 - 1.0;
        for _i in 0..ALLPASS_CHANNELS {
            allpass.push(Allpass::new(allpass_phase))
        }

        let mut filters: Vec<Biquad> = Vec::new();
        for _i in 0..ALLPASS_CHANNELS {
            filters.push(Biquad::new(
                sample_rate,
                random() * 4000.0 + 10.0,
                random() * 5.0 + 0.001,
                // rng().random_range(10.0..=4000.0),
                // rng().random_range(0.001..=5.0), // random() + 0.01 * 5.0,))
            ))
        }

        Channel {
            dc_block: DcBlock::new(),
            integrator: Integrator::new(INTEGRATOR_COEF),
            allpass,
            filters, // filters.push(Biquad::new(
                     // sample_rate,
                     // random() * 4000.0 + 10.0,
                     // random() * 5.0 + 0.001,
                     // // rng().random_range(10.0..=4000.0),
                     // // rng().random_range(0.001..=5.0), // random() + 0.01 * 5.0,
                     // ));
                     // allpass.push(Allpass::new(random() * 2.0 - 1.0))
        }
    }

    pub fn process(&mut self, input: f64) -> f64 {
        let mut out = input;

        for filter in self.allpass.iter_mut() {
            out += filter.process(out);
        }
        out = self.dc_block.process(out);
        out = out.clamp(-1.0, 1.0);
        self.integrator.process(out);
        out
    }
}

pub struct Synth {
    matrix: Matrix,
    channels: Vec<Channel>,
}

impl Synth {
    pub fn new(sample_rate: f64, num_channels: u32) -> Self {
        let mut channels: Vec<Channel> = Vec::new();

        for _i in 0..num_channels {
            channels.push(Channel::new(sample_rate));
        }

        Synth {
            matrix: Matrix::new(num_channels),
            channels,
        }
    }

    pub fn process(&mut self, _writer: &mut PointWriter) -> f32 {
        let mut int_outs: Vec<f64> = Vec::new();

        for channel in self.channels.iter() {
            int_outs.push(channel.integrator.get_value());
        }

        self.matrix.process_matrix(int_outs);
        let mut output = 0.0;
        let matrix_outs = self.matrix.get_outs();

        for (i, channel) in self.channels.iter_mut().enumerate() {
            let out = channel.process(matrix_outs[i]);
            channel.integrator.process(out);
            output += out;
        }

        let main_out = output / 3.0_f64.sqrt() * 0.3;

        to_audio_f32(main_out)
    }
}
