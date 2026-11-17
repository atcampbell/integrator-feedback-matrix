pub struct DcBlock {
    x1: f64,
    y1: f64,
    coef: f64,
}

impl DcBlock {
    pub fn new() -> DcBlock {
        DcBlock {
            x1: 0.0,
            y1: 0.0,
            coef: 0.995,
        }
    }

    pub fn process(&mut self, x: f64) -> f64 {
        self.y1 = x - self.x1 + self.coef * self.y1;
        self.x1 = x;
        self.y1
    }
}
