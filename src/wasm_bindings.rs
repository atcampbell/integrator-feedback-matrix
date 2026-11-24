// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use crate::audio_engine;
use crate::shared_params::{Params, PointBuffer, Trigger};
use cpal::Stream;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

const MAX_POINTS: usize = 8192;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    Ok(())
}

#[wasm_bindgen]
pub struct Handle {
    _stream: Stream,
    params: Params,
    points: PointBuffer,
    randomise: Trigger,
}

#[wasm_bindgen]
impl Handle {
    /// Update one named parameter. Throws in JS if the name is unknown.
    pub fn set_param(&self, name: &str, value: f64) -> Result<(), JsValue> {
        self.params
            .set(name, value)
            .map_err(|e| JsValue::from_str(&e))
    }

    pub fn randomise_channel_gain(&self) {
        self.randomise.fire();
    }

    /// Drain every point generated since the last call. The x, y and z
    /// arrays are taken together so they always line up.
    pub fn take_points(&self) -> Points {
        let (x, y, z) = self.points.take();
        Points { x, y, z }
    }
}

/// One batch of Lorenz points, split by axis. In JS each getter returns a
/// `Float32Array`; `x[i]`, `y[i]` and `z[i]` are the same point.
#[wasm_bindgen]
pub struct Points {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
}

#[wasm_bindgen]
impl Points {
    #[wasm_bindgen(getter)]
    pub fn x(&self) -> Vec<f32> {
        self.x.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> Vec<f32> {
        self.y.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn z(&self) -> Vec<f32> {
        self.z.clone()
    }
}

#[wasm_bindgen]
/// Start the synth. `names[i]` is a parameter name and `values[i]` its
/// starting value; later updates go through `Handle::set_param`.
pub fn beep(names: Vec<String>, values: Vec<f64>) -> Result<Handle, JsValue> {
    let params = Params::new(names, values).map_err(|e| JsValue::from_str(&e))?;
    let points = PointBuffer::new(MAX_POINTS);
    let randomise = Trigger::new();

    let stream = audio_engine::start(params.clone(), points.clone(), randomise.clone())
        .map_err(|e| JsValue::from_str(&e))?;

    Ok(Handle {
        _stream: stream,
        params,
        points,
        randomise,
    })
}
