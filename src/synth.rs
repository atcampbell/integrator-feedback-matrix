use crate::dc_block::DcBlock;
use crate::lorenz::Lorenz;
use crate::shared_params::{PointWriter, SharedParam};

const LORENZ_SCALE: f64 = 1.0 / 25.0;

fn to_audio_f32(sample: f64) -> f32 {
    // sample as f32
    sample.clamp(-1.0, 1.0) as f32
}

pub struct Synth {
    lorenz: Lorenz,
    dt: SharedParam,
    dc_block: DcBlock,
}

impl Synth {
    pub fn new(sample_rate: f64, dt: SharedParam) -> Self {
        Synth {
            lorenz: Lorenz::new(),
            dt,
            dc_block: DcBlock::new(),
        }
    }

    pub fn process(&mut self, writer: &mut PointWriter) -> f32 {
        self.lorenz.set_dt(self.dt.get());
        self.lorenz.step();
        writer.push(
            self.lorenz.x() as f32,
            self.lorenz.y() as f32,
            self.lorenz.z() as f32,
        );

        let signal = self.lorenz.x();
        to_audio_f32(self.dc_block.process(signal * LORENZ_SCALE))
    }
}
