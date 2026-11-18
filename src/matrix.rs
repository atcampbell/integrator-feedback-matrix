use web_sys::console;
use web_sys::js_sys::Math::random;

#[derive(Debug, Clone)]
pub struct Matrix {
    a1: f64,
    a2: f64,
    a3: f64,
    b1: f64,
    b2: f64,
    b3: f64,
    c1: f64,
    c2: f64,
    c3: f64,
    out1: f64,
    out2: f64,
    out3: f64,

    channels: u32,
    matrix: Vec<Vec<f64>>,
    outputs: Vec<f64>,
}

impl Matrix {
    pub fn new(channels: u32) -> Matrix {
        let mut matrix: Vec<Vec<f64>> = Vec::new();
        let mut outputs: Vec<f64> = Vec::new();

        // console::log_1(&format!("BEFORE").into());

        for i in 0..channels {
            let mut row: Vec<f64> = Vec::new();
            for x in 0..channels {
                row.push(random() * 2000.0 - 1000.0);
                // matrix[i as usize][x as usize] = random() * 2000.0 - 1000.0;
            }
            matrix.push(row);
            outputs.push(0.0);
            // outputs[i as usize] = 0.0;
        }

        // console::log_1(&format!("AFTER").into());

        Matrix {
            a1: random() * 2000.0 - 1000.0,
            a2: random() * 2000.0 - 1000.0,
            a3: random() * 2000.0 - 1000.0,
            b1: random() * 2000.0 - 1000.0,
            b2: random() * 2000.0 - 1000.0,
            b3: random() * 2000.0 - 1000.0,
            c1: random() * 2000.0 - 1000.0,
            c2: random() * 2000.0 - 1000.0,
            c3: random() * 2000.0 - 1000.0,
            out1: 0.0,
            out2: 0.0,
            out3: 0.0,

            channels,
            matrix,
            outputs,
        }
    }

    pub fn process(&mut self, ina: f64, inb: f64, inc: f64) {
        self.out1 = ina * self.a1 + inb * self.b1 + inc * self.c1;
        self.out2 = ina * self.a2 + inb * self.b2 + inc * self.c2;
        self.out3 = ina * self.a3 + inb * self.b3 + inc * self.c3;
    }

    // TODO something not quite right in this process - always giving same outputs...
    pub fn process_matrix(&mut self, input: Vec<f64>) {
        for i in 0..self.channels {
            let mut out = 0.0;
            for x in 0..self.channels {
                out += input[i as usize] * self.matrix[i as usize][x as usize];
                // matrix[i as usize][x as usize] = random() * 2000.0 - 1000.0;
            }
            self.outputs[i as usize] = out;
        }
    }

    pub fn get_out1(self) -> f64 {
        self.out1
    }
    pub fn get_out2(self) -> f64 {
        self.out2
    }
    pub fn get_out3(self) -> f64 {
        self.out3
    }

    pub fn get_outs(self) -> Vec<f64> {
        self.outputs
        // vec![self.out1, self.out2, self.out3]
    }
}
