#![allow(non_snake_case)]
mod ui_knob;
mod reverb;
use nih_plug::{prelude::*};
use nih_plug_egui::{create_egui_editor, egui::{self, Color32, Rect, Rounding, RichText, FontId, Pos2}, EguiState};
use reverb::{Reverb, ReverbType};
use std::{sync::{Arc}, ops::RangeInclusive};

/***************************************************************************
 * Canopy Reverb by Ardura
 * 
 * Build with: cargo xtask bundle CanopyReverb --profile <release or profiling>
 * *************************************************************************/

 // GUI Colors
const A_KNOB_OUTSIDE_COLOR: Color32 = Color32::from_rgb(255, 235, 59);
const A_BACKGROUND_COLOR: Color32 = Color32::from_rgb(0, 123, 94);
const A_KNOB_INSIDE_COLOR: Color32 = Color32::from_rgb(233, 109, 46);
const A_KNOB_OUTSIDE_COLOR2: Color32 = Color32::from_rgb(0, 74, 76);


// Plugin sizing
const WIDTH: u32 = 820;
const HEIGHT: u32 = 136;

pub struct Gain {
    params: Arc<GainParams>,
    reverb_l_array: Vec<reverb::Reverb>,
    reverb_r_array: Vec<reverb::Reverb>,
    prev_reverb_steps: i32,
    prev_reverb_alg: ReverbType,
    prev_reverb_delay: i32,
    prev_reverb_decay: f32,
    prev_processed_in_l: f32,
    prev_processed_in_r: f32,
    prev_processed_out_l: f32,
    prev_processed_out_r: f32,
}

#[derive(Params)]
struct GainParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "reverb_stack"]
    pub reverb_stack: IntParam,

    #[id = "reverb_delay"]
    pub reverb_delay: IntParam,

    #[id = "reverb_decay"]
    pub reverb_decay: FloatParam,

    #[id = "reverb_steps"]
    pub reverb_steps: IntParam,

    #[id = "reverb_step_alg"]
    pub reverb_step_alg: EnumParam<reverb::ReverbType>,

    #[id = "reverb_width"]
    pub reverb_width: FloatParam,

    #[id = "output_gain"]
    pub output_gain: FloatParam,

    #[id = "dry_wet"]
    pub dry_wet: FloatParam,
}

impl Default for Gain {
    fn default() -> Self {
        Self {
            params: Arc::new(GainParams::default()),
            reverb_l_array: (0..1).map(|_| reverb::Reverb::new(vec![0,0],0.6,400).clone()).collect(),
            reverb_r_array: (0..1).map(|_| reverb::Reverb::new(vec![0,0],0.6,400).clone()).collect(),
            prev_reverb_steps: 0,
            prev_reverb_alg: ReverbType::ExpSwirl,
            prev_reverb_delay: 0,
            prev_reverb_decay: 0.0,
            prev_processed_in_l: 0.0,
            prev_processed_in_r: 0.0,
            prev_processed_out_l: 0.0,
            prev_processed_out_r: 0.0,
        }
    }
}

impl Default for GainParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(WIDTH, HEIGHT),

            reverb_stack: IntParam::new(
                "Stack",
                4,
                IntRange::Linear { min: 1, max: 12 },
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Stack"),

            reverb_delay: IntParam::new(
                "Reverb Delay",
                954,
                IntRange::Linear { min: 100, max: 1200 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" ms Delay"),

            reverb_decay: FloatParam::new(
                "Reverb Decay",
                0.437,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 0.999,
                    factor: 0.7,
                },
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_value_to_string(formatters::v2s_f32_rounded(3))
            .with_unit(" Decay"),

            reverb_width: FloatParam::new(
                "Reverb Width",
                0.83,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_value_to_string(formatters::v2s_f32_rounded(2))
            .with_unit(" Width"),

            reverb_steps: IntParam::new(
                "Reverb Steps",
                10,
                IntRange::Linear {
                    min: 2,
                    max: 36,
                },
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Steps"),

            reverb_step_alg: EnumParam::new("Step Alg",reverb::ReverbType::ExpSwirl),

            // Output gain parameter
            output_gain: FloatParam::new(
                "Output Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-12.0),
                    max: util::db_to_gain(12.0),
                    factor: FloatRange::gain_skew_factor(-12.0, 12.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB Out Gain")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            // Dry/Wet parameter
            dry_wet: FloatParam::new(
                "Dry/Wet",
                1.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit("% Wet")
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}

impl Plugin for Gain {
    const NAME: &'static str = "Canopy Reverb";
    const VENDOR: &'static str = "Ardura";
    const URL: &'static str = "https://github.com/ardura";
    const EMAIL: &'static str = "azviscarra@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // This looks like it's flexible for running the plugin in mono or stereo
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {main_input_channels: NonZeroU32::new(2), main_output_channels: NonZeroU32::new(2), ..AudioIOLayout::const_default()},
        AudioIOLayout {main_input_channels: NonZeroU32::new(1), main_output_channels: NonZeroU32::new(1), ..AudioIOLayout::const_default()},
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();

        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default()
                    .show(egui_ctx, |ui| {
                        // Change colors - there's probably a better way to do this
                        let style_var = ui.style_mut().clone();

                        // Trying to draw background as rect
                        ui.painter().rect_filled(
                            Rect::from_x_y_ranges(
                                RangeInclusive::new(0.0, WIDTH as f32), 
                                RangeInclusive::new(0.0, HEIGHT as f32)), 
                            Rounding::from(16.0), A_BACKGROUND_COLOR);

                        // Screws for that vintage look
                        let screw_space = 10.0;
                        ui.painter().circle_filled(Pos2::new(screw_space,screw_space), 4.0, Color32::DARK_GRAY);
                        ui.painter().circle_filled(Pos2::new(screw_space,HEIGHT as f32 - screw_space), 4.0, Color32::DARK_GRAY);
                        ui.painter().circle_filled(Pos2::new(WIDTH as f32 - screw_space,screw_space), 4.0, Color32::DARK_GRAY);
                        ui.painter().circle_filled(Pos2::new(WIDTH as f32 - screw_space,HEIGHT as f32 - screw_space), 4.0, Color32::DARK_GRAY);

                        ui.set_style(style_var);

                        // GUI Structure
                        ui.vertical(|ui| {
                            // Spacing :)
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("    Canopy Reverb").font(FontId::monospace(14.0)).color(A_KNOB_OUTSIDE_COLOR2)).on_hover_text("by Ardura!");
                            });

                            ui.horizontal(|ui| {
                                let knob_size = 36.0;

                                let mut delay_knob = ui_knob::ArcKnob::for_param(&params.reverb_delay, setter, knob_size + 8.0);
                                delay_knob.preset_style(ui_knob::KnobStyle::LargeMedium);
                                delay_knob.set_fill_color(A_KNOB_INSIDE_COLOR);
                                delay_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                ui.add(delay_knob);

                                let mut stack_knob = ui_knob::ArcKnob::for_param(&params.reverb_stack, setter, knob_size + 8.0);
                                stack_knob.preset_style(ui_knob::KnobStyle::LargeMedium);
                                stack_knob.set_fill_color(A_KNOB_INSIDE_COLOR);
                                stack_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                ui.add(stack_knob);

                                let mut alg_knob = ui_knob::ArcKnob::for_param(&params.reverb_step_alg, setter, knob_size);
                                alg_knob.preset_style(ui_knob::KnobStyle::SmallTogether);
                                alg_knob.set_fill_color(A_KNOB_OUTSIDE_COLOR2);
                                alg_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                ui.add(alg_knob);

                                let mut decay_knob = ui_knob::ArcKnob::for_param(&params.reverb_decay, setter, knob_size);
                                decay_knob.preset_style(ui_knob::KnobStyle::LargeMedium);
                                decay_knob.set_fill_color(A_KNOB_INSIDE_COLOR);
                                decay_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                ui.add(decay_knob);

                                let mut step_knob = ui_knob::ArcKnob::for_param(&params.reverb_steps, setter, knob_size);
                                step_knob.preset_style(ui_knob::KnobStyle::LargeMedium);
                                step_knob.set_fill_color(A_KNOB_INSIDE_COLOR);
                                step_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                ui.add(step_knob);

                                let mut width_knob = ui_knob::ArcKnob::for_param(&params.reverb_width, setter, knob_size);
                                width_knob.preset_style(ui_knob::KnobStyle::LargeMedium);
                                width_knob.set_fill_color(A_KNOB_INSIDE_COLOR);
                                width_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                ui.add(width_knob);

                                let mut dry_wet_knob = ui_knob::ArcKnob::for_param(&params.dry_wet, setter, knob_size);
                                dry_wet_knob.preset_style(ui_knob::KnobStyle::SmallTogether);
                                dry_wet_knob.set_fill_color(A_KNOB_OUTSIDE_COLOR2);
                                dry_wet_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                ui.add(dry_wet_knob);

                                let mut output_knob = ui_knob::ArcKnob::for_param(&params.output_gain, setter, knob_size);
                                output_knob.preset_style(ui_knob::KnobStyle::SmallTogether);
                                output_knob.set_fill_color(A_KNOB_OUTSIDE_COLOR2);
                                output_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                ui.add(output_knob);
                            });
                        });
                    });
                }
            )
    }

    

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for mut channel_samples in buffer.iter_samples() {
            let mut processed_sample_l: f32;
            let mut processed_sample_r: f32;

            let reverb_stack: i32 = self.params.reverb_stack.smoothed.next();
            let reverb_delay: i32 = self.params.reverb_delay.smoothed.next();
            let reverb_decay: f32 = self.params.reverb_decay.smoothed.next();
            let reverb_width: f32 = self.params.reverb_width.smoothed.next();
            let reverb_steps: i32 = self.params.reverb_steps.smoothed.next();
            let reverb_step_alg: reverb::ReverbType = self.params.reverb_step_alg.value();
            let output_gain: f32 = self.params.output_gain.smoothed.next();
            let dry_wet: f32 = self.params.dry_wet.value();

            // Split left and right same way original subhoofer did
            let in_l = *channel_samples.get_mut(0).unwrap();
            let in_r = *channel_samples.get_mut(1).unwrap();
            
            ///////////////////////////////////////////////////////////////////////
            
            let temp_l_len: i32 = self.reverb_l_array.len() as i32;
            let temp_r_len: i32 = self.reverb_r_array.len() as i32;
            let temp_buffer: usize = reverb_delay as usize * 2 as usize;
            
            let mut update_bool = false;
            // Create or remove reverb stacks
            if reverb_stack > temp_l_len
            {
                while reverb_stack > self.reverb_l_array.len() as i32
                {
                    self.reverb_l_array.push(Reverb::new(Reverb::generate_steps(reverb_delay, reverb_steps, reverb_step_alg),reverb_decay,temp_buffer));
                }
                update_bool = true;
            }
            else if reverb_stack < temp_l_len
            {
                while reverb_stack < self.reverb_l_array.len() as i32
                {
                    self.reverb_l_array.pop();
                }
                update_bool = true;
            }
            if reverb_stack > temp_r_len
            {
                while reverb_stack > self.reverb_r_array.len() as i32
                {
                    self.reverb_r_array.push(Reverb::new(Reverb::generate_steps(reverb_delay, reverb_steps, reverb_step_alg),reverb_decay,temp_buffer));
                }
                update_bool = true;
            }
            if reverb_stack < temp_r_len
            {
                while reverb_stack < self.reverb_r_array.len() as i32
                {
                    self.reverb_r_array.pop();
                }
                update_bool = true;
            }
            if reverb_steps != self.prev_reverb_steps || reverb_step_alg != self.prev_reverb_alg || reverb_delay != self.prev_reverb_delay  || reverb_decay != self.prev_reverb_decay
            {
                update_bool = true;
                self.prev_reverb_alg = reverb_step_alg;
                self.prev_reverb_steps = reverb_steps;
                self.prev_reverb_delay = reverb_delay;
                self.prev_reverb_decay = reverb_decay;
            }

            if update_bool == true
            {
                let mut counter: i32 = 1;
                // Update our reverb stacks
                for (left, right) in 
                    self.reverb_l_array.iter_mut().zip(
                    self.reverb_r_array.iter_mut()) {
                    // Integer division to scale delay with amount of stack
                    left.update(Reverb::generate_steps(reverb_delay/counter, reverb_steps, reverb_step_alg), reverb_decay);
                    right.update(Reverb::generate_steps(reverb_delay/counter, reverb_steps, reverb_step_alg), reverb_decay);
                    counter += 1;
                }
            }

            // Set initial
            processed_sample_l = in_l;
            processed_sample_r = in_r;

            // Process our stacks
            for (left, right) in 
                self.reverb_l_array.iter_mut().zip(
                self.reverb_r_array.iter_mut()) {
                processed_sample_l += left.process(processed_sample_l);
                processed_sample_r += right.process(processed_sample_r);
            }

            // Reverb width control
            let widthInv = 1.0 - reverb_width;
            let mid = (processed_sample_l + processed_sample_r)*0.5;
            processed_sample_l  = widthInv * mid + reverb_width * processed_sample_l;
            processed_sample_r = widthInv * mid + reverb_width * processed_sample_r;

            // Calculated below by Ardura in advance!
            // double sqrt2 = 1.41421356237;
            // double corner_frequency = 5.0 / sqrt2;
            // double hp_gain = 1 / sqrt(1 + (5.0 / (corner_frequency)) ^ 2);
            // Remove DC Offset with single pole HP
            let hp_b0: f32 = 1.0;
            let hp_b1: f32 = -1.0;
            let hp_a1: f32 = -0.995;
            let hp_gain = 1.0;
        
            // Apply the 1 pole HP to left side
            processed_sample_l = hp_gain * processed_sample_l;
            let temp_sample: f32 = hp_b0 * processed_sample_l + hp_b1 * self.prev_processed_in_l - hp_a1 * self.prev_processed_out_l;
            self.prev_processed_in_l = processed_sample_l;
            self.prev_processed_out_l = temp_sample;
            processed_sample_l = temp_sample;

            // Apply the 1 pole HP to right side
            processed_sample_r = hp_gain * processed_sample_r;
            let temp_sample: f32 = hp_b0 * processed_sample_r + hp_b1 * self.prev_processed_in_r - hp_a1 * self.prev_processed_out_r;
            self.prev_processed_in_r = processed_sample_r;
            self.prev_processed_out_r = temp_sample;
            processed_sample_r = temp_sample;
                        
            ///////////////////////////////////////////////////////////////////////

            // Calculate dry/wet mix
            let wet_gain: f32 = dry_wet;
            processed_sample_l = in_l + processed_sample_l * wet_gain;
            processed_sample_r = in_r + processed_sample_r * wet_gain;
            
            // Output gain
            processed_sample_l *= output_gain;
            processed_sample_r *= output_gain;

            // Assign back so we can output our processed sounds
            *channel_samples.get_mut(0).unwrap() = processed_sample_l;
            *channel_samples.get_mut(1).unwrap() = processed_sample_r;
        }

        ProcessStatus::Normal
    }

    const MIDI_INPUT: MidiConfig = MidiConfig::None;

    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const HARD_REALTIME_ONLY: bool = false;

    fn task_executor(&mut self) -> TaskExecutor<Self> {
        // In the default implementation we can simply ignore the value
        Box::new(|_| ())
    }

    fn filter_state(_state: &mut PluginState) {}

    fn reset(&mut self) {}

    fn deactivate(&mut self) {}
}

impl ClapPlugin for Gain {
    const CLAP_ID: &'static str = "com.ardura.canopyreverb";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Reverb Experiment");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for Gain {
    const VST3_CLASS_ID: [u8; 16] = *b"CanopyReverbArda";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Reverb];
}

nih_export_clap!(Gain);
nih_export_vst3!(Gain);
