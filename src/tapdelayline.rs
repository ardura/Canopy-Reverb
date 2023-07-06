pub struct TappedDelayLine {
    buffer: Vec<f32>,
    tap_delays: Vec<usize>,
    next_sample_index: usize,
}

impl TappedDelayLine {
    pub(crate) fn new(max_delay: usize, tap_delays: Vec<usize>) -> Self {
        let buffer_size = max_delay + 1;
        let buffer = vec![0.0; buffer_size];
        let next_sample_index = 0;

        Self {
            buffer,
            tap_delays,
            next_sample_index,
        }
    }

    pub(crate) fn process(&mut self, input: f32, reverb_flutter: f32, reverb_size: i32, reverb_gain: f32) -> f32 {
        let mut output = self.buffer[self.next_sample_index];

        for (i, delay) in self.tap_delays.iter().enumerate() {
            let tap_index = (self.next_sample_index + delay) % self.buffer.len();
            //output += self.buffer[tap_index] * (i as f32 + 1.0); // Modify the tap output (optional)
            //output += self.buffer[tap_index] * 0.5 * (reverb_flutter * tap_index as f32).sin();
            output += self.buffer[tap_index] * reverb_gain;
        }

        self.buffer[self.next_sample_index] = input;
        self.next_sample_index = (self.next_sample_index + 1) % self.buffer.len();

        output
    }
}
