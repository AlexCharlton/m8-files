use std::fmt;

#[derive(PartialEq, Debug)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseError: {}", &self.0)
    }
}

impl std::error::Error for ParseError {}

/// Spefic result type for M8 song parsing
pub type M8Result<T> = std::result::Result<T, ParseError>;

pub struct Reader {
    buffer: Vec<u8>,
    position: usize,
}

#[allow(dead_code)]
impl Reader {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self { buffer, position: 0, }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn read(&mut self) -> u8 {
        let p: usize = self.position;
        let b = self.buffer[p];
        self.position += 1;
        b
    }

    pub fn read_bytes(&mut self, n: usize) -> &[u8] {
        let p: usize = self.position;
        let bs = &self.buffer[p..p + n];
        self.position += n;
        bs
    }

    pub fn read_bool(&mut self) -> bool {
        self.read() == 1
    }

    pub fn read_string(&mut self, n: usize) -> String {
        let b = self.read_bytes(n);
        let mut end = b.iter().position(|&x| x == 0 || x == 255).unwrap_or(n);

        while end > 0 {
            match std::str::from_utf8(&b[0..end]) {
                Ok(str) => return str.to_string(),
                Err(_) => end -= 1
            }
        }
        
        String::from("")
    }

    pub fn pos(&self) -> usize { self.position }

    pub fn set_pos(&mut self, n: usize) {
        self.position = n;
    }
}
