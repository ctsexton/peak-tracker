#[derive(Copy, Clone)]
pub struct Note {
    pub note_number: u8,
}

#[derive(Copy, Clone)]
pub struct Event {
    pub offset: f32,
    pub data: EventData,
}

#[derive(Copy, Clone)]
pub enum EventData {
    NoteOn { note_number: u8, velocity: u8 },
    NoteOff { note_number: u8 },
}

pub trait Voice {
    fn get_note(&self) -> &Option<Note>;

    fn set_note(&mut self, note: Option<Note>);

    fn render_block(&mut self, block: &mut [f32]);

    fn note_on(&mut self, note_number: u8, _velocity: u8) {
        self.set_note(Some(Note { note_number }));
    }

    fn note_off(&mut self) {
        self.set_note(None);
    }

    fn is_free(&self) -> bool {
        self.get_note().is_none()
    }

    fn matches_note(&self, note_number: u8) -> bool {
        if let Some(note) = &self.get_note() {
            note.note_number == note_number
        } else {
            false
        }
    }
}

pub trait Synth<V: Voice> {
    fn get_voices(&self) -> &[V];
    fn get_voices_mut(&mut self) -> &mut [V];

    fn allocate_note(&mut self, note_number: u8, velocity: u8) {
        if let Some(_voice) = self
            .get_voices_mut()
            .iter_mut()
            .find(|voice| voice.matches_note(note_number))
        {
            return;
        }
        if let Some(voice) = self
            .get_voices_mut()
            .iter_mut()
            .find(|voice| voice.is_free())
        {
            voice.note_on(note_number, velocity);
        }
    }

    fn deallocate_note(&mut self, note_number: u8) {
        for voice in self
            .get_voices_mut()
            .iter_mut()
            .filter(|voice| voice.matches_note(note_number))
        {
            voice.note_off();
        }
    }

    fn render_block(&mut self, output: &mut [f32], events: &[Event]) {
        let mut block_start = 0;
        for event in events.iter() {
            match event.data {
                EventData::NoteOn {
                    note_number,
                    velocity,
                } => {
                    self.allocate_note(note_number, velocity);
                }
                EventData::NoteOff { note_number } => {
                    self.deallocate_note(note_number);
                }
            }
            let block_end = event.offset as usize;
            let block = &mut output[block_start..block_end];
            for voice in self.get_voices_mut().iter_mut() {
                voice.render_block(block);
            }
            block_start = event.offset as usize;
        }
        let block_end = output.len();
        let block = &mut output[block_start..block_end];
        for voice in self.get_voices_mut().iter_mut() {
            voice.render_block(block);
        }
    }
}
