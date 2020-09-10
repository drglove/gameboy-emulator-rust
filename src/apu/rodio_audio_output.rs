use rodio::{Device, DeviceTrait};
use super::{AudioPlayer, StereoOutput};
use rodio::buffer::SamplesBuffer;

pub struct RodioAudioPlayer {
    device: Device,
}

impl RodioAudioPlayer {
    pub fn new() -> Self {
        let device = rodio::default_output_device().unwrap();
        RodioAudioPlayer { device }
    }
}

impl AudioPlayer for RodioAudioPlayer {
    fn play(&mut self, stereo_output: StereoOutput) {
        let format = self.device.default_output_format().unwrap();
        let left = stereo_output.left;
        let right = stereo_output.right;
        let mut interleaved: Vec<f32> = left.chunks(1).zip(right.chunks(1)).flat_map(|(a,b)| a.into_iter().chain(b)).copied().collect();
        let buffer = SamplesBuffer::new(format.channels, format.sample_rate.0, interleaved);
        rodio::play_raw(&self.device, buffer);
    }

    fn sample_rate(&self) -> u32 {
        self.device.default_output_format().unwrap().sample_rate.0
    }
}
