use std::ops::Neg;

pub struct Allpass {
    prev: f64,
    g: f64,
}

impl Allpass {
    // TODO g in random -1 and 1
    pub fn new(g: f64) -> Allpass {
        Allpass { prev: 0.0, g }
    }

    pub fn process(&mut self, input: f64) -> f64 {
        // + prev
        let mut out = input + self.prev * self.g;
        //     this point goes to prev
        out = out * self.g.neg();
        out = out + self.prev;
        self.prev = out;
        out
    }
}
