pub struct Ducking {
    trigger_threshold: f32,
    attack_time: f32,
    release_time: f32,
    attack_coeff: f32,
    release_coeff: f32,
    gain_reduction: f32,
    envelope: f32,
    sample_rate: f32,
    smoothing_coeff: f32,
    smoothed_gain_reduction: f32,
}

impl Ducking {
    // Not really sure how to get around the sample rate here on creation
    pub fn new(trigger_threshold: f32, attack_time: f32, release_time: f32, sample_rate_new: f32) -> Self {
        Ducking {
            trigger_threshold,
            attack_time,
            release_time,
            gain_reduction: 1.0,
            envelope: 0.0,
            attack_coeff: (-1.0 / (attack_time * sample_rate_new)).exp(),
            release_coeff: (-1.0 / (release_time * sample_rate_new)).exp(),
            sample_rate: sample_rate_new,
            smoothing_coeff: 0.9999, // Adjust this value to control the smoothing amount
            smoothed_gain_reduction: 1.0,
        }
    }

    // I couldn't get these to work so removed them for now
    #[allow(dead_code)]
    pub fn update_attack(&mut self, new_attack: f32, sample_rate_new: f32) {
        self.sample_rate = sample_rate_new;
        self.attack_time = new_attack;
        self.attack_coeff = (-1.0 / (self.attack_time*1000.0 * self.sample_rate)).exp();
    }

    // I couldn't get these to work so removed them for now
    #[allow(dead_code)]
    pub fn update_release(&mut self, new_release: f32, sample_rate_new: f32) {
        self.sample_rate = sample_rate_new;
        self.release_time = new_release;
        self.release_coeff = (-1.0 / (self.release_time*1000.0 * self.sample_rate)).exp();
    }

    pub fn update_threshold(&mut self, new_threshold: f32, sample_rate_new: f32) {
        self.sample_rate = sample_rate_new;
        self.trigger_threshold = new_threshold;
    }

    pub fn process(&mut self, input: f32, trigger: f32) -> f32 {
        // Calculate the envelope based on the trigger signal - threshold is in db already
        let target_envelope = if 20.0*trigger.log10() > self.trigger_threshold {
            1.0
        } else {
            0.0
        };

        // Update the envelope
        self.envelope = if target_envelope > self.envelope {
            self.envelope * (1.0 - self.attack_coeff) + target_envelope * self.attack_coeff
        } else {
            self.envelope * (1.0 - self.release_coeff) + target_envelope * self.release_coeff
        };

        // Calculate gain reduction based on the envelope
        self.gain_reduction = 1.0 - self.envelope;

        // Smooth out the gain reduction using a low-pass filter
        self.smoothed_gain_reduction = self.smoothed_gain_reduction * self.smoothing_coeff + self.gain_reduction * (1.0 - self.smoothing_coeff);

        // Apply smoothed gain reduction to the input signal
        input * self.smoothed_gain_reduction
    }
}