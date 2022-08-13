use crate::reconstructor::Reconstructor;
use lv2::prelude::*;

#[derive(PortCollection)]
struct Ports {
    input: InputPort<Audio>,
    output: OutputPort<Audio>,
}

#[uri("https://github.com/ctsexton/reconstructor-lv2")]
struct ReconstructorPlugin {
    reconstructor: Reconstructor,
    input: Vec<f32>,
    output: Vec<f32>,
}

impl Plugin for ReconstructorPlugin {
    type Ports = Ports;

    type InitFeatures = ();
    type AudioFeatures = ();

    fn new(_plugin_info: &PluginInfo, _features: &mut ()) -> Option<Self> {
        let reconstructor = Reconstructor::new();
        let input = vec![0_f32; 512];
        let output = vec![0_f32; 512];
        Some(Self {
            reconstructor,
            input,
            output,
        })
    }

    fn run(&mut self, ports: &mut Ports, _features: &mut (), _: u32) {
        for (in_frame, in_copy) in ports.input.iter().zip(self.input.iter_mut()) {
            *in_copy = *in_frame;
        }
        for out_copy in self.output.iter_mut() {
            *out_copy = 0.0;
        }
        for out_frame in ports.output.iter_mut() {
            *out_frame = 0.0;
        }
        self.reconstructor
            .run(self.input.as_slice(), self.output.as_mut_slice());
        for (out_frame, out_copy) in ports.output.iter_mut().zip(self.output.iter()) {
            *out_frame = *out_copy;
        }
    }
}

lv2_descriptors!(ReconstructorPlugin);
