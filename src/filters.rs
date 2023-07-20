use std::f32::consts::PI;

pub(crate) struct LowpassFilter {
    cutoff_frequency: f32,
    num_recursions: usize,

    lowpass_samples: Vec<f32>,
}

impl LowpassFilter {
    pub fn new(cutoff_frequency: f32, num_recursions: usize) -> Self {
        Self {
            cutoff_frequency: 2.0 * PI * cutoff_frequency,
            num_recursions,
            lowpass_samples: vec![0.0; num_recursions],
        }
    }

    pub fn update_cutoff(&mut self, new_frequency: f32) {
        self.cutoff_frequency = new_frequency;
    }

    pub fn apply_filter(&mut self, sample: f32) -> f32 {
        let mut output = 0.0;

        for i in 0..self.num_recursions {
            output += self.lowpass_samples[i] * (1.0 - self.cutoff_frequency);
        }

        output += sample * self.cutoff_frequency;

        self.lowpass_samples.push(output);
        self.lowpass_samples.remove(0);

        output
    }
}
