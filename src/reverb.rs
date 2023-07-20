use std::{sync::Arc};
use nih_plug_egui::egui::mutex::Mutex;
use nih_plug::{prelude::Enum};

#[derive(Enum, PartialEq, Eq, Debug, Copy, Clone)]
pub enum ReverbType{
    #[name = "Alg:Linear Small"]
    LinearSmall,
    #[name = "Alg:Exp Swirl"]
    ExpSwirl,
    #[name = "Alg:Geo Phase"]
    GeoPhase,
    #[name = "Alg:Quad Metal"]
    QuadMetal,
    #[name = "Alg:Specific Swirl"]
    SpecificSwirl,
    #[name = "Alg:Chaos Steps"]
    ChaosSteps,
    #[name = "Alg:Golden Ratio"]
    GoldenRatio
}

#[derive(Clone)]
pub(crate) struct Reverb {
    delay_times: Arc<Mutex<Vec<i32>>>,
    decay: f32,
    buffer: Arc<Mutex<Vec<f32>>>,
    write_index: Arc<Mutex<usize>>,
    buf_changed: bool,
}

impl Reverb {
    pub(crate) fn new(delay_times: Vec<i32>, decay: f32, buffer_size: usize) -> Self {
        Reverb {
            delay_times: Arc::new(Mutex::new(delay_times)),
            decay,
            buffer: Arc::new(Mutex::new(vec![0.0; buffer_size])),
            write_index: Arc::new(Mutex::new(0)),
            buf_changed: false,
        }
    }

    // Update to new delay times + decay when a parameter changes that affects either
    pub fn update(&mut self, delay_times: Vec<i32>, decay: f32) {
        let mut buffer_lock = self.buffer.lock();
        let mut delay_times_lock = self.delay_times.lock();
        if *delay_times_lock != delay_times {
            *delay_times_lock = delay_times;
            self.buf_changed = true;
        }
        self.decay = decay;

        if self.buf_changed {
            self.buf_changed = false;
            let new_buffer_len = delay_times_lock.iter().sum::<i32>() as usize * 2;
            let buffer_len = buffer_lock.len();

            if new_buffer_len > buffer_len {
                let mut temp_buffer = vec![0.0; new_buffer_len];
                temp_buffer[..buffer_len].copy_from_slice(&buffer_lock);
                *buffer_lock = temp_buffer;

            } else if new_buffer_len < buffer_len {
                if *self.write_index.lock() >= new_buffer_len {
                    *self.write_index.lock() = new_buffer_len - 1;
                }
                buffer_lock.truncate(new_buffer_len);
            }
        }

        drop(buffer_lock);
        drop(delay_times_lock);
    }

    // This creates the vector for update() function's delay_times input
    pub(crate) fn generate_steps(input_number: i32, number_of_integers: i32, algorithm: ReverbType) -> Vec<i32> {
        let mut output_vector = Vec::new();
        for i in 1..number_of_integers {
            match algorithm {
                // Linear small
                // Reverb TDLs spaced evenly through the delay time at # of steps
                ReverbType::LinearSmall => {
                    let step = input_number / (number_of_integers);
                    output_vector.push(i * step);
                },
                // Exponential Swirl
                // Reverb TDLs expanding at delay^2 time for number of steps
                ReverbType::ExpSwirl => {
                    let mut value = input_number;
                    value *= 2;
                    output_vector.push(value);
                },
                // Geometric Phase
                // Reverb TDLs in a geometric sequence phasing slightly at delay^(1/step amount) * stack
                ReverbType::GeoPhase => {
                    let ratio = f32::powf(input_number as f32, 1.0 / number_of_integers as f32);
                    let value = input_number * ratio.floor() as i32;
                    output_vector.push(value);
                },
                // Quadratic Metal
                // Reverb TDLs in step - (step/steps) sequence (used to be quadratic...this sounds metallic)
                ReverbType::QuadMetal => {
                    let value = input_number;
                    let step = input_number / (number_of_integers);
                    output_vector.push(value - step);
                },
                // Specific Swirl
                // Reverb TDLs expanding at an arbitrary multiplier
                ReverbType::SpecificSwirl => {
                    let mut value = input_number;
                    value *= 4;
                    value -= 1;
                    output_vector.push(value);
                },
                // Chaos Steps
                // Not really sure how to descrive but it sounds cool
                ReverbType::ChaosSteps => {
                    let mut x = input_number;
                    let mut y = number_of_integers;
                    for _ in 1..number_of_integers {
                        let z = (x + y) / 2;
                        let step = input_number*2 - 1;
                        x = z + step;
                        y = z - step;
                        output_vector.push(x);
                    }
                },
                // Golden Ratio
                ReverbType::GoldenRatio => {
                    let gr = 1.618;
                    let value = (gr * i as f32).floor() as i32;
                    output_vector.push(value);
                },
            }
        }
        output_vector
    }

    pub(crate) fn process(&mut self, input: f32) -> f32 {
        let delay_times_lock = self.delay_times.lock();
        let mut buffer_lock = self.buffer.lock();
        let buffer_len = buffer_lock.len();
        let mut write_index = *self.write_index.lock();
        let output: f32;
        let mut delayed_sample = 0.0;

        for delay_time in delay_times_lock.iter() {
            let read_index = (write_index + *delay_time as usize) % buffer_len;
            delayed_sample = buffer_lock[read_index] * self.decay;
            if delayed_sample < 1e-6 as f32 {
                delayed_sample = 0.0;
            }
        }
        output = input + delayed_sample;

        if write_index >= buffer_len {
            write_index = 0;
        }

        buffer_lock[write_index] = output;
        *self.write_index.lock() = write_index + 1;

        drop(buffer_lock);
        drop(delay_times_lock);

        //output
        delayed_sample
    }
}
