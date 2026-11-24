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
/// Smoothing for coupling and drive, so slider moves don't click (~20ms).
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
}

impl Channel {
    pub fn new(sample_rate: f64) -> Channel {
        let pan_angle = random() * FRAC_PI_2;

        Channel {
            dc_block: DcBlock::new(),
            integrator: Integrator::new(INTEGRATOR_COEF),
            allpass: Phaser::new(sample_rate, ALLPASS_CHANNELS, -0.0),
            // filters,
            pan_left: pan_angle.cos(),
            pan_right: pan_angle.sin(),
            level: 1.0,
            level_smoothing: Integrator::new(LEVEL_SLEW),
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
    /// Multiplier on the matrix's cross-channel weights. Set from the UI,
    /// smoothed per sample.
    coupling: f64,
    coupling_smoothing: Integrator,
    /// Gain into each channel's tanh. Set from the UI, smoothed per sample.
    drive: f64,
    drive_smoothing: Integrator,
}

impl Synth {
    pub fn new(sample_rate: f64, num_channels: u32) -> Self {
        let mut channels: Vec<Channel> = Vec::new();

        for _i in 0..num_channels {
            channels.push(Channel::new(sample_rate));
        }

        // The smoothers start at 1.0, which matches these defaults.
        Synth {
            matrix: Matrix::new(num_channels),
            channels,
            coupling: 1.0,
            coupling_smoothing: Integrator::new(PARAM_SLEW),
            drive: 1.0,
            drive_smoothing: Integrator::new(PARAM_SLEW),
        }
    }

    /// Read the latest values from the UI. Called once per audio buffer.
    /// Each name matches a `data-param` attribute in index.html. For example:
    ///
    ///     if let Some(coupling) = params.get("coupling") {
    ///         self.coupling = coupling;
    ///     }
    pub fn apply_params(&mut self, params: &Params) {
        if let Some(amount) = params.get("integrator") {
            self.set_integrator_coefficient(amount);
        }
        if let Some(amount) = params.get("phaser_feedback") {
            self.set_phaser_feedback(amount);
        }
        if let Some(amount) = params.get("coupling") {
            self.set_coupling(amount);
        }
        if let Some(amount) = params.get("colour") {
            self.set_colour(amount);
        }
        if let Some(amount) = params.get("drive") {
            self.set_drive(amount);
        }
    }

    /// `amount` is 0–1 from the UI. It maps to a multiplier on the matrix's
    /// cross-channel weights of 0.001 → 10 on a log scale (10^x, with x from
    /// -3 to 1); 0.75 gives 1 (unchanged). At the bottom the channels are
    /// almost independent; at the top they're tightly interlocked.
    fn set_coupling(&mut self, amount: f64) {
        self.coupling = 10f64.powf(-3.0 + 4.0 * amount.clamp(0.0, 1.0));
    }

    /// `amount` is 0–1 from the UI. It maps to the gain into each channel's
    /// tanh of 0.1 → 10 on a log scale (10^x, with x from -1 to 1);
    /// 0.5 gives 1 (unchanged).
    fn set_drive(&mut self, amount: f64) {
        self.drive = 10f64.powf(-1.0 + 2.0 * amount.clamp(0.0, 1.0));
    }

    /// `amount` is 0–1 from the UI. It maps linearly to an offset of -1 → 1
    /// on every phaser's allpass coefficient; 0.5 leaves them unchanged.
    fn set_colour(&mut self, amount: f64) {
        let offset = amount.clamp(0.0, 1.0) * 2.0 - 1.0;
        for channel in self.channels.iter_mut() {
            channel.allpass.set_colour(offset);
        }
    }

    /// `amount` is 0–1 from the UI. It maps linearly to a feedback of -1 → 1,
    /// so the middle of the slider (0.5) is no feedback.
    fn set_phaser_feedback(&mut self, amount: f64) {
        let feedback = amount.clamp(0.0, 1.0) * 2.0 - 1.0;
        for channel in self.channels.iter_mut() {
            channel.allpass.set_feedback(feedback);
        }
    }

    /// `amount` is 0–1 from the UI. It maps to a coefficient of 0.99 → 0.9999
    /// on a log scale (1 - 10^-x, with x from 2 to 4), so the slider spends
    /// equal travel on each extra "9" rather than bunching them at the end.
    fn set_integrator_coefficient(&mut self, amount: f64) {
        let x = 2.0 + 2.2 * amount.clamp(0.0, 1.0);
        let coefficient = 1.0 - 10f64.powf(-x);
        for channel in self.channels.iter_mut() {
            channel.integrator.set_coefficient(coefficient);
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

        let coupling = self.coupling_smoothing.process(self.coupling);
        self.matrix.process_matrix(integrator_outs, coupling);

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
