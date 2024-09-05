use super::audio_data::AudioData;
use super::audio_stream::PlaybackContext;
use basedrop::{Collector, Handle, Owned, Shared, SharedCell};
use creek::read::ReadError;
use creek::{Decoder, ReadDiskStream, SeekMode, SymphoniaDecoder};
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};
use rubato::{
    Resampler, SincFixedIn, SincFixedOut, SincInterpolationParameters, SincInterpolationType,
    WindowFunction,
};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Playing,
    Stopped,
}

enum PlayerAction {
    Seek(usize),
    Play,
    Pause,
    Stop,
    Restart,
    UseStream(Owned<ReadDiskStream<SymphoniaDecoder>>),
    //Volume(f32),
    SetLoop { start: usize, end: usize },
}

#[derive(Lens)]
pub struct SamplePlayerController {
    #[lens(ignore)]
    tx: HeapProd<PlayerAction>,
    #[lens(ignore)]
    collector: Handle,
    pub playhead: Arc<AtomicUsize>,
    pub should_loop: Arc<AtomicBool>,
    pub play_state: PlayerState,
    pub sample_rate: Option<f64>,
    pub num_channels: Option<usize>,
    pub num_samples: Option<usize>,
}

/// create a new sample player and its controller
pub fn sample_player(c: &Collector) -> (Process, SamplePlayerController) {
    let playhead = Arc::new(AtomicUsize::new(0));
    let should_loop = Arc::new(AtomicBool::new(true));

    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    let resampler = SincFixedOut::new(1.0, 2.0, params, 512, 2).unwrap();

    let (tx, rx) = HeapRb::new(2048).split();

    (
        Process {
            read_disk_stream: None,
            rx,
            input_count: 0,
            output_buffer: HeapRb::new(8192),
            process_buffer: vec![0.0f32; 1024],
            resample_buffer_in: resampler.input_buffer_allocate(true),
            resample_buffer_out: resampler.output_buffer_allocate(true),
            resampler,
            playhead: playhead.clone(),
            should_loop: should_loop.clone(),
            playback_state: PlayerState::Stopped,
            had_cache_miss_last_cycle: false,
            loop_start: 0,
            loop_end: 0,
            fatal_error: false,
        },
        SamplePlayerController {
            tx,
            playhead: playhead.clone(),
            should_loop: should_loop.clone(),
            collector: c.handle(),
            sample_rate: None,
            num_channels: None,
            num_samples: None,
            // file: None,
            play_state: PlayerState::Stopped,
        },
    )
}

#[allow(dead_code)]
impl SamplePlayerController {
    fn send_msg(&mut self, playeraction: PlayerAction) {
        let mut e = self.tx.try_push(playeraction);
        while let Err(playeraction) = e {
            e = self.tx.try_push(playeraction);
        }
    }

    pub fn seek(&mut self, seconds: usize) {
        self.send_msg(PlayerAction::Seek(seconds));
    }

    pub fn playhead_position(&self) -> usize {
        self.playhead.load(Ordering::SeqCst)
    }

    pub fn play(&mut self) {
        self.play_state = PlayerState::Playing;
        self.send_msg(PlayerAction::Play);
    }

    pub fn stop(&mut self) {
        self.play_state = PlayerState::Stopped;
        self.send_msg(PlayerAction::Stop);
    }

    pub fn pause(&mut self) {
        self.play_state = PlayerState::Stopped;
        self.send_msg(PlayerAction::Pause);
    }

    // pub fn volume(&mut self, val: f32) {
    //     self.send_msg(PlayerAction::Volume(val));
    // }

    pub fn load_file(&mut self, audio_file: Owned<ReadDiskStream<SymphoniaDecoder>>) {
        self.send_msg(PlayerAction::UseStream(audio_file));
    }

    pub fn toggle_looping(&mut self) {
        self.should_loop.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| Some(!x));
    }
}

pub struct Process {
    read_disk_stream: Option<Owned<ReadDiskStream<SymphoniaDecoder>>>,

    rx: HeapCons<PlayerAction>,

    input_count: usize,
    output_buffer: HeapRb<f32>,

    process_buffer: Vec<f32>,

    resample_buffer_in: Vec<Vec<f32>>,
    resample_buffer_out: Vec<Vec<f32>>,
    resampler: SincFixedOut<f32>,

    playhead: Arc<AtomicUsize>,
    should_loop: Arc<AtomicBool>,

    playback_state: PlayerState,
    had_cache_miss_last_cycle: bool,

    loop_start: usize,
    loop_end: usize,

    fatal_error: bool,
}

unsafe impl Send for Process {}
unsafe impl Sync for Process {}

impl Process {
    pub fn process(&mut self, context: PlaybackContext) {
        if self.fatal_error {
            silence(context.output_buffer);
            return;
        }

        if let Err(e) = self.try_process(context.output_buffer, context.sample_rate) {
            if matches!(e, ReadError::FatalError(_)) {
                self.fatal_error = true;
            }

            println!("{:?}", e);
            silence(context.output_buffer);
        }
    }

    fn try_process(
        &mut self,
        mut data: &mut [f32],
        sample_rate: f64,
    ) -> Result<(), ReadError<<SymphoniaDecoder as Decoder>::FatalError>> {
        // Process messages from GUI.
        while let Some(msg) = self.rx.try_pop() {
            match msg {
                PlayerAction::UseStream(read_disk_stream) => {
                    self.playback_state = PlayerState::Stopped;
                    self.loop_start = 0;
                    self.loop_end = 0;

                    self.read_disk_stream = Some(read_disk_stream);
                }
                PlayerAction::SetLoop { start, end } => {
                    self.loop_start = start;
                    self.loop_end = end;

                    if start != 0 {
                        if let Some(read_disk_stream) = &mut self.read_disk_stream {
                            // cache loop starting position (cache_index 1 = loop start cache)
                            read_disk_stream.cache(1, start)?;
                        }
                    }
                }
                PlayerAction::Play => {
                    self.playback_state = PlayerState::Playing;
                }
                PlayerAction::Pause => {
                    self.playback_state = PlayerState::Stopped;
                }
                PlayerAction::Stop => {
                    self.playback_state = PlayerState::Stopped;

                    if let Some(read_disk_stream) = &mut self.read_disk_stream {
                        read_disk_stream.seek(self.loop_start, SeekMode::Auto)?;

                        self.playhead.store(read_disk_stream.playhead(), Ordering::SeqCst);
                    }
                }
                PlayerAction::Restart => {
                    self.playback_state = PlayerState::Playing;

                    if let Some(read_disk_stream) = &mut self.read_disk_stream {
                        read_disk_stream.seek(self.loop_start, SeekMode::Auto)?;
                    }
                }
                PlayerAction::Seek(pos) => {
                    if let Some(read_disk_stream) = &mut self.read_disk_stream {
                        read_disk_stream.seek(pos, SeekMode::Auto)?;
                    }
                }
            }
        }

        silence(data);

        self.input_count += data.len();

        if self.input_count >= 1024 {
            let mut buffer = self.process_buffer.as_mut_slice();

            let mut cache_missed_this_cycle = false;
            if let Some(read_disk_stream) = &mut self.read_disk_stream {
                let file_sample_rate = read_disk_stream.info().sample_rate.unwrap_or(41000) as f64;

                // Update client and check if it is ready.

                if !read_disk_stream.is_ready()? {
                    cache_missed_this_cycle = true;
                    // We can choose to either continue reading (which will return silence),
                    // or pause playback until the buffer is filled. This demo uses the former.
                }

                if let PlayerState::Stopped = self.playback_state {
                    self.input_count = 0;
                    // Paused, do nothing.
                    silence(buffer);
                    return Ok(());
                }

                let num_frames = read_disk_stream.info().num_frames;
                let num_channels = usize::from(read_disk_stream.info().num_channels);

                // Keep reading data until output buffer is filled.

                self.resampler.set_resample_ratio(sample_rate / file_sample_rate, true);
                let read_frames = if file_sample_rate != sample_rate {
                    self.resampler.input_frames_next()
                } else {
                    buffer.len() / 2
                };

                let mut playhead = read_disk_stream.playhead();

                // If user seeks ahead of the loop end, continue playing until the end
                // of the file.
                // let loop_end = if playhead < self.loop_end { self.loop_end } else { num_frames };
                let loop_end = num_frames;

                let read_data = read_disk_stream.read(read_frames)?;

                let output_size = if file_sample_rate != sample_rate {
                    self.resample_buffer_in[0][0..read_data.num_frames()]
                        .copy_from_slice(read_data.read_channel(0));
                    if read_data.num_channels() == 2 {
                        self.resample_buffer_in[1][0..read_data.num_frames()]
                            .copy_from_slice(read_data.read_channel(1));
                    }

                    self.resampler.set_resample_ratio(sample_rate / file_sample_rate, true);

                    let (input_frames, output_frames) = self
                        .resampler
                        .process_into_buffer(
                            self.resample_buffer_in.as_slice(),
                            self.resample_buffer_out.as_mut_slice(),
                            None,
                        )
                        .unwrap();

                    output_frames
                } else {
                    self.resample_buffer_out[0][0..read_data.num_frames()]
                        .copy_from_slice(read_data.read_channel(0));
                    if read_data.num_channels() == 2 {
                        self.resample_buffer_out[1][0..read_data.num_frames()]
                            .copy_from_slice(read_data.read_channel(1));
                    }

                    read_data.num_frames()
                };

                playhead += read_data.num_frames();
                if playhead >= loop_end {
                    // Copy up to the end of the loop.
                    let to_end_of_loop = output_size - (playhead - loop_end);

                    if read_data.num_channels() == 1 {
                        let ch = self.resample_buffer_out[0].as_slice();

                        for i in 0..to_end_of_loop {
                            buffer[i * 2] = ch[i];
                            buffer[i * 2 + 1] = ch[i];
                        }
                    } else if read_data.num_channels() == 2 {
                        let ch1 = self.resample_buffer_out[0].as_slice();
                        let ch2 = self.resample_buffer_out[1].as_slice();

                        for i in 0..to_end_of_loop {
                            buffer[i * 2] = ch1[i];
                            buffer[i * 2 + 1] = ch2[i];
                        }
                    }

                    if self.should_loop.load(Ordering::SeqCst) {
                        read_disk_stream.seek(self.loop_start, SeekMode::Auto)?;
                    } else {
                        self.playback_state = PlayerState::Stopped;
                    }
                } else {
                    // Else copy all the read data.
                    if read_data.num_channels() == 1 {
                        let ch = self.resample_buffer_out[0].as_slice();

                        for i in 0..output_size {
                            buffer[i * 2] = ch[i];
                            buffer[i * 2 + 1] = ch[i];
                        }
                    } else if read_data.num_channels() == 2 {
                        let ch1 = self.resample_buffer_out[0].as_slice();
                        let ch2 = self.resample_buffer_out[1].as_slice();

                        for i in 0..output_size {
                            buffer[i * 2] = ch1[i];
                            buffer[i * 2 + 1] = ch2[i];
                        }
                    }
                }

                self.playhead.store(read_disk_stream.playhead(), Ordering::SeqCst);
            } else {
                // Output silence until file is received.
                silence(buffer);
            }

            // When the cache misses, the buffer is filled with silence. So the next
            // buffer after the cache miss is starting from silence. To avoid an audible
            // pop, apply a ramping gain from 0 up to unity.
            if self.had_cache_miss_last_cycle {
                let buffer_size = buffer.len() as f32;
                for (i, sample) in buffer.iter_mut().enumerate() {
                    *sample *= i as f32 / buffer_size;
                }
            }

            self.input_count -= 1024;

            self.output_buffer.push_slice(buffer);

            self.had_cache_miss_last_cycle = cache_missed_this_cycle;
        }

        if self.output_buffer.occupied_len() >= data.len() {
            self.output_buffer.pop_slice(data);
        }

        Ok(())
    }
}

fn silence(data: &mut [f32]) {
    for sample in data.iter_mut() {
        *sample = 0.0;
    }
}
