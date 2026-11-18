use std::f64::consts::TAU;
use std::ops::Neg;

pub struct Coefficients {
    b0: f64,
    b1: f64,
    b2: f64,
    a0: f64,
    a1: f64,
    a2: f64,
}

fn calculate_coefficients(sample_rate: f64, hz: f64, q: f64) -> Coefficients {
    let omega = hz.abs() * (TAU / sample_rate);
    let alpha = (omega.sin() * 0.5) / q.abs();
    let cos_omega = omega.cos();

    let a0 = alpha + 1.0;

    let a_thing = 1.0 / a0;

    let a1 = cos_omega.neg() * 2.0 * a_thing;
    let a2 = 1.0 - alpha * a_thing;

    let b0 = alpha * a_thing;
    let b1 = 0.0;
    let b2 = alpha.neg() * a_thing;

    Coefficients {
        a0,
        a1,
        a2,
        b0,
        b1,
        b2,
    }
}

pub struct Biquad {
    coefficients: Coefficients,
    hz: f64,
    q: f64,
    sample_rate: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl Biquad {
    pub fn new(sample_rate: f64, hz: f64, q: f64) -> Biquad {
        Biquad {
            coefficients: calculate_coefficients(sample_rate, hz, q),
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            sample_rate,
            hz,
            q,
        }
    }

    pub fn set_q(&mut self, q: f64) {
        self.q = q;
        self.coefficients = calculate_coefficients(self.sample_rate, self.hz, self.q);
    }

    pub fn set_hz(&mut self, hz: f64) {
        self.hz = hz;
        self.coefficients = calculate_coefficients(self.sample_rate, self.hz, self.q);
    }

    pub fn process(&mut self, x0: f64) -> f64 {
        let y0 = self.coefficients.b0 * x0
            + self.coefficients.b1 * self.x1
            + self.coefficients.b2 * self.x2
            - self.coefficients.a1 * self.y1
            - self.coefficients.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;

        y0
    }
}
