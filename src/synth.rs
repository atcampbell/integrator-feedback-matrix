use crate::dc_block::DcBlock;
use crate::integrator::Integrator;
use crate::matrix::Matrix;
use crate::phaser::Phaser;
use crate::shared_params::PointWriter;
use std::f64::consts::FRAC_PI_2;
// TODO not using rusts random here update to use rand::random
use web_sys::js_sys::Math::random;

const INTEGRATOR_COEF: f64 = 0.99;
const ALLPASS_CHANNELS: u32 = 16;
const LEVEL_SLEW: f64 = 0.99995;

fn to_audio_f32(sample: f64) -> f32 {
    // sample as f32
    sample.clamp(-1.0, 1.0) as f32
}

pub struct Channel {
    dc_block: DcBlock,
    integrator: Integrator,
    allpass: Phaser,
    pan_left: f64,
    pan_right: f64,
    level: f64,
    level_smoothing: Integrator,
}

impl Channel {
    pub fn new(sample_rate: f64) -> Channel {
        let pan_angle = random() * FRAC_PI_2;

        Channel {
            dc_block: DcBlock::new(),
            integrator: Integrator::new(INTEGRATOR_COEF),
            allpass: Phaser::new(sample_rate, ALLPASS_CHANNELS, -1.0),
            // filters,
            pan_left: pan_angle.cos(),
            pan_right: pan_angle.sin(),
            level: 1.0,
            level_smoothing: Integrator::new(LEVEL_SLEW),
        }
    }

    pub fn process(&mut self, input: f64) -> f64 {
        let mut out = input;
        out = self.allpass.process(input);
        out = self.dc_block.process(out);
        // out = (out * 0.01).tanh();
        out = out.tanh();
        out *= self.level_smoothing.process(self.level);
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

    pub fn randomise_levels(&mut self) {
        for channel in self.channels.iter_mut() {
            channel.level = random() * 0.5 + 0.5;
        }
    }

    pub fn process(&mut self, writer: &mut PointWriter) -> (f32, f32) {
        let integrator_outs = self
            .channels
            .iter()
            .map(|c| c.integrator.get_value())
            .collect();

        self.matrix.process_matrix(integrator_outs);

        let matrix_outs = self.matrix.get_outs();

        let mut left = 0.0;
        let mut right = 0.0;

        for (channel, &m) in self.channels.iter_mut().zip(matrix_outs) {
            let out = channel.process(m);
            left += out * channel.pan_left;
            right += out * channel.pan_right;
        }

        let gain = 0.3 / (self.channels.len() as f64).sqrt();
        let left_out = to_audio_f32(left * gain);
        let right_out = to_audio_f32(right * gain);

        // Send the output to the scope: x = left, y = right, z = mid.
        writer.push(left_out, right_out, (left_out + right_out) * 0.5);

        (left_out, right_out)
    }
}
