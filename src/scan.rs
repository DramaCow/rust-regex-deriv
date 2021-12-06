use std::ops::Range;
use super::LexTable;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token {
    pub class: usize,
    pub span: Range<usize>,
}

pub struct Scan<'a, S> {
    table: &'a S,
    input: &'a [u8],
    index: usize,
}

#[derive(Debug)]
pub struct ScanError {
    pos: usize,
}

impl<'a, S: LexTable> Scan<'a, S> {
    #[must_use]
    pub fn new<I: AsRef<[u8]> + ?Sized>(table: &'a S, input: &'a I) -> Self {
        Self {
            table,
            input: input.as_ref(),
            index: 0,
        }
    }
}

impl<'a, S: LexTable> Iterator for Scan<'a, S> {
    type Item = Result<Token, ScanError>;

    #[allow(clippy::option_if_let_else)]
    fn next(&mut self) -> Option<Self::Item> {       
        if self.index < self.input.as_ref().len() {
            let mut state = S::START_STATE;
            let mut index = self.index;
            
            let mut last_accept_state = self.table.sink();
            let mut last_accept_index = 0_usize;

            // simulate dfa until hit the sink state or end of input
            for byte in self.input[self.index..].iter().copied() {            
                if state == self.table.sink() {
                    break;
                }
                
                if self.table.class(state).is_some() {
                    last_accept_state = state;
                    last_accept_index = index;
                }
                
                state = self.table.step(state, byte);
                index += 1;
            }

            // currently on an accept state
            if let Some(class) = self.table.class(state) {
                let i = self.index;
                self.index = index;
                Some(Ok(Token { span: i..self.index, class }))
            // landed on an accept state in the past
            } else if let Some(class) = self.table.class(last_accept_state) {
                let i = self.index;
                self.index = last_accept_index;
                Some(Ok(Token { span: i..self.index, class }))
            // failed to match anything
            } else {
                let i = self.index;
                self.index = usize::MAX; // forces next iteration to return None
                Some(Err(ScanError { pos: i }))
            }
        } else {
            None
        }
    }
}