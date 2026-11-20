use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone)]
pub struct Trigger(Arc<AtomicBool>);

impl Trigger {
    pub fn new() -> Self {
        Trigger(Arc::new(AtomicBool::new(false)))
    }

    /// Request the action. Called from JS.
    pub fn fire(&self) {
        self.0.store(true, Ordering::Relaxed)
    }

    /// Returns true once per `fire`, clearing the flag. Called from the audio callback.
    pub fn take(&self) -> bool {
        self.0.swap(false, Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub struct SharedParam(Arc<AtomicU64>);

impl SharedParam {
    pub fn new(value: f64) -> Self {
        SharedParam(Arc::new(AtomicU64::new(value.to_bits())))
    }

    pub fn get(&self) -> f64 {
        f64::from_bits(self.0.load(Ordering::Relaxed))
    }

    pub fn set(&self, value: f64) {
        self.0.store(value.to_bits(), Ordering::Relaxed)
    }
}

type Points = VecDeque<(f32, f32, f32)>;

/// A bounded queue of (x, y, z) points. The audio callback writes to it and
/// the UI drains it. When full, the oldest points are dropped, so it can't
/// grow without limit if the UI stops reading (e.g. a background tab).
#[derive(Clone)]
pub struct PointBuffer {
    points: Arc<Mutex<Points>>,
    capacity: usize,
}

impl PointBuffer {
    pub fn new(capacity: usize) -> Self {
        PointBuffer {
            points: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
        }
    }

    /// Lock the buffer for writing. Holding one `PointWriter` for a whole
    /// audio buffer means locking once per buffer rather than per sample.
    pub fn writer(&self) -> PointWriter<'_> {
        PointWriter {
            points: lock(&self.points),
            capacity: self.capacity,
        }
    }

    /// Remove all points and return them as separate `(xs, ys, zs)` arrays.
    /// Index `i` in each array belongs to the same point.
    pub fn take(&self) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let mut points = lock(&self.points);
        let len = points.len();
        let mut xs = Vec::with_capacity(len);
        let mut ys = Vec::with_capacity(len);
        let mut zs = Vec::with_capacity(len);
        for (x, y, z) in points.drain(..) {
            xs.push(x);
            ys.push(y);
            zs.push(z);
        }
        (xs, ys, zs)
    }
}

pub struct PointWriter<'a> {
    points: MutexGuard<'a, Points>,
    capacity: usize,
}

impl PointWriter<'_> {
    pub fn push(&mut self, x: f32, y: f32, z: f32) {
        if self.points.len() == self.capacity {
            self.points.pop_front();
        }
        self.points.push_back((x, y, z));
    }
}

/// Lock a mutex, recovering the data even if a previous holder panicked.
/// The data is just a list of points, so it's always safe to keep using.
fn lock(points: &Mutex<Points>) -> MutexGuard<'_, Points> {
    points
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
