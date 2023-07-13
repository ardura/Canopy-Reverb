use std::{sync::Arc, f32::consts::PI};
use nih_plug_egui::egui::mutex::Mutex;

#[derive(Clone)]
pub(crate) struct Reverb {
    delay_times: Arc<Mutex<Vec<i32>>>,
    decay: f32,
    buffer: Arc<Mutex<Vec<f32>>>,
    //input_copy_buffer: Arc<Mutex<Vec<f32>>>,
    write_index: Arc<Mutex<usize>>,
    buf_changed: bool,
}

impl Reverb {
    pub(crate) fn new(delay_times: Vec<i32>, decay: f32, buffer_size: usize) -> Self {
        Reverb {
            delay_times: Arc::new(Mutex::new(delay_times)),
            decay,
            buffer: Arc::new(Mutex::new(vec![0.0; buffer_size])),
            //input_copy_buffer: Arc::new(Mutex::new(vec![0.0; buffer_size])),
            write_index: Arc::new(Mutex::new(0)),
            buf_changed: false,
        }
    }

    pub fn update(&mut self, delay_times: Vec<i32>, decay: f32) {
        let mut buffer_lock = self.buffer.lock();
        //let mut input_copy_buffer_lock = self.input_copy_buffer.lock();

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

                //let mut temp_input_buffer = vec![0.0; new_buffer_len];
                //temp_input_buffer[..buffer_len].copy_from_slice(&input_copy_buffer_lock);
                //*input_copy_buffer_lock = temp_input_buffer;
            } else if new_buffer_len < buffer_len {
                if *self.write_index.lock() >= new_buffer_len {
                    *self.write_index.lock() = new_buffer_len - 1;
                }
                buffer_lock.truncate(new_buffer_len);
                //input_copy_buffer_lock.truncate(new_buffer_len);
            }
        }

        drop(buffer_lock);
        //drop(input_copy_buffer_lock);
        drop(delay_times_lock);
    }

    /*
    pub(crate) fn display(&mut self) -> String {
        format!(
            "Delay times: {:?}, Decay: {}, Buffer size: {}",
            *self.delay_times.lock(),
            self.decay,
            self.buffer.lock().len()
        )
    }
    */

    pub(crate) fn generate_steps(input_number: i32, number_of_integers: i32, algorithm: i32) -> Vec<i32> {
        let mut output_vector = Vec::new();
        for i in 0..number_of_integers {
            match algorithm {
                // Logarithmic
                1 => {
                    output_vector.push(input_number * f32::powf(10.0, -i as f32).round() as i32);
                },
                // Exponential
                2 => {
                    output_vector.push(input_number * (2 as i32).pow(i as u32));
                },
                // Sinusoidal
                3 => {
                    output_vector.push(input_number * f32::sin(PI * i as f32 / (number_of_integers - 1) as f32).round() as i32);
                },
                // S-Curve
                4 => {
                    output_vector.push(input_number * f32::tanh((i as f32 / (number_of_integers - 1) as f32) * PI).round() as i32);
                },
                // S-Curve with wobbles
                5 => {
                    let s_curve_value = input_number * f32::tanh((i as f32 / (number_of_integers - 1) as f32) * PI).round() as i32;
                    let wobble_value = (s_curve_value as f32 * f32::sin((i as f32 / (number_of_integers - 1) as f32) * 2.0 * PI)).round() as i32;
                    output_vector.push(s_curve_value + wobble_value);
                },
                // Logarithmic with wobbles
                6 => {
                    let logarithmic_value = input_number * f32::powf(10.0, -i as f32).round() as i32;
                    let wobble_value = (logarithmic_value as f32 * f32::sin((i as f32 / (number_of_integers - 1) as f32) * 2.0 * PI)).round() as i32;
                    output_vector.push(logarithmic_value + wobble_value);
                },
                _ => {}
            }
        }
        output_vector
    }

    pub(crate) fn process(&mut self, input: f32) -> f32 {
        let delay_times_lock = self.delay_times.lock();
        let mut buffer_lock = self.buffer.lock();
        //let mut input_buffer_lock = self.input_copy_buffer.lock();
        let buffer_len = buffer_lock.len();
        let mut write_index = *self.write_index.lock();
        let mut output = 0.0;

        for delay_time in delay_times_lock.iter() {
            let read_index = (write_index + *delay_time as usize) % buffer_len;
            let delayed_sample = buffer_lock[read_index] * self.decay;
            //let input_sample = input_buffer_lock[read_index];
            //output += input + delayed_sample - input_sample;
            output += input + delayed_sample;
            //output += delayed_sample;
        }

        if write_index >= buffer_len {
            write_index = 0;
        }

        buffer_lock[write_index] = output;
        //input_buffer_lock[write_index] = input;
        *self.write_index.lock() = write_index + 1;

        drop(buffer_lock);
        //drop(input_buffer_lock);
        drop(delay_times_lock);

        output
    }
}
