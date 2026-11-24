use crate::dc_block::DcBlock;
use web_sys::js_sys::Math::random;

pub struct Allpass {
    prev: f64,
    g: f64,
}

impl Allpass {
    pub fn new(g: f64) -> Allpass {
        Allpass { prev: 0.0, g }
    }

    pub fn process(&mut self, input: f64) -> f64 {
        let v = input + self.g * self.prev;
        let out = -self.g * v + self.prev;
        self.prev = v;
        out
    }
}

fn crossfade(in1: f64, in2: f64, crossfade: f64) -> f64 {
    ((1.0 - crossfade) * in1) + (crossfade * in2)
}

pub struct Phaser {
    allpass: Vec<Allpass>,
    feedback: f64,
    sample_rate: f64,
    value: f64,
    dc_block: DcBlock,
}

impl Phaser {
    pub fn new(sample_rate: f64, num_channels: u32, feedback: f64) -> Phaser {
        let mut allpass: Vec<Allpass> = Vec::new();
        let allpass_phase = random() * 2.0 - 1.0;
        for _i in 0..num_channels {
            allpass.push(Allpass::new(allpass_phase))
        }

        Phaser {
            value: 0.0,
            dc_block: DcBlock::new(),
            feedback,
            sample_rate,
            allpass,
        }
    }

    pub fn process(&mut self, input: f64) -> f64 {
        let mut out = crossfade(input, self.value, self.feedback);

        for filter in self.allpass.iter_mut() {
            out = filter.process(out);
        }

        out += input;
        self.value = self.dc_block.process(out);
        out
    }
}
