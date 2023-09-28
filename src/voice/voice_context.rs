use std::{collections::VecDeque, sync::{Arc, Mutex}};

pub struct VoiceContext
{
    pub input_stream: cpal::Stream,
    pub output_stream: cpal::Stream,

    pub decoder: opus::Decoder,
    pub encoder: opus::Encoder,

    pub input_resampler: rubato::FftFixedOut<f32>,
    pub output_resampler: rubato::FftFixedIn<f32>,

    pub input_resampler_buffer: Vec<Vec<f32>>,
    pub output_resampler_buffer: Vec<Vec<f32>>,

    pub input_channel: Arc<Mutex<VecDeque<f32>>>,
    pub output_channel: Arc<Mutex<VecDeque<f32>>>,
}