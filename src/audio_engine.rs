use web_sys::console;

use crate::shared_params::{PointBuffer, SharedParam};
use crate::synth::Synth;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample, Stream};

pub fn start(dt: SharedParam, points: PointBuffer) -> Result<Stream, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("failed to find a default output device")?;
    let supported = device.default_output_config().map_err(|e| e.to_string())?;
    let config = supported.config();

    match supported.sample_format() {
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, config, points),
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, config, points),
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, config, points),
        other => Err(format!("unsupported sample format: {:?}", other)),
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    points: PointBuffer,
) -> Result<Stream, String>
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate as f64;
    let channels = config.channels as usize;

    let mut synth = Synth::new(sample_rate);

    let err_fn = |err| console::error_1(&format!("an error occurred on stream: {}", err).into());

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
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

fn write_data<T>(output: &mut [T], channels: usize, mut next_sample: impl FnMut() -> f32)
where
    T: SizedSample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
