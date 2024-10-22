
pub struct Writer {
    buffer: Vec<u8>,
    pos: usize
}

impl Writer {
    /// Initialize the writer from a loaded song
    pub fn new(v: Vec<u8>) -> Writer {
        Writer { buffer: v, pos: 0 }
    }

    /// Terminate writing and return the buffer
    pub fn finish(self) -> Vec<u8> {
        self.buffer
    }

    pub fn write(&mut self, v: u8) {
        self.buffer[self.pos] = v;
        self.pos += 1;
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        let mut cursor = self.pos;
        let buff = &mut self.buffer;

        for b in bytes {
            buff[cursor] = *b;
            cursor += 1;
        }

        self.pos = cursor;
    }

    pub fn write_string(&mut self, str: &str, fill: usize) {
        let bytes = str.as_bytes();
        self.write_bytes(bytes);
        self.fill_till(0, fill - bytes.len());
    }

    pub fn skip(&mut self, skip: usize) {
        self.pos += skip
    }

    pub fn seek(&mut self, new_pos: usize) {
        self.pos = new_pos;
    }

    pub fn pos(&self) -> usize { self.pos }

    fn fill_till(&mut self, v: u8, until : usize) {
        if until == 0 { return }

        for _i in 0 .. until {
            self.buffer[self.pos] = v;
            self.pos += 1;
        }
    }
}
