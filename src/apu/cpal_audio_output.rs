use super::AudioLoop;
use crate::cpu::{CPU, CPU_CLOCK_RATE_HZ};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Stream};

pub struct CpalAudioLoop {
    stream: Stream,
}

pub enum CpalCreationError {
    DeviceFetchFailed,
}

impl CpalAudioLoop {
    pub fn new(mut cpu: CPU) -> Result<Self, CpalCreationError> {
        let audio_host = cpal::default_host();
        let audio_device = audio_host.default_output_device();
        let audio_supported_configs_range =
            (&audio_device).as_ref().unwrap().supported_output_configs();
        let audio_supported_config = audio_supported_configs_range
            .unwrap()
            .next()
            .unwrap()
            .with_max_sample_rate();
        let mut audio_config = (&audio_supported_config.config()).clone();
        audio_config.buffer_size = match audio_supported_config.buffer_size() {
            cpal::SupportedBufferSize::Range { min, max } => {
                let ideal_samples = 10 * CPU_CLOCK_RATE_HZ / audio_config.sample_rate.0;
                let buffer_size = if ideal_samples > *max {
                    *max
                } else if ideal_samples < *min {
                    *min
                } else {
                    ideal_samples
                };
                BufferSize::Fixed(buffer_size)
            }
            cpal::SupportedBufferSize::Unknown => BufferSize::Default,
        };

        cpu.bus
            .apu
            .initialize_buffers(audio_config.sample_rate.0, CPU_CLOCK_RATE_HZ);

        let stream = audio_device.unwrap().build_output_stream(
            &audio_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let samples_needed = data.len() / 2;
                <dyn AudioLoop>::run_cycles_for_desired_samples(samples_needed as u32, &mut cpu);
                let mut samples = cpu.bus.apu.gather_samples();
                let flattened_samples = samples.interleave();
                data[..flattened_samples.len()].copy_from_slice(flattened_samples.as_slice());
            },
            |err| eprintln!("Error occurred on the output audio stream: {:?}", err),
        );

        let audio_player = CpalAudioLoop {
            stream: stream.unwrap(),
        };
        audio_player.stream.play().unwrap();

        Ok(audio_player)
    }
}

impl AudioLoop for CpalAudioLoop {}
