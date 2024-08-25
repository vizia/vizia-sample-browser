use crate::utils::interleave;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;

use super::Process;

/// The playback context is used by the audio callback to map data from the audio
/// file to the playback buffer.
pub struct PlaybackContext<'a> {
    pub buffer_size: usize,
    pub sample_rate: f64,
    pub num_channels: usize,
    pub output_buffer: &'a mut [f32],
}

impl<'a> PlaybackContext<'a> {
    /// Return a buffer of output samples corresponding to a channel index
    pub fn get_output(&mut self, idx: usize) -> &'_ mut [f32] {
        let offset = idx * self.buffer_size;
        &mut self.output_buffer[offset..offset + self.buffer_size]
    }
}

/// Start the audio stream
pub fn audio_stream(mut main_callback: impl FnMut(PlaybackContext) + Send + 'static) -> Stream {
    let host = cpal::default_host();
    let output_device = host.default_output_device().expect("no output found");
    let config = output_device.default_output_config().expect("no default output config").config();

    let sample_rate = config.sample_rate.0 as f64;
    let num_channels = config.channels as usize;
    let mut output_buffer = vec![];
    let mut input_buffer = vec![];

    output_buffer.resize_with(1 << 16, || 0.0);
    input_buffer.resize_with(1 << 16, || 0.0);

    let callback = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        let buffer_size = data.len() / num_channels;
        output_buffer.resize(data.len(), 0.0);
        for sample in data.iter_mut() {
            *sample = 0.0;
        }
        for sample in &mut output_buffer.iter_mut() {
            *sample = 0.0;
        }

        let context =
            PlaybackContext { buffer_size, num_channels, sample_rate, output_buffer: data };

        main_callback(context);
        // interleave(&output_buffer, data, num_channels);
    };

    output_device
        .build_output_stream(&config, callback, |err| eprintln!("{}", err), None)
        .expect("failed to open stream")
}

// pub fn spawn_cpal_stream(process: Process) -> cpal::Stream {
//     // Setup cpal audio output

//     let host = cpal::default_host();

//     let device = host.default_output_device().expect("no output device available");

//     let sample_rate = device.default_output_config().unwrap().sample_rate();

//     let config =
//         cpal::StreamConfig { channels: 2, sample_rate, buffer_size: cpal::BufferSize::Default };

//     let stream = device
//         .build_output_stream(
//             &config,
//             move |data: &mut [f32], _: &cpal::OutputCallbackInfo| process.process(data),
//             move |err| {
//                 eprintln!("{}", err);
//             },
//             None,
//         )
//         .unwrap();

//     stream.play().unwrap();

//     stream
// }
