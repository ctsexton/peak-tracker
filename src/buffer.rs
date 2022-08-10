pub struct Ringbuffer {
    data: Vec<f32>,
    index: usize,
}

impl Ringbuffer {
    pub fn new(length: usize) -> Self {
        let data = vec![0_f32; length];
        Self { data, index: 0 }
    }

    pub fn write(&mut self, value: f32) {
        self.data[self.index] = value;
        self.index += 1 % self.data.len();
    }

    pub fn get_reader(&self) -> BufferReader {
        BufferReader {
            data: &self.data,
            starting_index: self.index,
            index: 0,
        }
    }
}

pub struct BufferReader<'a> {
    data: &'a Vec<f32>,
    starting_index: usize,
    index: usize,
}

impl Iterator for BufferReader<'_> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.data.len() {
            return None;
        }
        let buffer_index = (self.index + self.starting_index) % self.data.len();
        let value = self.data[buffer_index];
        self.index += 1;
        Some(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ringbuffer() {
        let mut buffer = Ringbuffer::new(4);
        buffer.write(1.0);
        buffer.write(2.0);
        let mut reader = buffer.get_reader();
        assert!((reader.next().unwrap() - 0.0).abs() < f32::EPSILON);
        assert!((reader.next().unwrap() - 0.0).abs() < f32::EPSILON);
        assert!((reader.next().unwrap() - 1.0).abs() < f32::EPSILON);
        assert!((reader.next().unwrap() - 2.0).abs() < f32::EPSILON);
        assert!(reader.next().is_none());
    }
}
