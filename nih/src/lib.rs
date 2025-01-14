use nih_plug::prelude::*;
use std::sync::Arc;
use core::reconstructor::Reconstructor;
use core::voice::{Event, EventData};

struct PeakTracker {
    params: Arc<PeakTrackerParams>,
    reconstructor: Option<Reconstructor>,
    input: Vec<f32>,
    output: Vec<f32>,
    events: Vec<Event>,
}

#[derive(Params)]
struct PeakTrackerParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "freeze"]
    pub freeze: BoolParam,
    #[id = "transpose"]
    pub transpose: FloatParam,
    #[id = "detune"]
    pub detune: FloatParam,
    #[id = "synth_mode"]
    pub synth_mode: BoolParam,
}

impl Default for PeakTracker {
    fn default() -> Self {
        Self {
            params: Arc::new(PeakTrackerParams::default()),
            reconstructor: None,
            input: vec![0_f32; 4096],
            output: vec![0_f32; 4096],
            events: Vec::<Event>::with_capacity(256),
        }
    }
}

impl Default for PeakTrackerParams {
    fn default() -> Self {
        Self {
            freeze: BoolParam::new(
                "Freeze",
                false,
            ),
            transpose: FloatParam::new(
                "Transpose",
                0.0,
                FloatRange::Linear {
                    min: -2.0,
                    max: 2.0,
                },
            ),
            detune: FloatParam::new(
                "Detune",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            ),
            synth_mode: BoolParam::new(
                "Synth Mode",
                false,
            ),
        }
    }
}

impl Plugin for PeakTracker {
    const NAME: &'static str = "Peak Tracker";
    const VENDOR: &'static str = "Cam Sexton";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "cameron.t.sexton@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(1),
        main_output_channels: NonZeroU32::new(1),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];


    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.reconstructor = Some(Reconstructor::new(buffer_config.sample_rate));
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for in_copy in self.input.iter_mut() {
            *in_copy = 0.0;
        }
        let input_channel = buffer.as_slice().first().unwrap();
        for (sample, in_copy) in input_channel.iter().zip(self.input.iter_mut()) {
            *in_copy = *sample;
        }
        for out_copy in self.output.iter_mut() {
            *out_copy = 0.0;
        }
        for channel_samples in buffer.iter_samples() {
            for sample in channel_samples {
                *sample = 0_f32;
            }
        }
        self.events.clear();

        while let Some(event) = context.next_event() {
            if self.events.len() == self.events.capacity() {
                break;
            }
            let timestamp = event.timing();

            match event {
                NoteEvent::NoteOn{ note, .. } => {
                    let event = Event {
                        offset: timestamp as f32,
                        data: EventData::NoteOn {
                            note_number: note,
                            velocity: 127,
                        },
                    };
                    self.events.push(event);
                }
                NoteEvent::NoteOff{ note, .. } => {
                    let event = Event {
                        offset: timestamp as f32,
                        data: EventData::NoteOff {
                            note_number: note,
                        },
                    };
                    self.events.push(event);
                }
                _ => (),
            }
        }

        let mut reconstructor = self.reconstructor.take().unwrap();
        reconstructor.set_freeze(self.params.freeze.value());
        reconstructor.set_transpose(self.params.transpose.value());
        reconstructor.set_detune(self.params.detune.value());
        reconstructor.set_synth_mode(self.params.synth_mode.value());
        reconstructor.run(
            &self.input[0..buffer.samples()],
            &mut self.output[0..buffer.samples()],
            self.events.as_slice(),
        );

        self.reconstructor = Some(reconstructor);

        let output_channel = buffer.as_slice().get_mut(0).unwrap();
        for (out, out_copy) in output_channel.iter_mut().zip(self.output.iter()) {
            *out = *out_copy;
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for PeakTracker {
    const CLAP_ID: &'static str = "com.your-domain.peak-tracker";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Audio reconstructor using peak tracking");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for PeakTracker {
    const VST3_CLASS_ID: [u8; 16] = *b"ctsextonpeaktrak";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(PeakTracker);
nih_export_vst3!(PeakTracker);
