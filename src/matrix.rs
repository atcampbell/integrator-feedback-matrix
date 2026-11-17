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
}

impl Matrix {
    pub fn new() -> Matrix {
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
        }
    }

    pub fn process(&mut self, ina: f64, inb: f64, inc: f64) {
        self.out1 = ina * self.a1 + inb * self.b1 + inc * self.c1;
        self.out2 = inb * self.a2 + inb * self.b2 + inc * self.c2;
        self.out3 = inc * self.a3 + inb * self.b3 + inc * self.c3;
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
}
