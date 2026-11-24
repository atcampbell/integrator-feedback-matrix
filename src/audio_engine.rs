use web_sys::console;

use crate::shared_params::{Params, PointBuffer, Trigger};
use crate::synth::Synth;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample, Stream};

const NUM_CHANNELS: u32 = 9;

pub fn start(params: Params, points: PointBuffer, randomise: Trigger) -> Result<Stream, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("failed to find a default output device")?;
    let supported = device.default_output_config().map_err(|e| e.to_string())?;
    let config = supported.config();

    match supported.sample_format() {
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, config, params, points, randomise),
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, config, params, points, randomise),
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, config, params, points, randomise),
        other => Err(format!("unsupported sample format: {:?}", other)),
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    params: Params,
    points: PointBuffer,
    randomise: Trigger,
) -> Result<Stream, String>
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate as f64;
    let channels = config.channels as usize;

    let mut synth = Synth::new(sample_rate, NUM_CHANNELS);

    let err_fn = |err| console::error_1(&format!("an error occurred on stream: {}", err).into());

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
                if randomise.take() {
                    synth.randomise_levels();
                }
                synth.apply_params(&params);
                let mut writer = points.writer();
                write_data(data, channels, || synth.process(&mut writer));
            },
            err_fn,
            None,
        )
        .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    Ok(stream)
}

fn write_data<T>(output: &mut [T], channels: usize, mut next_frame: impl FnMut() -> (f32, f32))
where
    T: SizedSample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let (left, right) = next_frame();
        match frame {
            [mono] => *mono = T::from_sample((left + right) * 0.5),
            [l, r, rest @ ..] => {
                *l = T::from_sample(left);
                *r = T::from_sample(right);
                for s in rest {
                    *s = T::from_sample(0.0f32);
                }
            }
            [] => {}
        }
    }
}
