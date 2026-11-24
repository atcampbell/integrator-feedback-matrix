use web_sys::js_sys::Math::random;

pub struct Matrix {
    channels: u32,
    matrix: Vec<Vec<f64>>,
    outputs: Vec<f64>,
}

impl Matrix {
    pub fn new(channels: u32) -> Matrix {
        let size = channels as usize;
        let mut matrix = Matrix {
            channels,
            matrix: vec![vec![0.0; size]; size],
            outputs: vec![0.0; size],
        };
        matrix.randomise();
        matrix
    }

    /// Give every weight a new random value between -1000 and 1000.
    pub fn randomise(&mut self) {
        for row in self.matrix.iter_mut() {
            for weight in row.iter_mut() {
                *weight = random() * 2000.0 - 1000.0;
            }
        }
    }

    /// `cross_feedback` scales only the cross-channel weights (where `i != j`), so it
    /// sets how strongly channels drive each other. Each channel's feedback
    /// into itself (the diagonal, `i == j`) is left unscaled.
    pub fn process_matrix(&mut self, input: Vec<f64>, cross_feedback: f64) {
        for i in 0..self.channels {
            let mut out = 0.0;
            for j in 0..self.channels {
                let mut weight = self.matrix[i as usize][j as usize];
                if i != j {
                    weight *= cross_feedback;
                }
                out += input[j as usize] * weight;
            }
            self.outputs[i as usize] = out;
        }
    }

    pub fn get_outs(&self) -> &Vec<f64> {
        &self.outputs
    }
}
