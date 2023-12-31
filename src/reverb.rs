use std::{sync::Arc, collections::VecDeque};
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
    delay_times: Arc<Mutex<VecDeque<i32>>>,
    decay: f32,
    buffer: Arc<Mutex<VecDeque<f32>>>,
    write_index: Arc<Mutex<usize>>,
    read_offset: usize,
    buf_changed: bool,
}

impl Reverb {
    pub(crate) fn new(delay_times: VecDeque<i32>, decay: f32, buffer_size: usize) -> Self {
        Reverb {
            delay_times: Arc::new(Mutex::new(delay_times)),
            decay,
            buffer: Arc::new(Mutex::new(VecDeque::from(vec![0.0; buffer_size]))),
            write_index: Arc::new(Mutex::new(0)),
            buf_changed: false,
            read_offset: 0,
        }
    }

    // Update to new delay times + decay when a parameter changes that affects either
    pub fn update(&mut self, delay_times: VecDeque<i32>, decay: f32) {
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
                let mut temp_buffer = VecDeque::from(vec![0.0; new_buffer_len]);
                //temp_buffer[..buffer_len].copy_from_slice(&buffer_lock);
                // Copy elements one by one from buffer_lock to temp_buffer
                for (i, val) in buffer_lock.iter().enumerate() {
                    temp_buffer[i] = *val;
                }
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
    pub(crate) fn generate_steps(input_number: i32, number_of_integers: i32, algorithm: ReverbType) -> VecDeque<i32> {
        let mut output_vector = VecDeque::new();
        for i in 1..number_of_integers {
            match algorithm {
                // Linear small
                // Reverb TDLs spaced evenly through the delay time at # of steps
                ReverbType::LinearSmall => {
                    let step = input_number / (number_of_integers);
                    output_vector.push_back(i * step);
                },
                // Exponential Swirl
                // Reverb TDLs expanding at delay^2 time for number of steps
                ReverbType::ExpSwirl => {
                    let mut value = input_number;
                    value *= 2;
                    output_vector.push_back(value);
                },
                // Geometric Phase
                // Reverb TDLs in a geometric sequence phasing slightly at delay^(1/step amount) * stack
                ReverbType::GeoPhase => {
                    let ratio = f32::powf(input_number as f32, 1.0 / number_of_integers as f32);
                    let value = input_number * ratio.floor() as i32;
                    output_vector.push_back(value);
                },
                // Quadratic Metal
                // Reverb TDLs in step - (step/steps) sequence (used to be quadratic...this sounds metallic)
                ReverbType::QuadMetal => {
                    let value = input_number;
                    let step = input_number / (number_of_integers);
                    output_vector.push_back(value - step);
                },
                // Specific Swirl
                // Reverb TDLs expanding at an arbitrary multiplier
                ReverbType::SpecificSwirl => {
                    let mut value = input_number;
                    value *= 4;
                    value -= 1;
                    output_vector.push_back(value);
                },
                // Chaos Steps
                // Not really sure how to describe but it sounds cool
                ReverbType::ChaosSteps => {
                    let mut x = input_number;
                    let y = number_of_integers;
                    let z = (x + y) / 2;
                    let step = input_number*2 - 1;
                    x = z + step;
                    //y = z - step;
                    output_vector.push_back(x);
                },
                // Golden Ratio
                ReverbType::GoldenRatio => {
                    let gr = 1.618;
                    let value = (gr * i as f32).floor() as i32;
                    output_vector.push_back(value);
                },
            }
        }
        output_vector
    }

    // Process a writable buffer
    pub(crate) fn process(&mut self, input: f32) -> f32 {
        let delay_times_lock = self.delay_times.lock();
        let mut buffer_lock = self.buffer.lock();
        let buffer_len = buffer_lock.len();
        let mut write_index = *self.write_index.lock();
        let output: f32;
        let mut delayed_sample = 0.0;

        for delay_time in delay_times_lock.iter() {
            let read_index: usize;
            // Was getting panic dividing by zero here
            if buffer_len == 0 {
                return input;
            }
            read_index = (self.read_offset + write_index + *delay_time as usize) % buffer_len;
            
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

    // Process without writing
    pub(crate) fn locked_buffer_process(&mut self, input: f32) -> f32 {
        let delay_times_lock = self.delay_times.lock();
        let buffer_lock = self.buffer.lock();
        let buffer_len = buffer_lock.len();
        let mut write_index = *self.write_index.lock();
        let mut delayed_sample = 0.0;

        for delay_time in delay_times_lock.iter() {
            let read_index: usize;
            // Was getting panic dividing by zero here
            if buffer_len == 0 {
                return input;
            }
            read_index = (self.read_offset + write_index + *delay_time as usize) % buffer_len;
            
            delayed_sample = buffer_lock[read_index] * self.decay;
            if delayed_sample < 1e-6 as f32 {
                delayed_sample = 0.0;
            }
        }

        if write_index >= buffer_len {
            write_index = 0;
        }

        // Skip this write of output to our buffer
        //buffer_lock[write_index] = output;
        *self.write_index.lock() = write_index + 1;

        drop(buffer_lock);
        drop(delay_times_lock);

        //output
        delayed_sample
    }

    // This is kind of a way to create an offset by shifting the write spot
    pub(crate) fn shift_buffer(&mut self, amount: i32) {
        // Lock these so at the time of modification nothing changes
        let buf_lock = self.buffer.lock();
        let write_index = *self.write_index.lock();
        self.read_offset = amount as usize;
        drop(buf_lock);
        drop(write_index);
    }
}
