use super::{AudioPlayer, StereoOutput};
use sdl2::audio::{AudioQueue, AudioSpecDesired};

pub struct SDLAudioPlayer {
    queue: AudioQueue<f32>,
}

impl SDLAudioPlayer {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();
        let queue = audio_subsystem
            .open_queue(
                None,
                &AudioSpecDesired {
                    freq: Some(44100),
                    channels: Some(2),
                    samples: None,
                },
            )
            .unwrap();
        SDLAudioPlayer { queue }
    }
}

impl AudioPlayer for SDLAudioPlayer {
    fn play(&mut self, stereo_output: StereoOutput) {
        let left = stereo_output.left;
        let right = stereo_output.right;
        let mut interleaved: Vec<f32> = left.chunks(1).zip(right.chunks(1)).flat_map(|(a,b)| a.into_iter().chain(b)).copied().collect();
        self.queue.queue(interleaved.as_slice());
        self.queue.resume();
    }

    fn sample_rate(&self) -> u32 {
        44100
    }
}
