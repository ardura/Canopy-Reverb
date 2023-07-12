use std::sync::Arc;
use nih_plug_egui::egui::mutex::Mutex;

#[derive(Clone)]

pub(crate) struct Reverb {
    delay_time: Arc<Mutex<i32>>,
    decay: f32,
    buffer: Arc<Mutex<Vec<f32>>>,
    write_index: Arc<Mutex<usize>>,
    buf_changed: bool,
}

impl Reverb {
    pub(crate) fn new(delay_time: i32, decay: f32, buffer_size: usize) -> Self {
        Reverb {
            delay_time: Arc::new(Mutex::new(delay_time)),
            decay,
            buffer: Arc::new(Mutex::new(vec![0.0; buffer_size])),
            write_index: Arc::new(Mutex::new(0)),
            buf_changed: false,
        }
    }

    pub fn update(&mut self, delay_time: i32, decay: f32) {
        let mut buffer_lock = self.buffer.lock();
        let mut delay_time_lock = *self.delay_time.lock();
        if delay_time_lock != delay_time {
            delay_time_lock = delay_time;
            self.buf_changed = true;
        }
        self.decay = decay;

        if self.buf_changed {
            self.buf_changed = false;
            let new_buffer_len = delay_time_lock as usize * 2;
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
        drop(delay_time_lock);
    }

    pub(crate) fn display(&mut self) -> String
    {
        return format!("Delay: {} , Decay: {} , Buffer: {}" , *self.delay_time.lock() , self.decay ,self.buffer.lock().len());
    }

    pub(crate) fn process(&mut self, input: f32) -> f32 {
        let delaylock = *self.delay_time.lock();
        let mut bufferlock = self.buffer.lock();
        let buffer_len = bufferlock.len();        
        let mut write_index = *self.write_index.lock();
        let read_index = (write_index + delaylock as usize) % buffer_len;
        let delayed_sample = bufferlock[read_index] * self.decay;
        let output = input + delayed_sample;

        if write_index >= buffer_len {
            write_index = 0;
        }
        bufferlock[write_index] = output;
        *self.write_index.lock() = write_index + 1;

        drop(bufferlock);
        drop(delaylock);

        output
    }
}

struct AllpassFilter {
    delay_line: Vec<f32>,
    delay_length: usize,
    gain: f32,
    index: usize,
}

impl AllpassFilter {
    fn new(delay_length: usize, gain: f32) -> AllpassFilter {
        AllpassFilter {
            delay_line: vec![0.0; delay_length],
            delay_length,
            gain,
            index: 0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed_sample = self.delay_line[self.index];
        let output = -input + delayed_sample * self.gain;
        self.delay_line[self.index] = input + self.delay_line[self.index] * self.gain; // Use delayed_sample from the previous line
        self.index = (self.index + 1) % self.delay_length;
        output
    }
    
}