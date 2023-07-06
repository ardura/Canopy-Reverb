#![allow(non_snake_case)]
mod ui_knob;
mod tapdelayline;
use nih_plug::{prelude::*};
use nih_plug_egui::{create_egui_editor, egui::{self, Color32, Rect, Rounding, RichText, FontId, Pos2}, EguiState};
use std::{sync::{Arc}, ops::RangeInclusive};

/***************************************************************************
 * Subhoofer v2 by Ardura
 * 
 * Build with: cargo xtask bundle subhoofer --profile <release or profiling>
 * *************************************************************************/

 // GUI Colors
const A_KNOB_OUTSIDE_COLOR: Color32 = Color32::from_rgb(112,141,129);
const A_BACKGROUND_COLOR: Color32 = Color32::from_rgb(0,20,39);
const A_KNOB_INSIDE_COLOR: Color32 = Color32::from_rgb(244,213,141);
const A_KNOB_OUTSIDE_COLOR2: Color32 = Color32::from_rgb(242,100,25);

// Plugin sizing
const WIDTH: u32 = 360;
const HEIGHT: u32 = 380;

pub struct Gain {
    params: Arc<GainParams>,
    tdl: tapdelayline::TappedDelayLine,
}

#[derive(Params)]
struct GainParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "reverb_size"]
    pub reverb_size: IntParam,

    #[id = "reverb_flutter"]
    pub reverb_flutter: FloatParam,

    #[id = "output_gain"]
    pub output_gain: FloatParam,

    #[id = "dry_wet"]
    pub dry_wet: FloatParam,
}

impl Default for Gain {
    fn default() -> Self {
        Self {
            params: Arc::new(GainParams::default()),
            tdl: tapdelayline::TappedDelayLine::new(3000, vec![2000,2500,1000,500]),
        }
    }
}

impl Default for GainParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(WIDTH, HEIGHT),

            reverb_size: IntParam::new(
                "Size",
                0,
                IntRange::Linear { min: 0, max: 200 },
            )
            .with_smoother(SmoothingStyle::Logarithmic(30.0)),

            reverb_flutter: FloatParam::new(
                "Flutter",
                0.5,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(30.0)),

            output_gain: FloatParam::new(
                "Output Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed { 
                    min: util::db_to_gain(-12.0), 
                    max: util::db_to_gain(12.0),
                    factor: FloatRange::gain_skew_factor(-12.0, 12.0) },
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" dB"),

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
    const NAME: &'static str = "Tapverb";
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
                            ui.label(RichText::new("    Tapverb").font(FontId::proportional(14.0)).color(A_KNOB_OUTSIDE_COLOR)).on_hover_text("by Ardura!");

                            ui.horizontal(|ui| {
                                let knob_size = 40.0;
                                ui.vertical(|ui| {
                                    let mut output_knob = ui_knob::ArcKnob::for_param(&params.output_gain, setter, knob_size);
                                    output_knob.preset_style(ui_knob::KnobStyle::SmallTogether);
                                    output_knob.set_fill_color(A_KNOB_OUTSIDE_COLOR2);
                                    output_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                    ui.add(output_knob);
                                
                                    let mut dry_wet_knob = ui_knob::ArcKnob::for_param(&params.dry_wet, setter, knob_size);
                                    dry_wet_knob.preset_style(ui_knob::KnobStyle::SmallTogether);
                                    dry_wet_knob.set_fill_color(A_KNOB_OUTSIDE_COLOR2);
                                    dry_wet_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                    ui.add(dry_wet_knob);
                                });

                                ui.vertical(|ui| {
                                    let mut size_knob = ui_knob::ArcKnob::for_param(&params.reverb_size, setter, knob_size + 24.0);
                                    size_knob.preset_style(ui_knob::KnobStyle::MediumThin);
                                    size_knob.set_fill_color(A_KNOB_INSIDE_COLOR);
                                    size_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                    ui.add(size_knob);

                                    let mut flutter_knob = ui_knob::ArcKnob::for_param(&params.reverb_flutter, setter, knob_size + 24.0);
                                    flutter_knob.preset_style(ui_knob::KnobStyle::LargeMedium);
                                    flutter_knob.set_fill_color(A_KNOB_INSIDE_COLOR);
                                    flutter_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                    ui.add(flutter_knob);
                                });
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
            let mut processed_sample_r: f32 = 0.0;

            let reverb_size: i32 = self.params.reverb_size.smoothed.next();
            let reverb_flutter: f32 = self.params.reverb_flutter.smoothed.next();
            let output_gain: f32 = self.params.output_gain.smoothed.next();
            let dry_wet: f32 = self.params.dry_wet.value();

            // Split left and right same way original subhoofer did
            let in_l = *channel_samples.get_mut(0).unwrap();
            let in_r = *channel_samples.get_mut(1).unwrap();
            
            ///////////////////////////////////////////////////////////////////////
            
            // Process Audio
            processed_sample_l = self.tdl.process(in_l, reverb_flutter, reverb_size, output_gain);
            processed_sample_r = 0.0;

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
    const CLAP_ID: &'static str = "com.ardura.tapverb";
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
    const VST3_CLASS_ID: [u8; 16] = *b"TapverbArduraAAA";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Reverb];
}

nih_export_clap!(Gain);
nih_export_vst3!(Gain);
