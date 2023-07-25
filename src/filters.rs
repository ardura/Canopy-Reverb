// Filters for rust plugins inspired by Airwindows iir filter format
// by Ardura

#[derive(Clone, Copy)]
pub(crate) struct StereoFilter {
    cutoff_frequency: f32,
    l_old: f32,
	r_old: f32,
	l_old2: f32,
	r_old2: f32,
	lowpass: bool,
}

impl StereoFilter {
    pub(crate) fn new(cutoff_frequency: f32, lowpass: bool) -> StereoFilter {
        StereoFilter {
            cutoff_frequency,
            l_old: 0.0,
			r_old: 0.0,
			l_old2: 0.0,
			r_old2: 0.0,
			lowpass,
        }
    }

	/// Update Cutoff frequency and filtertype
	pub(crate) fn update_params(&mut self, cutoff_frequency: f32, lowpass: bool) {
		self.lowpass = lowpass;
		self.cutoff_frequency = cutoff_frequency;
	}

	/// Perform filtering on left and right audio using the struct
    pub(crate) fn filter(&mut self, left: f32, right: f32) -> (f32,f32) {
		let mut l_filtered;
		let mut r_filtered;
		if self.lowpass {
			l_filtered = (self.l_old * (1.0 - self.cutoff_frequency)) + (left * self.cutoff_frequency);
			r_filtered = (self.r_old * (1.0 - self.cutoff_frequency)) + (right * self.cutoff_frequency);
			self.l_old = l_filtered;
        	self.r_old = r_filtered;
			l_filtered = (self.l_old2 * (1.0 - self.cutoff_frequency)) + (l_filtered * self.cutoff_frequency);
			r_filtered = (self.r_old2 * (1.0 - self.cutoff_frequency)) + (r_filtered * self.cutoff_frequency);
			self.l_old2 = l_filtered;
        	self.r_old2 = r_filtered;
		}
		else {
			l_filtered = (self.l_old * (1.0 - self.cutoff_frequency)) + (left * self.cutoff_frequency);
			r_filtered = (self.r_old * (1.0 - self.cutoff_frequency)) + (right * self.cutoff_frequency);
			self.l_old = l_filtered;
        	self.r_old = r_filtered;
			l_filtered = (self.l_old2 * (1.0 - self.cutoff_frequency)) + (l_filtered * self.cutoff_frequency);
			r_filtered = (self.r_old2 * (1.0 - self.cutoff_frequency)) + (r_filtered * self.cutoff_frequency);
			self.l_old2 = l_filtered;
        	self.r_old2 = r_filtered;
		}

		if self.lowpass {
			(l_filtered, r_filtered)
		}
		else {
			(left - self.l_old2 - self.l_old, right - self.r_old2 - self.r_old)
		}
    }
}
