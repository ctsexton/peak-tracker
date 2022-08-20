use crate::reconstructor::Reconstructor;
use lv2::prelude::*;
use wmidi::*;

#[derive(FeatureCollection)]
pub struct Features<'a> {
    map: LV2Map<'a>,
}

#[derive(PortCollection)]
struct Ports {
    input: InputPort<Audio>,
    output: OutputPort<Audio>,
    freeze: InputPort<Control>,
    transpose: InputPort<Control>,
    detune: InputPort<Control>,
    synth_mode: InputPort<Control>,
    events_in: InputPort<AtomPort>,
}

#[derive(URIDCollection)]
struct URIDs {
    atom: AtomURIDCollection,
    midi: MidiURIDCollection,
    units: UnitURIDCollection,
}

#[uri("https://github.com/ctsexton/reconstructor-lv2")]
struct ReconstructorPlugin {
    reconstructor: Reconstructor,
    input: Vec<f32>,
    output: Vec<f32>,
    urids: URIDs,
}

impl Plugin for ReconstructorPlugin {
    type Ports = Ports;

    type InitFeatures = Features<'static>;
    type AudioFeatures = ();

    fn new(plugin_info: &PluginInfo, features: &mut Features<'static>) -> Option<Self> {
        let reconstructor = Reconstructor::new(plugin_info.sample_rate() as f32);
        let input = vec![0_f32; 2048];
        let output = vec![0_f32; 2048];
        Some(Self {
            reconstructor,
            input,
            output,
            urids: features.map.populate_collection()?,
        })
    }

    fn run(&mut self, ports: &mut Ports, features: &mut (), _: u32) {
        for in_copy in self.input.iter_mut() {
            *in_copy = 0.0;
        }
        for (in_frame, in_copy) in ports.input.iter().zip(self.input.iter_mut()) {
            *in_copy = *in_frame;
        }
        for out_copy in self.output.iter_mut() {
            *out_copy = 0.0;
        }
        for out_frame in ports.output.iter_mut() {
            *out_frame = 0.0;
        }

        let midi_sequence = ports
            .events_in
            .read(self.urids.atom.sequence, self.urids.units.beat)
            .unwrap();

        for (timestamp, message) in midi_sequence {
            let timestamp = timestamp.as_frames().unwrap();

            let message = if let Some(message) = message.read(self.urids.midi.wmidi, ()) {
                message
            } else {
                continue;
            };

            match message {
                MidiMessage::NoteOn(_, _, _) => {
                    println!("NOTE ON");
                }
                MidiMessage::NoteOff(_, _, _) => {
                    println!("NOTE OFF");
                }
                _ => (),
            }
        }

        let block_size = ports.input.len();
        let freeze_active = *ports.freeze > 0.0;
        self.reconstructor.set_freeze(freeze_active);
        self.reconstructor.set_transpose(*ports.transpose);
        self.reconstructor.set_detune(*ports.detune);
        self.reconstructor.set_synth_mode(*ports.synth_mode > 0.0);
        self.reconstructor
            .run(&self.input[0..block_size], &mut self.output[0..block_size]);
        for (out_frame, out_copy) in ports.output.iter_mut().zip(self.output.iter()) {
            *out_frame = *out_copy;
        }
    }
}

lv2_descriptors!(ReconstructorPlugin);
