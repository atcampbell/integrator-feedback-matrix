use crate::allpass::Allpass;
use crate::biquad::Biquad;
use crate::dc_block::DcBlock;
use crate::integrator::Integrator;
use crate::matrix::Matrix;
use crate::shared_params::{PointWriter, SharedParam};
use web_sys::console;
use web_sys::js_sys::Math::random;
// TODO not using rusts random here update to use rand::random

const INTEGRATOR_COEF: f64 = 0.98;

fn to_audio_f32(sample: f64) -> f32 {
    // sample as f32
    sample.clamp(-1.0, 1.0) as f32
}

// TODO make a channel struct which has all of the required parts

pub struct Synth {
    dc_blocks: Vec<DcBlock>,
    integrators: Vec<Integrator>,
    filters: Vec<Biquad>,
    allpass: Vec<Allpass>,
    matrix: Matrix,
}

impl Synth {
    pub fn new(sample_rate: f64, channels: u32) -> Self {
        let mut dc_blocks: Vec<DcBlock> = Vec::new();
        let mut integrators: Vec<Integrator> = Vec::new();
        let mut filters: Vec<Biquad> = Vec::new();
        let mut allpass: Vec<Allpass> = Vec::new();

        for _i in 0..channels {
            dc_blocks.push(DcBlock::new());
            integrators.push(Integrator::new(INTEGRATOR_COEF));
            filters.push(Biquad::new(
                sample_rate,
                random() * 4000.0 + 10.0,
                random() * 5.0 + 0.001,
                // rng().random_range(10.0..=4000.0),
                // rng().random_range(0.001..=5.0), // random() + 0.01 * 5.0,
            ));
            allpass.push(Allpass::new(random() * 2.0 - 1.0))
        }

        Synth {
            matrix: Matrix::new(channels),
            integrators,
            dc_blocks,
            filters,
            allpass,
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

        for i in 0..self.integrators.iter().len() {
            // let mut out = self.dc_blocks[i].process(matrix_outs[i]).clamp(-1.0, 1.0);
            let mut out = matrix_outs[i];
            // let mut out = self.dc_blocks[i].process(matrix_outs[i]);
            // out = self.filters[i].process(out);
            out = self.allpass[i].process(out);
            out = self.dc_blocks[i].process(out);
            out = out.clamp(-1.0, 1.0);
            self.integrators[0].process(out);
            output += out;
        }

        // for ((integrator, dc_block), &m) in self
        //     .integrators
        //     .iter_mut()
        //     .zip(self.dc_blocks.iter_mut())
        //     .zip(matrix_outs.iter())
        // {
        //     let out = dc_block.process(m).clamp(-1.0, 1.0);
        //     integrator.process(out);
        //     output += out;
        // }

        let main_out = output / 3.0_f64.sqrt() * 0.3;

        to_audio_f32(main_out)
    }
}
