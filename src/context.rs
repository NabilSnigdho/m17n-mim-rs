use std::collections::HashMap;

pub struct Context {
    pub preedit: Vec<char>,
    pub current_pos: usize,
    pub committed: String,
    pub variables: HashMap<String, i64>,
    pub current_state: String,
}

impl Context {
    pub fn new(initial_state: String) -> Self {
        Context {
            preedit: Vec::new(),
            current_pos: 0,
            committed: String::new(),
            variables: HashMap::new(),
            current_state: initial_state,
        }
    }

    pub fn insert(&mut self, text: &str) {
        for ch in text.chars() {
            self.preedit.insert(self.current_pos, ch);
            self.current_pos += 1;
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.preedit.insert(self.current_pos, ch);
        self.current_pos += 1;
    }

    pub fn delete(&mut self, pos: isize) {
        let target_pos = self.resolve_position(pos);
        if target_pos < self.current_pos {
            self.preedit.drain(target_pos..self.current_pos);
            self.current_pos = target_pos;
        } else if target_pos > self.current_pos {
            self.preedit.drain(self.current_pos..target_pos);
        }
    }

    pub fn move_pos(&mut self, pos: isize) {
        self.current_pos = self.resolve_position(pos);
    }

    pub fn resolve_position(&self, pos: isize) -> usize {
        match pos {
            0 => 0,                                              // @<
            -1 => self.current_pos.saturating_sub(1),            // @-
            1 => (self.current_pos + 1).min(self.preedit.len()), // @+
            isize::MAX => self.preedit.len(),                    // @>
            _ if pos < 0 => self.current_pos.saturating_sub((-pos) as usize),
            _ => (self.current_pos + pos as usize).min(self.preedit.len()),
        }
    }

    pub fn get_char_at(&self, pos: isize) -> i64 {
        if pos == 0 {
            return -1; // Surrounding text supported
        }

        let idx = if pos < 0 {
            let offset = (-pos) as usize;
            if offset <= self.current_pos {
                self.current_pos - offset
            } else {
                return 0;
            }
        } else {
            let offset = pos as usize - 1;
            self.current_pos + offset
        };

        if idx < self.preedit.len() {
            self.preedit[idx] as i64
        } else {
            0
        }
    }

    pub fn commit(&mut self) {
        self.committed
            .push_str(&self.preedit.iter().collect::<String>());
        self.preedit.clear();
        self.current_pos = 0;
    }

    pub fn set_var(&mut self, name: &str, value: i64) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get_var(&self, name: &str) -> i64 {
        *self.variables.get(name).unwrap_or(&0)
    }
}
