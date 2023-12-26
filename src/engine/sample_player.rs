use super::audio_data::AudioData;
use super::audio_stream::PlaybackContext;
use basedrop::{Collector, Handle, Shared};
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

enum PlayerState {
    Playing,
    Stopped,
}

enum PlayerAction {
    Seek(f64),
    Scrub(f64),
    Play,
    Stop,
    SetActive(usize, bool),
    NewFile(Shared<AudioData>),
    Volume(f32),
}

pub struct SamplePlayer {
    file: Option<Shared<AudioData>>,
    active: [bool; 32],
    playhead: Arc<AtomicUsize>,
    state: PlayerState,
    rx: HeapConsumer<PlayerAction>,
    volume: f32,
}

pub struct SamplePlayerController {
    tx: HeapProducer<PlayerAction>,
    playhead: Arc<AtomicUsize>,
    collector: Handle,
    sample_rate: Option<f64>,
    num_channels: Option<usize>,
    num_samples: Option<usize>,
    pub file: Option<Shared<AudioData>>,
}

/// create a new sample player and its controller
pub fn sample_player(c: &Collector) -> (SamplePlayer, SamplePlayerController) {
    let playhead = Arc::new(AtomicUsize::new(0));
    let (tx, rx) = HeapRb::new(2048).split();
    (
        SamplePlayer {
            file: None,
            active: [true; 32],
            playhead: playhead.clone(),
            state: PlayerState::Stopped,
            rx,
            volume: 1.0,
        },
        SamplePlayerController {
            tx,
            playhead: playhead.clone(),
            collector: c.handle(),
            sample_rate: None,
            num_channels: None,
            num_samples: None,
            file: None,
        },
    )
}

impl SamplePlayer {
    pub fn playhead(&self) -> usize {
        self.playhead.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn advance(&mut self, context: &mut PlaybackContext) {
        while let Some(msg) = self.rx.pop() {
            match msg {
                PlayerAction::Seek(pos) => {
                    if let Some(f) = &self.file {
                        self.playhead.store(
                            ((f.sample_rate * pos) as usize).min(f.num_samples),
                            Ordering::SeqCst,
                        );
                    }
                }
                PlayerAction::NewFile(file) => {
                    self.file = Some(file);
                }
                PlayerAction::Scrub(_) => {
                    //todo...
                }
                PlayerAction::SetActive(channel, active) => {
                    self.active[channel] = active;
                }
                PlayerAction::Play => self.state = PlayerState::Playing,
                PlayerAction::Stop => self.state = PlayerState::Stopped,
                PlayerAction::Volume(val) => self.volume = val,
            }
        }

        if let PlayerState::Stopped = self.state {
            return;
        }

        if let Some(file) = &self.file {
            if self.playhead() >= file.num_samples {
                self.state = PlayerState::Stopped;
                return;
            }

            for channel in 0..context.num_channels.max(file.num_channels) {
                if !self.active[channel] {
                    continue;
                }
                let start = channel * file.num_samples + self.playhead().min(file.num_samples);
                let end = channel * file.num_samples
                    + (self.playhead() + context.buffer_size).min(file.num_samples);
                context.get_output(channel)[0..(end - start)]
                    .copy_from_slice(&file.data[start..end]);
                context.get_output(channel)[0..(end - start)]
                    .iter_mut()
                    .for_each(|sample| *sample = *sample * self.volume);
            }

            self.playhead.fetch_add(context.buffer_size, Ordering::SeqCst);
        }
    }
}

#[allow(dead_code)]
impl SamplePlayerController {
    pub fn sample_rate(&self) -> Option<f64> {
        self.sample_rate
    }

    pub fn duration_samples(&self) -> Option<usize> {
        self.num_samples
    }

    pub fn num_channels(&self) -> Option<usize> {
        self.num_channels
    }

    fn send_msg(&mut self, playeraction: PlayerAction) {
        let mut e = self.tx.push(playeraction);
        while let Err(playeraction) = e {
            e = self.tx.push(playeraction);
        }
    }

    pub fn seek(&mut self, seconds: f64) {
        self.send_msg(PlayerAction::Seek(seconds));
    }

    pub fn playhead(&self) -> usize {
        self.playhead.load(Ordering::SeqCst)
    }

    pub fn play(&mut self) {
        self.send_msg(PlayerAction::Play);
    }

    pub fn stop(&mut self) {
        self.send_msg(PlayerAction::Stop);
    }

    pub fn scrub(&mut self, seconds: f64) {
        self.send_msg(PlayerAction::Scrub(seconds));
    }

    pub fn set_active(&mut self, channel_index: usize, active: bool) {
        self.send_msg(PlayerAction::SetActive(channel_index, active));
    }

    pub fn volume(&mut self, val: f32) {
        self.send_msg(PlayerAction::Volume(val));
    }

    pub fn load_file(&mut self, s: &str) {
        let audio_file =
            Shared::new(&self.collector, AudioData::open(s).expect("file does not exist"));
        self.num_samples = Some(audio_file.num_samples);
        self.num_channels = Some(audio_file.num_channels);
        self.sample_rate = Some(audio_file.sample_rate);
        self.file = Some(Shared::clone(&audio_file));
        self.send_msg(PlayerAction::NewFile(audio_file));
    }

    pub fn get_magnitude(&self, sample_idx: usize) -> f32 {
        if let Some(file) = &self.file {
            let ldx = sample_idx;
            let rdx = sample_idx + file.num_samples;
            (file.data[ldx].abs() + file.data[rdx].abs()) / 2.0
        } else {
            0.0
        }
    }
}
