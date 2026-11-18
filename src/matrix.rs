use web_sys::js_sys::Math::random;

#[derive(Clone)]
pub struct Matrix {
    channels: u32,
    matrix: Vec<Vec<f64>>,
    outputs: Vec<f64>,
}

impl Matrix {
    pub fn new(channels: u32) -> Matrix {
        let mut matrix: Vec<Vec<f64>> = Vec::new();
        let mut outputs: Vec<f64> = Vec::new();

        for _i in 0..channels {
            let mut row: Vec<f64> = Vec::new();
            for _j in 0..channels {
                row.push(random() * 2000.0 - 1000.0);
            }
            matrix.push(row);
            outputs.push(0.0);
        }

        Matrix {
            channels,
            matrix,
            outputs,
        }
    }

    pub fn process_matrix(&mut self, input: Vec<f64>) {
        for i in 0..self.channels {
            let mut out = 0.0;
            for j in 0..self.channels {
                out += input[j as usize] * self.matrix[i as usize][j as usize];
            }
            self.outputs[i as usize] = out;
        }
    }

    pub fn get_outs(&self) -> &Vec<f64> {
        &self.outputs
    }
}
