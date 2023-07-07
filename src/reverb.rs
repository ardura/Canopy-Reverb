use std::sync::Arc;

use nih_plug_egui::egui::mutex::Mutex;

pub(crate)struct Reverb {
    delay_time: Arc<Mutex<i32>>,
    decay: f32,
    buffer: Arc<Mutex<Vec<f32>>>,
    write_index: usize,
    buf_changed: bool,
}

impl Reverb {
    pub(crate)fn new(delay_time: i32, decay: f32, buffer_size: usize) -> Self {
        Reverb {
            delay_time: Arc::new(Mutex::new(delay_time)),
            decay,
            buffer: Arc::new(Mutex::new(vec![0.0; buffer_size])),
            write_index: 0,
            buf_changed: false,
        }
    }

    pub fn update(&mut self, delay_time: i32, decay: f32) {
        let mut delay_time_lock = self.delay_time.lock();
        if *delay_time_lock != delay_time {
            *delay_time_lock = delay_time;
            self.buf_changed = true;
        }
        self.decay = decay;
    
        if self.buf_changed {
            self.buf_changed = false;
            let new_buffer_len = *delay_time_lock as usize * 4;
    
            {
                let mut buffer_lock = self.buffer.lock();
                let buffer_len = buffer_lock.len();
    
                if new_buffer_len > buffer_len {
                    // Create a temporary buffer with the new size
                    let mut temp_buffer = vec![0.0; new_buffer_len];
    
                    // Copy the existing buffer to the temporary buffer
                    temp_buffer[..buffer_len].copy_from_slice(&buffer_lock);
    
                    // Replace the buffer with the temporary buffer
                    *buffer_lock = temp_buffer;
                } else if new_buffer_len < buffer_len {
                    // Create a temporary buffer to store required samples
                    let mut temp_buffer = vec![0.0; new_buffer_len];
    
                    // Copy required samples to the temporary buffer
                    temp_buffer.copy_from_slice(&buffer_lock[..new_buffer_len]);
    
                    // Apply fade-out to the delayed samples in the temporary buffer
                    let fade_samples = buffer_len - new_buffer_len;
                    let fade_factor: f32 = 0.5; // Adjust this value to control the fade-out rate
    
                    for i in 0..fade_samples {
                        let fade_multiplier: f32 = fade_factor.powi(i as i32);
                        temp_buffer[i] *= fade_multiplier;
                    }
    
                    // Replace the buffer with the faded temporary buffer
                    *buffer_lock = temp_buffer;
                }
            }
        }
    }
    
    pub(crate) fn process(&mut self, input: f32) -> f32 {
        let delaylock = *self.delay_time.lock();
        let mut bufferlock = self.buffer.lock();
        let buffer_len = bufferlock.len();
        let read_index = (self.write_index + delaylock as usize) % buffer_len;
        let delayed_sample = bufferlock[read_index] * self.decay;
        let output = input + delayed_sample;
    
        bufferlock[self.write_index] = output;
        self.write_index = (self.write_index + 1) % buffer_len;
    
        drop(bufferlock); // Release the lock explicitly here
    
        output
    }
    
}