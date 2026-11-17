pub struct Integrator {
    input: f64,
    coefficient: f64,
    value: f64,
}

impl Integrator {
    pub fn new(coefficient: f64) -> Integrator {
        Integrator {
            coefficient,
            input: 0.0,
            value: 0.0,
        }
    }

    pub fn set_coefficient(&mut self, coefficient: f64) {
        self.coefficient = coefficient
    }

    pub fn process(&mut self, input: f64) -> f64 {
        self.value =
            input.clamp(0.0, 1.0) * (1.0 - self.coefficient) + (self.coefficient * self.value);
        self.value
    }
}
