use std::collections::HashMap;
use crate::im_info::*;
use crate::lisp_parser::*;

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

    pub fn process_key(&mut self, im_info: &ImInfo, remaining: &[char]) -> usize {
        // Find current state
        let state = im_info.states.iter()
            .find(|s| s.name == self.current_state);

        if state.is_none() {
            return 0;
        }

        let state = state.unwrap();

        let mut best_match = 0;
        let mut best_map_actions = None;
        let mut best_branch_actions = None;
        // Try to match against each map in the state
        for branch in &state.branches {
            if let Some(keyseq_map) = im_info.maps.get(&branch.map_name) {
                // Try to find longest matching key sequence
                for len in 1..=remaining.len().min(10) {
                    let key_str: String = remaining[..len].iter().collect();
                    let key_bytes = key_str.as_bytes();

                    // Check if this key sequence exists in the FST
                    if let Some(idx) = keyseq_map.fst.get(key_bytes) && len > best_match {
                        best_match = len;
                        best_map_actions = keyseq_map.rules.get(idx as usize);
                        best_branch_actions = Some(&branch.actions);
                    }
                }
            }
        }

        // Execute the map actions for the best match
        if let Some(map_actions) = best_map_actions {
            if let Element::List(action_list) = map_actions {
                for action in action_list {
                    self.execute_action(action);
                }
            } else {
                self.execute_action(map_actions);
            }
        }

        // Execute the branch actions for the best match
        if let Some(branch_actions) = best_branch_actions {
            for branch_action in branch_actions {
                self.execute_action(&branch_action);
            }
        }

        best_match
    }

    fn execute_action(&mut self, action: &Element) {
        match action {
            Element::Str(s) => self.insert(s),
            Element::Int(ch) => self.insert_char(char::from_u32(*ch as u32).unwrap_or('?')),
            Element::List(list) if !list.is_empty() => {
                if let Element::Symbol(cmd) = &list[0] {
                    match cmd.as_str() {
                        "insert" => {
                            if list.len() > 1 {
                                match &list[1] {
                                    Element::Str(s) => self.insert(s),
                                    Element::Int(ch) => self.insert_char(
                                        char::from_u32(*ch as u32).unwrap_or('?')
                                    ),
                                    Element::Symbol(var) => {
                                        let val = self.get_var(var);
                                        if val > 0 {
                                            self.insert_char(
                                                char::from_u32(val as u32).unwrap_or('?')
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        "delete" => {
                            if list.len() > 1 {
                                if let Some(pos) = self.resolve_pos_arg(&list[1]) {
                                    self.delete(pos);
                                }
                            }
                        }
                        "move" => {
                            if list.len() > 1 {
                                if let Some(pos) = self.resolve_pos_arg(&list[1]) {
                                    self.move_pos(pos);
                                }
                            }
                        }
                        "set" => {
                            if list.len() > 2 {
                                if let Element::Symbol(var) = &list[1] {
                                    let value = self.eval_expr(&list[2]);
                                    self.set_var(var, value);
                                }
                            }
                        }
                        "shift" => {
                            if list.len() > 1 {
                                if let Element::Symbol(state) = &list[1] {
                                    self.current_state = state.clone();
                                }
                            }
                        }
                        "commit" => self.commit(),
                        "cond" => {
                            for i in 1..list.len() {
                                if let Element::List(branch) = &list[i] {
                                    if !branch.is_empty() {
                                        let cond_val = self.eval_expr( &branch[0]);
                                        if cond_val != 0 {
                                            for j in 1..branch.len() {
                                                self.execute_action(&branch[j]);
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn resolve_pos_arg(&self, arg: &Element) -> Option<isize> {
        match arg {
            Element::Symbol(s) => match s.as_str() {
                "@<" => Some(0),
                "@>" => Some(isize::MAX),
                "@-" => Some(-1),
                "@+" => Some(1),
                _ => None,
            },
            Element::Int(i) => Some(*i as isize),
            _ => None,
        }
    }

    fn eval_expr(&self, expr: &Element) -> i64 {
        match expr {
            Element::Int(i) => *i,
            Element::Symbol(s) => {
                if s.starts_with("@-") || s.starts_with("@+") {
                    let offset: isize = s[1..].parse().unwrap_or(0);
                    self.get_char_at(offset)
                } else if s == "@-0" {
                    -1 // Surrounding text supported
                } else {
                    self.get_var(s)
                }
            }
            Element::List(list) if !list.is_empty() => {
                if let Element::Symbol(op) = &list[0] {
                    match op.as_str() {
                        "+" if list.len() > 2 => {
                            self.eval_expr(&list[1]) + self.eval_expr(&list[2])
                        }
                        "-" if list.len() > 2 => {
                            self.eval_expr(&list[1]) - self.eval_expr(&list[2])
                        }
                        "*" if list.len() > 2 => {
                            self.eval_expr(&list[1]) * self.eval_expr(&list[2])
                        }
                        "/" if list.len() > 2 => {
                            let divisor = self.eval_expr(&list[2]);
                            if divisor != 0 {
                                self.eval_expr(&list[1]) / divisor
                            } else {
                                0
                            }
                        }
                        "&" if list.len() > 2 => {
                            self.eval_expr(&list[1]) & self.eval_expr(&list[2])
                        }
                        "|" if list.len() > 2 => {
                            self.eval_expr(&list[1]) | self.eval_expr(&list[2])
                        }
                        "==" if list.len() > 2 => {
                            if self.eval_expr(&list[1]) == self.eval_expr(&list[2]) {
                                1
                            } else {
                                0
                            }
                        }
                        "=" if list.len() > 2 => {
                            if self.eval_expr(&list[1]) == self.eval_expr(&list[2]) {
                                1
                            } else {
                                0
                            }
                        }
                        "<" if list.len() > 2 => {
                            if self.eval_expr(&list[1]) < self.eval_expr(&list[2]) {
                                1
                            } else {
                                0
                            }
                        }
                        ">" if list.len() > 2 => {
                            if self.eval_expr(&list[1]) > self.eval_expr(&list[2]) {
                                1
                            } else {
                                0
                            }
                        }
                        "<=" if list.len() > 2 => {
                            if self.eval_expr(&list[1]) <= self.eval_expr(&list[2]) {
                                1
                            } else {
                                0
                            }
                        }
                        ">=" if list.len() > 2 => {
                            if self.eval_expr(&list[1]) >= self.eval_expr(&list[2]) {
                                1
                            } else {
                                0
                            }
                        }
                        "!" if list.len() > 1 => {
                            if self.eval_expr(&list[1]) == 0 {
                                1
                            } else {
                                0
                            }
                        }
                        _ => 0,
                    }
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
}
