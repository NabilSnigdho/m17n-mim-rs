mod im_info;
mod lisp_parser;
mod context;

use crate::im_info::*;
use crate::lisp_parser::*;
use crate::context::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct M17nMim {
    im_info: ImInfo,
}

#[wasm_bindgen]
impl M17nMim {
    #[wasm_bindgen(constructor)]
    pub fn new(mim_str: &str) -> M17nMim {
        // parse the MIM string at creation
        let list = parse_mim(mim_str);
        let im_info = load_im_info(list);
        M17nMim { im_info }
    }

    #[wasm_bindgen]
    pub fn get_lang(&self) -> String {
        self.im_info.lang.clone()
    }

    #[wasm_bindgen]
    pub fn get_name(&self) -> String {
        self.im_info.name.clone()
    }

    #[wasm_bindgen]
    pub fn get_title(&self) -> String {
        self.im_info.title.clone()
    }

    #[wasm_bindgen]
    pub fn get_description(&self) -> String {
        self.im_info.description.clone()
    }

    #[wasm_bindgen]
    pub fn convert(&self, input: &str) -> String {
        let mut ctx = Context::new(
            self.im_info.states.first()
                .map(|s| s.name.clone())
                .unwrap_or_else(|| "init".to_string())
        );

        let input_chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < input_chars.len() {
            let matched = self.process_key(&mut ctx, &input_chars[i..]);
            if matched > 0 {
                i += matched;
            } else {
                // No match, commit current preedit and add the character directly
                ctx.commit();
                ctx.committed.push(input_chars[i]);
                i += 1;
            }
        }

        // Commit any remaining preedit
        ctx.commit();
        ctx.committed
    }

    fn process_key(&self, ctx: &mut Context, remaining: &[char]) -> usize {
        // Find current state
        let state = self.im_info.states.iter()
            .find(|s| s.name == ctx.current_state);

        if state.is_none() {
            return 0;
        }

        let state = state.unwrap();

        // Try to match against each map in the state
        for branch in &state.branches {
            if let Some(keyseq_map) = self.im_info.maps.get(&branch.map_name) {
                // Try to find longest matching key sequence
                let matched = self.match_keys(keyseq_map, remaining, ctx);
                if matched > 0 {
                    // Execute branch actions (elements after map name)
                    for i in 1..branch.actions.len() {
                        self.execute_action(ctx, &branch.actions[i]);
                    }
                    return matched;
                }
            }
        }

        0
    }

    fn match_keys(&self, keyseq_map: &KeySeqRuleMap, keys: &[char], ctx: &mut Context) -> usize {
        let mut best_match = 0;
        let mut best_actions = None;

        // Try progressively longer key sequences
        for len in 1..=keys.len().min(10) {
            let key_str: String = keys[..len].iter().collect();
            let key_bytes = key_str.as_bytes();

            // Check if this key sequence exists in the FST
            if let Some(idx) = keyseq_map.fst.get(key_bytes) {
                best_match = len;
                best_actions = keyseq_map.rules.get(idx as usize);
            }
        }

        // Execute the map actions for the best match
        if let Some(actions) = best_actions {
            if let Element::List(action_list) = actions {
                for action in action_list {
                    self.execute_action(ctx, action);
                }
            } else {
                self.execute_action(ctx, actions);
            }
        }

        best_match
    }

    fn execute_action(&self, ctx: &mut Context, action: &Element) {
        match action {
            Element::Str(s) => ctx.insert(s),
            Element::Int(ch) => ctx.insert_char(char::from_u32(*ch as u32).unwrap_or('?')),
            Element::List(list) if !list.is_empty() => {
                if let Element::Symbol(cmd) = &list[0] {
                    match cmd.as_str() {
                        "insert" => {
                            if list.len() > 1 {
                                match &list[1] {
                                    Element::Str(s) => ctx.insert(s),
                                    Element::Int(ch) => ctx.insert_char(
                                        char::from_u32(*ch as u32).unwrap_or('?')
                                    ),
                                    Element::Symbol(var) => {
                                        let val = ctx.get_var(var);
                                        if val > 0 {
                                            ctx.insert_char(
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
                                    ctx.delete(pos);
                                }
                            }
                        }
                        "move" => {
                            if list.len() > 1 {
                                if let Some(pos) = self.resolve_pos_arg(&list[1]) {
                                    ctx.move_pos(pos);
                                }
                            }
                        }
                        "set" => {
                            if list.len() > 2 {
                                if let Element::Symbol(var) = &list[1] {
                                    let value = self.eval_expr(ctx, &list[2]);
                                    ctx.set_var(var, value);
                                }
                            }
                        }
                        "shift" => {
                            if list.len() > 1 {
                                if let Element::Symbol(state) = &list[1] {
                                    ctx.current_state = state.clone();
                                }
                            }
                        }
                        "commit" => ctx.commit(),
                        "cond" => {
                            for i in 1..list.len() {
                                if let Element::List(branch) = &list[i] {
                                    if !branch.is_empty() {
                                        let cond_val = self.eval_expr(ctx, &branch[0]);
                                        if cond_val != 0 {
                                            for j in 1..branch.len() {
                                                self.execute_action(ctx, &branch[j]);
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

    fn eval_expr(&self, ctx: &Context, expr: &Element) -> i64 {
        match expr {
            Element::Int(i) => *i,
            Element::Symbol(s) => {
                if s.starts_with("@-") || s.starts_with("@+") {
                    let offset: isize = s[1..].parse().unwrap_or(0);
                    ctx.get_char_at(offset)
                } else if s == "@-0" {
                    -1 // Surrounding text supported
                } else {
                    ctx.get_var(s)
                }
            }
            Element::List(list) if !list.is_empty() => {
                if let Element::Symbol(op) = &list[0] {
                    match op.as_str() {
                        "+" if list.len() > 2 => {
                            self.eval_expr(ctx, &list[1]) + self.eval_expr(ctx, &list[2])
                        }
                        "-" if list.len() > 2 => {
                            self.eval_expr(ctx, &list[1]) - self.eval_expr(ctx, &list[2])
                        }
                        "*" if list.len() > 2 => {
                            self.eval_expr(ctx, &list[1]) * self.eval_expr(ctx, &list[2])
                        }
                        "/" if list.len() > 2 => {
                            let divisor = self.eval_expr(ctx, &list[2]);
                            if divisor != 0 {
                                self.eval_expr(ctx, &list[1]) / divisor
                            } else {
                                0
                            }
                        }
                        "&" if list.len() > 2 => {
                            self.eval_expr(ctx, &list[1]) & self.eval_expr(ctx, &list[2])
                        }
                        "|" if list.len() > 2 => {
                            self.eval_expr(ctx, &list[1]) | self.eval_expr(ctx, &list[2])
                        }
                        "==" if list.len() > 2 => {
                            if self.eval_expr(ctx, &list[1]) == self.eval_expr(ctx, &list[2]) {
                                1
                            } else {
                                0
                            }
                        }
                        "=" if list.len() > 2 => {
                            if self.eval_expr(ctx, &list[1]) == self.eval_expr(ctx, &list[2]) {
                                1
                            } else {
                                0
                            }
                        }
                        "<" if list.len() > 2 => {
                            if self.eval_expr(ctx, &list[1]) < self.eval_expr(ctx, &list[2]) {
                                1
                            } else {
                                0
                            }
                        }
                        ">" if list.len() > 2 => {
                            if self.eval_expr(ctx, &list[1]) > self.eval_expr(ctx, &list[2]) {
                                1
                            } else {
                                0
                            }
                        }
                        "<=" if list.len() > 2 => {
                            if self.eval_expr(ctx, &list[1]) <= self.eval_expr(ctx, &list[2]) {
                                1
                            } else {
                                0
                            }
                        }
                        ">=" if list.len() > 2 => {
                            if self.eval_expr(ctx, &list[1]) >= self.eval_expr(ctx, &list[2]) {
                                1
                            } else {
                                0
                            }
                        }
                        "!" if list.len() > 1 => {
                            if self.eval_expr(ctx, &list[1]) == 0 {
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
