use std::collections::HashMap;
use crate::m17n_input_method::*;

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

    pub fn delete(&mut self, pos: &Pos) {
        let target = self.resolve_pos(pos);
        if target < self.current_pos {
            self.preedit.drain(target..self.current_pos);
            self.current_pos = target;
        } else if target > self.current_pos {
            self.preedit.drain(self.current_pos..target);
        }
    }

    pub fn move_cursor(&mut self, pos: &Pos) {
        self.current_pos = self.resolve_pos(pos);
    }

    fn resolve_pos(&self, pos: &Pos) -> usize {
        match pos {
            Pos::Beginning => 0,
            Pos::End => self.preedit.len(),
            Pos::Relative(n) => {
                let n = *n as isize;
                if n < 0 {
                    self.current_pos.saturating_sub((-n) as usize)
                } else {
                    (self.current_pos + n as usize).min(self.preedit.len())
                }
            }
        }
    }

    pub fn get_char_at(&self, pos: i32) -> i64 {
        if pos == 0 {
            return -1; // surrounding text not supported
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
        self.committed.push_str(&self.preedit.iter().collect::<String>());
        self.preedit.clear();
        self.current_pos = 0;
    }

    pub fn set_var(&mut self, name: &str, value: i64) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get_var(&self, name: &str) -> i64 {
        *self.variables.get(name).unwrap_or(&0)
    }

    pub fn process_key(&mut self, im: &M17nInputMethod, remaining: &[char]) -> usize {
        let Some(state) = im.states.iter().find(|s| s.name == self.current_state) else {
            return 0;
        };

        let mut best_match = 0;
        let mut best_map_actions: Option<&Vec<Action>> = None;
        let mut best_branch_actions: Option<&Vec<Action>> = None;

        for branch in &state.branches {
            if let Some(keyseq_map) = im.maps.get(&branch.map_name) {
                for len in 1..=remaining.len().min(10) {
                    let key_str: String = remaining[..len].iter().collect();
                    if let Some(idx) = keyseq_map.fst.get(key_str.as_bytes()) {
                        if len > best_match {
                            best_match = len;
                            best_map_actions = keyseq_map.rules.get(idx as usize);
                            best_branch_actions = Some(&branch.actions);
                        }
                    }
                }
            }
        }

        if best_match > 0 {
            if let Some(actions) = best_map_actions {
                for action in actions {
                    self.execute_action(action, im);
                }
            }
            if let Some(actions) = best_branch_actions {
                for action in actions {
                    self.execute_action(action, im);
                }
            }
        } else if let Some(state) = im.states.iter().find(|s| s.name == self.current_state) {
            for action in &state.default_actions {
                self.execute_action(action, im);
            }
        }

        best_match
    }

    fn execute_action(&mut self, action: &Action, im: &M17nInputMethod) {
        match action {
            Action::Insert(arg) => match arg {
                InsertArg::Str(s) => self.insert(s),
                InsertArg::Char(ch) => self.insert_char(*ch),
                InsertArg::Expr(expr) => {
                    let val = self.eval_expr(expr);
                    if let Some(ch) = u32::try_from(val).ok().and_then(char::from_u32) {
                        self.insert_char(ch);
                    }
                }
            },
            Action::Delete(pos) => self.delete(pos),
            Action::Move(pos) => self.move_cursor(pos),
            Action::Set(var, expr) => {
                let value = self.eval_expr(expr);
                self.set_var(var, value);
            }
            Action::Shift(state_name) => {
                self.current_state = state_name.clone();
                if let Some(state) = im.states.iter().find(|s| s.name == self.current_state) {
                    for a in &state.state_actions {
                        self.execute_action(a, im);
                    }
                }
            }
            Action::Commit => self.commit(),
            Action::Cond(branches) => {
                for branch in branches {
                    if self.eval_expr(&branch.condition) != 0 {
                        for a in &branch.actions {
                            self.execute_action(a, im);
                        }
                        break;
                    }
                }
            }
        }
    }

    fn eval_expr(&self, expr: &Expr) -> i64 {
        match expr {
            Expr::Int(i) => *i,
            Expr::Var(name) => self.get_var(name),
            Expr::CharAt(pos) => self.get_char_at(*pos),
            Expr::BinOp(op, left, right) => {
                let l = self.eval_expr(left);
                let r = self.eval_expr(right);
                match op {
                    BinOp::Add => l + r,
                    BinOp::Sub => l - r,
                    BinOp::Mul => l * r,
                    BinOp::Div => if r != 0 { l / r } else { 0 },
                    BinOp::BitAnd => l & r,
                    BinOp::BitOr => l | r,
                    BinOp::Eq => (l == r) as i64,
                    BinOp::Lt => (l < r) as i64,
                    BinOp::Gt => (l > r) as i64,
                    BinOp::Le => (l <= r) as i64,
                    BinOp::Ge => (l >= r) as i64,
                }
            }
            Expr::Not(inner) => (self.eval_expr(inner) == 0) as i64,
        }
    }
}
