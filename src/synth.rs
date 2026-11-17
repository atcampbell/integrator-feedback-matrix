use web_sys::console;

use crate::dc_block::DcBlock;
use crate::integrator::Integrator;
use crate::matrix::Matrix;
use crate::shared_params::{PointWriter, SharedParam};

fn to_audio_f32(sample: f64) -> f32 {
    // sample as f32
    sample.clamp(-1.0, 1.0) as f32
}

pub struct Synth {
    dc_block1: DcBlock,
    dc_block2: DcBlock,
    dc_block3: DcBlock,
    integrator1: Integrator,
    integrator2: Integrator,
    integrator3: Integrator,
    matrix: Matrix,
    count: i32,
}

impl Synth {
    pub fn new(sample_rate: f64) -> Self {
        Synth {
            dc_block1: DcBlock::new(),
            dc_block2: DcBlock::new(),
            dc_block3: DcBlock::new(),
            integrator1: Integrator::new(0.995),
            integrator2: Integrator::new(0.995),
            integrator3: Integrator::new(0.995),
            matrix: Matrix::new(),
            count: 0,
        }
    }

    pub fn process(&mut self, writer: &mut PointWriter) -> f32 {
        // TODO get rid of these clones
        self.matrix.process(
            self.integrator1.clone().get_value(),
            self.integrator2.clone().get_value(),
            self.integrator3.clone().get_value(),
        );

        // TODO get rid of clone
        let out1 = self
            .dc_block1
            .process(self.matrix.clone().get_out1())
            .clamp(-1.0, 1.0);
        let out2 = self
            .dc_block2
            .process(self.matrix.clone().get_out2())
            .clamp(-1.0, 1.0);
        let out3 = self
            .dc_block3
            .process(self.matrix.clone().get_out3())
            .clamp(-1.0, 1.0);

        self.integrator1.process(out1);
        self.integrator2.process(out2);
        self.integrator3.process(out3);

        let out = (out1 + out2 + out3) / 3.0_f64.sqrt() * 0.3;

        if self.count < 20 {
            console::log_1(
                &format!(
                    "[{}] matrix out1: {:.4}, out2: {:.4}, out3: {:.4}, OUT: {:.4}",
                    self.count,
                    self.matrix.clone().get_out1(),
                    self.matrix.clone().get_out2(),
                    self.matrix.clone().get_out3(),
                    to_audio_f32(out),
                )
                .into(),
            );
            self.count += 1;
        }

        to_audio_f32(out)
    }
}
