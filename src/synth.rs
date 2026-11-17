use web_sys::console;

use crate::dc_block::DcBlock;
use crate::integrator::Integrator;
use crate::matrix::Matrix;
use crate::shared_params::{PointWriter, SharedParam};

const INTEGRATOR_COEFF: f64 = 0.995;

fn to_audio_f32(sample: f64) -> f32 {
    // sample as f32
    sample.clamp(-1.0, 1.0) as f32
}

pub struct Synth {
    channels: i32,
    dc_blocks: Vec<DcBlock>,
    integrators: Vec<Integrator>,
    dc_block1: DcBlock,
    dc_block2: DcBlock,
    dc_block3: DcBlock,
    integrator1: Integrator,
    integrator2: Integrator,
    integrator3: Integrator,
    matrix: Matrix,
    // count: i32,
}

impl Synth {
    pub fn new(sample_rate: f64) -> Self {
        let channels = 3;

        let mut count = 0;

        let mut dc_blocks: Vec<DcBlock> = Vec::new();
        let mut integrators: Vec<Integrator> = Vec::new();

        while count < channels {
            dc_blocks.push(DcBlock::new());
            integrators.push(Integrator::new(INTEGRATOR_COEFF));
            count += 1;
        }

        Synth {
            dc_block1: DcBlock::new(),
            dc_block2: DcBlock::new(),
            dc_block3: DcBlock::new(),
            integrator1: Integrator::new(INTEGRATOR_COEFF),
            integrator2: Integrator::new(INTEGRATOR_COEFF),
            integrator3: Integrator::new(INTEGRATOR_COEFF),
            matrix: Matrix::new(),
            // count: 0,
            channels,
            integrators,
            dc_blocks,
        }
    }

    pub fn process(&mut self, writer: &mut PointWriter) -> f32 {
        let mut int_outs: Vec<f64> = Vec::new();
        let mut count = 0;
        while count < self.channels {
            int_outs.push(self.integrators[count as usize].clone().get_value());
            count += 1;
        }

        self.matrix.process(int_outs[0], int_outs[1], int_outs[2]);

        let mut count = 0;
        let mut output = 0.0;
        let matrix_outs = self.matrix.clone().get_outs();

        while count < self.channels {
            let mut out = matrix_outs[count as usize];
            // TODO update with options from vector get
            // let mut out = matrix_outs.get(count as usize);
            out = self.dc_blocks[count as usize].process(out);
            out = out.clamp(-1.0, 1.0);

            self.integrators[count as usize].process(out);

            output += out;
            count += 1;
        }

        let main_out = output / 3.0_f64.sqrt() * 0.3;

        // if self.count < 20 {
        //     console::log_1(
        //         &format!(
        //             "[{}] matrix out1: {:.4}, out2: {:.4}, out3: {:.4}, OUT: {:.4}",
        //             self.count,
        //             self.matrix.clone().get_out1(),
        //             self.matrix.clone().get_out2(),
        //             self.matrix.clone().get_out3(),
        //             to_audio_f32(out),
        //         )
        //         .into(),
        //     );
        //     self.count += 1;
        // }

        to_audio_f32(main_out)
    }
}
