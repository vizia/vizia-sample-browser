use std::cmp::Ordering;
use vizia::prelude::Data;

#[derive(Data, Clone, PartialEq)]
pub struct Waveform {
    pub data: Vec<(f32, f32)>,
    pub samples_per_pixel: usize,
    pub remainder: Vec<f32>,
}

impl Waveform {
    pub fn new() -> Self {
        Self { data: Vec::new(), samples_per_pixel: 0, remainder: Vec::new() }
    }

    pub fn load(&mut self, audio: &[f32], num_of_pixels: usize) {
        self.data.clear();

        self.samples_per_pixel = (audio.len() as f32 / num_of_pixels as f32).ceil() as usize;

        let chunks = audio.chunks(self.samples_per_pixel);
        for chunk in chunks {
            let v_min =
                *chunk.iter().min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)).unwrap();
            let v_max =
                *chunk.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)).unwrap();
            self.data.push((v_min, v_max));
        }
    }

    pub fn append(&mut self, data: &[f32], samples_per_pixel: usize) {
        self.samples_per_pixel = samples_per_pixel;
        self.remainder.extend(data.iter());

        let mut new_remainder = Vec::new();
        let chunks = self.remainder.chunks(samples_per_pixel);
        for chunk in chunks {
            if chunk.len() == samples_per_pixel {
                let v_min = *chunk
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                    .unwrap();
                let v_max = *chunk
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                    .unwrap();
                self.data.push((v_min, v_max));
            } else {
                new_remainder = chunk.to_owned();
            }
        }

        self.remainder = new_remainder;
    }

    // pub fn set_num_pixels(&mut self, audio: &[f32], num_of_pixels: usize) {
    //     if num_of_pixels > 0 {
    //         if let Some(last) = self.index.last() {
    //             let samples_per_pixel = audio.len() / num_of_pixels;
    //             let chunks = audio.chunks(samples_per_pixel);
    //             for (idx, chunk) in chunks.enumerate() {
    //                 let v_min = *chunk
    //                     .iter()
    //                     .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
    //                     .unwrap();
    //                 let v_max = *chunk
    //                     .iter()
    //                     .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
    //                     .unwrap();
    //                 if last + idx < self.data.len() {
    //                     self.data[last + idx] = (v_min, v_max)
    //                 } else {
    //                     self.data.push((v_min, v_max));
    //                 }
    //             }
    //         }
    //     }
    // }

    pub fn get_data(&self, level: usize) -> Option<&[(f32, f32)]> {
        return Some(&self.data);
    }
}
