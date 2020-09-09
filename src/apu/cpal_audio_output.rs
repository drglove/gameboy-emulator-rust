use super::{AudioPlayer, StereoOutput};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Stream, StreamConfig};
use ringbuf::{RingBuffer, Producer};

pub struct CpalAudioPlayer {
    producer: Producer<Sample>,
    stream: Stream,
    stream_config: StreamConfig,
}

struct Sample {
    left: f32,
    right: f32,
}

pub enum CpalCreationError {
    DeviceFetchFailed,
}

impl CpalAudioPlayer {
    pub fn new() -> Result<Self, CpalCreationError> {
        let audio_host = cpal::default_host();
        let audio_device = audio_host.default_output_device();
        let audio_supported_configs_range = (&audio_device).as_ref().unwrap().supported_output_configs();
        let audio_supported_config = audio_supported_configs_range
            .unwrap()
            .next()
            .unwrap()
            .with_max_sample_rate();
        let mut audio_config = (&audio_supported_config.config()).clone();
        audio_config.buffer_size = match audio_supported_config.buffer_size() {
            cpal::SupportedBufferSize::Range { min, .. } => BufferSize::Fixed(*min),
            cpal::SupportedBufferSize::Unknown => BufferSize::Default,
        };

        let buffer = RingBuffer::new(audio_config.sample_rate.0 as usize);
        let (producer, mut consumer) = buffer.split();
        let stream = audio_device.unwrap().build_output_stream(
            &audio_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample_to_set in data.chunks_mut(2) {
                    let sample = consumer.pop().unwrap_or(Sample {
                        left: 0f32,
                        right: 0f32,
                    });
                    sample_to_set[0] = cpal::Sample::from::<f32>(&sample.left);
                    sample_to_set[1] = cpal::Sample::from::<f32>(&sample.right);
                }
            },
            |err| eprintln!("Error occurred on the output audio stream: {:?}", err),
        );

        let audio_player = CpalAudioPlayer {
            producer,
            stream: stream.unwrap(),
            stream_config: audio_config,
        };
        audio_player.stream.play().unwrap();

        Ok(audio_player)
    }
}

impl AudioPlayer for CpalAudioPlayer {
    fn play(&mut self, stereo_output: StereoOutput) {
        for (left, right) in stereo_output.left.iter().zip(stereo_output.right.iter()) {
            self.producer.push(Sample {
                left: *left,
                right: *right,
            }).ok().unwrap();
        }
    }

    fn sample_rate(&self) -> u32 {
        self.stream_config.sample_rate.0
    }
}
