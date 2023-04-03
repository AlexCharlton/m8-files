use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(PartialEq, Debug)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseError: {}", &self.0)
    }
}

impl std::error::Error for ParseError {}

pub type Result<T> = std::result::Result<T, ParseError>;

pub struct Reader {
    buffer: Vec<u8>,
    position: Rc<RefCell<usize>>,
}

#[allow(dead_code)]
impl Reader {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self {
            buffer,
            position: Rc::new(RefCell::new(0)),
        }
    }

    pub fn read(&self) -> u8 {
        let p: usize = *self.position.borrow();
        let b = self.buffer[p];
        *self.position.borrow_mut() += 1;
        b
    }

    pub fn read_bytes(&self, n: usize) -> &[u8] {
        let p: usize = *self.position.borrow();
        let bs = &self.buffer[p..p + n];
        *self.position.borrow_mut() += n;
        bs
    }

    pub fn read_bool(&self) -> bool {
        self.read() == 1
    }

    pub fn read_string(&self, n: usize) -> String {
        let b = self.read_bytes(n);
        let end = b.iter().position(|&x| x == 0 || x == 255).unwrap_or(0);
        std::str::from_utf8(&b[0..end])
            .expect("invalid utf-8 sequence in string")
            .to_string()
    }

    pub fn pos(&self) -> usize {
        *self.position.borrow()
    }

    pub fn set_pos(&self, n: usize) {
        *self.position.borrow_mut() = n;
    }
}
