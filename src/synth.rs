use web_sys::console;

use crate::dc_block::DcBlock;
use crate::integrator::Integrator;
use crate::matrix::Matrix;
use crate::shared_params::{PointWriter, SharedParam};

const INTEGRATOR_COEF: f64 = 0.98;

fn to_audio_f32(sample: f64) -> f32 {
    // sample as f32
    sample.clamp(-1.0, 1.0) as f32
}

pub struct Synth {
    dc_blocks: Vec<DcBlock>,
    integrators: Vec<Integrator>,
    matrix: Matrix,
}

impl Synth {
    pub fn new(_sample_rate: f64, channels: u32) -> Self {
        let mut dc_blocks: Vec<DcBlock> = Vec::new();
        let mut integrators: Vec<Integrator> = Vec::new();

        for _i in 0..channels {
            dc_blocks.push(DcBlock::new());
            integrators.push(Integrator::new(INTEGRATOR_COEF));
        }

        Synth {
            matrix: Matrix::new(channels),
            integrators,
            dc_blocks,
        }
    }

    pub fn process(&mut self, _writer: &mut PointWriter) -> f32 {
        let mut int_outs: Vec<f64> = Vec::new();

        for integrator in self.integrators.iter() {
            int_outs.push(integrator.get_value())
        }

        self.matrix.process_matrix(int_outs);

        let mut output = 0.0;
        let matrix_outs = self.matrix.get_outs();

        for ((integrator, dc_block), &m) in self
            .integrators
            .iter_mut()
            .zip(self.dc_blocks.iter_mut())
            .zip(matrix_outs.iter())
        {
            let out = dc_block.process(m).clamp(-1.0, 1.0);
            integrator.process(out);
            output += out;
        }

        let main_out = output / 3.0_f64.sqrt() * 0.3;

        to_audio_f32(main_out)
    }
}
