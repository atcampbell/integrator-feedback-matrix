use crate::dc_block::DcBlock;
use crate::integrator::Integrator;
use crate::matrix::Matrix;
use crate::phaser::Phaser;
use crate::shared_params::{Params, PointWriter};
use std::f64::consts::FRAC_PI_2;
// TODO not using rusts random here update to use rand::random
use web_sys::js_sys::Math::random;

const INTEGRATOR_COEF: f64 = 0.99;
const ALLPASS_CHANNELS: u32 = 16;
const LEVEL_SLEW: f64 = 0.99995;
const PARAM_SLEW: f64 = 0.999;

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
    level_offset: f64,
}

impl Channel {
    /// `pan` is the channel's stereo position: 0 is hard left, 1 is hard right.
    pub fn new(sample_rate: f64, pan: f64) -> Channel {
        // Equal-power panning: cos/sin keep the level constant across the field.
        let pan_angle = pan * FRAC_PI_2;

        Channel {
            dc_block: DcBlock::new(),
            integrator: Integrator::new(INTEGRATOR_COEF),
            allpass: Phaser::new(sample_rate, ALLPASS_CHANNELS, -0.0),
            // filters,
            pan_left: pan_angle.cos(),
            pan_right: pan_angle.sin(),
            level: 1.0,
            level_smoothing: Integrator::new(LEVEL_SLEW),
            level_offset: random(),
        }
    }

    pub fn process(&mut self, input: f64, drive: f64) -> f64 {
        let mut out = self.allpass.process(input);
        out = self.dc_block.process(out);
        out = (out * drive).tanh();
        out *= self.level_smoothing.process(self.level);
        self.integrator.process(out);
        out
    }
}

pub struct Synth {
    matrix: Matrix,
    channels: Vec<Channel>,
    cross_feedback: f64,
    cross_feedback_smoothing: Integrator,
    drive: f64,
    drive_smoothing: Integrator,
}

impl Synth {
    pub fn new(sample_rate: f64, num_channels: u32) -> Self {
        let mut channels: Vec<Channel> = Vec::new();

        for i in 0..num_channels {
            let pan = if num_channels > 1 {
                i as f64 / (num_channels - 1) as f64
            } else {
                0.5
            };
            channels.push(Channel::new(sample_rate, pan));
        }

        // The smoothers start at 1.0, which matches these defaults.
        Synth {
            matrix: Matrix::new(num_channels),
            channels,
            cross_feedback: 1.0,
            cross_feedback_smoothing: Integrator::new(PARAM_SLEW),
            drive: 1.0,
            drive_smoothing: Integrator::new(PARAM_SLEW),
        }
    }

    pub fn apply_params(&mut self, params: &Params) {
        if let Some(amount) = params.get("integrator") {
            self.set_integrator_coefficient(amount);
        }
        if let Some(amount) = params.get("phaser_feedback") {
            self.set_phaser_feedback(amount);
        }
        if let Some(amount) = params.get("cross_feedback") {
            self.set_cross_feedback(amount);
        }
        if let Some(amount) = params.get("phaser_coefficient_offset") {
            self.set_phaser_coefficient_offset(amount);
        }
        if let Some(amount) = params.get("drive") {
            self.set_drive(amount);
        }
        if let Some(amount) = params.get("channel_level_offset") {
            self.set_channel_level_offset(amount);
        }
    }

    fn set_channel_level_offset(&mut self, amount: f64) {
        let offset = amount.clamp(0.0, 1.0);
        for channel in self.channels.iter_mut() {
            channel.level = 1.0 - offset * channel.level_offset;
        }
    }

    fn set_cross_feedback(&mut self, amount: f64) {
        self.cross_feedback = 10f64.powf(-3.0 + 4.0 * amount.clamp(0.0, 1.0));
    }

    fn set_drive(&mut self, amount: f64) {
        self.drive = 10f64.powf(-1.0 + 2.0 * amount.clamp(0.0, 1.0));
    }

    fn set_phaser_coefficient_offset(&mut self, amount: f64) {
        let offset = amount.clamp(0.0, 1.0) * 2.0 - 1.0;
        for channel in self.channels.iter_mut() {
            channel.allpass.set_coefficient_offset(offset);
        }
    }

    fn set_phaser_feedback(&mut self, amount: f64) {
        let feedback = amount.clamp(0.0, 1.0) * 2.0 - 1.0;
        for channel in self.channels.iter_mut() {
            channel.allpass.set_feedback(feedback);
        }
    }

    fn set_integrator_coefficient(&mut self, amount: f64) {
        let x = 1.0 + 2.2 * amount.clamp(0.0, 1.0);
        let coefficient = 1.0 - 10f64.powf(-x);
        for channel in self.channels.iter_mut() {
            channel.integrator.set_coefficient(coefficient);
        }
    }

    pub fn randomise_matrix(&mut self) {
        self.matrix.randomise();
        for channel in self.channels.iter_mut() {
            channel.level_offset = random();
        }
    }

    pub fn process(&mut self, writer: &mut PointWriter) -> (f32, f32) {
        let integrator_outs = self
            .channels
            .iter()
            .map(|c| c.integrator.get_value())
            .collect();

        let cross_feedback = self.cross_feedback_smoothing.process(self.cross_feedback);
        self.matrix.process_matrix(integrator_outs, cross_feedback);

        let matrix_outs = self.matrix.get_outs();
        let drive = self.drive_smoothing.process(self.drive);

        let mut left = 0.0;
        let mut right = 0.0;

        for (channel, &m) in self.channels.iter_mut().zip(matrix_outs) {
            let out = channel.process(m, drive);
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
