// Copyright (C) 2025 Mahmud Nabil
// Portions derived from the M17N library
//   Copyright (C) 2003–2012 AIST (H15PRO112)
// Licensed under the GNU Lesser General Public License v2.1 or later.
// See the LICENSE file for full terms.

use crate::lisp_parser::Element;
use fst::Map;
use std::collections::HashMap;

// --- Typed action model ---

#[derive(Debug, Clone)]
pub enum Pos {
    Beginning,
    End,
    Relative(i32),
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    BitAnd,
    BitOr,
    Eq,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Var(String),
    /// Raw position value passed to get_char_at (e.g. @-1 → -1, @+1 → 1)
    CharAt(i32),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum InsertArg {
    Str(String),
    Char(char),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct CondBranch {
    pub condition: Expr,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone)]
pub enum Action {
    Insert(InsertArg),
    Delete(Pos),
    Move(Pos),
    Set(String, Expr),
    Shift(String),
    Commit,
    Cond(Vec<CondBranch>),
}

// --- Data structures ---

#[derive(Debug)]
pub struct KeySeqRuleMap {
    pub fst: Map<Vec<u8>>,
    pub rules: Vec<Vec<Action>>,
}

#[derive(Debug)]
pub struct Branch {
    pub map_name: String,
    pub actions: Vec<Action>,
}

#[derive(Debug)]
pub struct State {
    pub name: String,
    pub branches: Vec<Branch>,
    pub state_actions: Vec<Action>,
    pub default_actions: Vec<Action>,
}

pub struct M17nInputMethod {
    pub lang: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub maps: HashMap<String, KeySeqRuleMap>,
    pub states: Vec<State>,
}

// --- Public loader ---

pub fn load_im_info(parsed_mim: Element) -> M17nInputMethod {
    let mut lang = String::new();
    let mut name = String::new();
    let mut title = String::new();
    let mut description = String::new();
    let mut maps = HashMap::new();
    let mut states = Vec::new();

    if let Element::List(root) = parsed_mim {
        for element in root {
            if let Element::List(section) = element {
                if section.is_empty() {
                    continue;
                }

                match &section[0] {
                    Element::Symbol(sym) if sym == "input-method" => {
                        if let Some(Element::Symbol(l)) = section.get(1) {
                            lang = l.clone();
                        }
                        if let Some(Element::Symbol(n)) = section.get(2) {
                            name = n.clone();
                        }
                    }
                    Element::Symbol(sym) if sym == "description" => {
                        if let Some(Element::List(desc_list)) = section.get(1) {
                            if let Some(Element::Str(s)) = desc_list.get(1) {
                                description = s.clone();
                            }
                        } else if let Some(Element::Str(s)) = section.get(1) {
                            description = s.clone();
                        }
                    }
                    Element::Symbol(sym) if sym == "title" => {
                        if let Some(Element::Str(t)) = section.get(1) {
                            title = t.clone();
                        }
                    }
                    Element::Symbol(sym) if sym == "map" => {
                        maps = parse_maps(&section[1..]);
                    }
                    Element::Symbol(sym) if sym == "state" => {
                        states = parse_states(&section[1..]);
                    }
                    _ => {}
                }
            }
        }
    }

    M17nInputMethod { lang, name, title, description, maps, states }
}

// --- Map / state builders ---

fn parse_maps(map_elements: &[Element]) -> HashMap<String, KeySeqRuleMap> {
    let mut maps = HashMap::new();

    for element in map_elements {
        if let Element::List(map_def) = element {
            if map_def.is_empty() {
                continue;
            }

            if let Element::Symbol(map_name) = &map_def[0] {
                if map_name == "t" || map_name == "nil" {
                    continue;
                }
                let mut keys: Vec<Vec<u8>> = Vec::new();
                let mut values: Vec<Vec<Action>> = Vec::new();

                for rule in &map_def[1..] {
                    if let Element::List(rule_parts) = rule {
                        if rule_parts.is_empty() {
                            continue;
                        }

                        let keyseq = element_to_keyseq(&rule_parts[0]);
                        if keys.contains(&keyseq) {
                            continue;
                        }

                        keys.push(keyseq);
                        values.push(parse_actions(&rule_parts[1..]));
                    }
                }

                if !keys.is_empty() {
                    let mut builder = fst::MapBuilder::memory();
                    let mut indexed_keys: Vec<_> = keys.into_iter().enumerate().collect();
                    indexed_keys.sort_by(|a, b| a.1.cmp(&b.1));

                    for (idx, key) in indexed_keys {
                        builder.insert(&key, idx as u64).unwrap();
                    }

                    let fst = Map::new(builder.into_inner().unwrap()).unwrap();
                    maps.insert(map_name.clone(), KeySeqRuleMap { fst, rules: values });
                }
            }
        }
    }

    maps
}

fn parse_states(state_elements: &[Element]) -> Vec<State> {
    let mut states = Vec::new();

    for element in state_elements {
        if let Element::List(state_def) = element {
            if state_def.is_empty() {
                continue;
            }

            if let Element::Symbol(state_name) = &state_def[0] {
                let mut branches = Vec::new();
                let mut state_actions = Vec::new();
                let mut default_actions = Vec::new();
                for element in &state_def[1..] {
                    if let Element::List(branch_def) = element {
                        if branch_def.is_empty() {
                            continue;
                        }
                        if let Element::Symbol(map_name) = &branch_def[0] {
                            match map_name.as_str() {
                                "t" => state_actions = parse_actions(&branch_def[1..]),
                                "nil" => default_actions = parse_actions(&branch_def[1..]),
                                _ => branches.push(Branch {
                                    map_name: map_name.to_string(),
                                    actions: parse_actions(&branch_def[1..]),
                                }),
                            }
                        }
                    }
                }
                states.push(State {
                    name: state_name.to_string(),
                    branches,
                    state_actions,
                    default_actions,
                });
            }
        }
    }

    states
}

fn element_to_keyseq(element: &Element) -> Vec<u8> {
    match element {
        Element::Str(s) => s.as_bytes().to_vec(),
        Element::Symbol(s) => s.as_bytes().to_vec(),
        Element::List(list) => {
            let mut result = Vec::new();
            for item in list {
                result.extend_from_slice(&element_to_keyseq(item));
            }
            result
        }
        Element::Int(i) => i.to_string().as_bytes().to_vec(),
    }
}

// --- Action / expr parsers ---

fn parse_actions(elems: &[Element]) -> Vec<Action> {
    elems.iter().filter_map(parse_action).collect()
}

fn parse_action(elem: &Element) -> Option<Action> {
    match elem {
        Element::Str(s) => Some(Action::Insert(InsertArg::Str(s.clone()))),
        Element::Int(ch) => {
            char::from_u32(*ch as u32).map(|c| Action::Insert(InsertArg::Char(c)))
        }
        Element::List(list) if !list.is_empty() => {
            let Element::Symbol(cmd) = &list[0] else { return None; };
            match cmd.as_str() {
                "insert" => {
                    if list.len() < 2 {
                        return None;
                    }
                    let arg = match &list[1] {
                        Element::Str(s) => InsertArg::Str(s.clone()),
                        Element::Int(ch) => InsertArg::Char(char::from_u32(*ch as u32)?),
                        elem => InsertArg::Expr(parse_expr(elem)),
                    };
                    Some(Action::Insert(arg))
                }
                "delete" => list.get(1).and_then(parse_pos).map(Action::Delete),
                "move" => list.get(1).and_then(parse_pos).map(Action::Move),
                "set" => {
                    if list.len() < 3 {
                        return None;
                    }
                    let Element::Symbol(var) = &list[1] else { return None; };
                    Some(Action::Set(var.clone(), parse_expr(&list[2])))
                }
                "shift" => {
                    if list.len() < 2 {
                        return None;
                    }
                    let Element::Symbol(state) = &list[1] else { return None; };
                    Some(Action::Shift(state.clone()))
                }
                "commit" => Some(Action::Commit),
                "cond" => {
                    let branches: Vec<CondBranch> = list[1..]
                        .iter()
                        .filter_map(|branch| {
                            let Element::List(b) = branch else { return None; };
                            if b.is_empty() {
                                return None;
                            }
                            Some(CondBranch {
                                condition: parse_expr(&b[0]),
                                actions: parse_actions(&b[1..]),
                            })
                        })
                        .collect();
                    if branches.is_empty() { None } else { Some(Action::Cond(branches)) }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn parse_pos(elem: &Element) -> Option<Pos> {
    match elem {
        Element::Symbol(s) => match s.as_str() {
            "@<" => Some(Pos::Beginning),
            "@>" => Some(Pos::End),
            "@-" => Some(Pos::Relative(-1)),
            "@+" => Some(Pos::Relative(1)),
            s if s.starts_with("@-") => s[2..].parse::<i32>().ok().map(|n| Pos::Relative(-n)),
            s if s.starts_with("@+") => s[2..].parse::<i32>().ok().map(|n| Pos::Relative(n)),
            _ => None,
        },
        Element::Int(i) => Some(Pos::Relative(*i as i32)),
        _ => None,
    }
}

fn parse_expr(elem: &Element) -> Expr {
    match elem {
        Element::Int(i) => Expr::Int(*i),
        Element::Symbol(s) if s.starts_with("@-") || s.starts_with("@+") => {
            // s[1..] gives "-1", "+1", "-", "+" etc.; parse yields the signed offset
            let n: i32 = s[1..].parse().unwrap_or(0);
            Expr::CharAt(n)
        }
        Element::Symbol(s) => Expr::Var(s.clone()),
        Element::List(list) if !list.is_empty() => {
            let Element::Symbol(op) = &list[0] else { return Expr::Int(0); };
            if op == "!" && list.len() > 1 {
                return Expr::Not(Box::new(parse_expr(&list[1])));
            }
            if list.len() > 2 {
                let left = Box::new(parse_expr(&list[1]));
                let right = Box::new(parse_expr(&list[2]));
                let binop = match op.as_str() {
                    "+" => Some(BinOp::Add),
                    "-" => Some(BinOp::Sub),
                    "*" => Some(BinOp::Mul),
                    "/" => Some(BinOp::Div),
                    "&" => Some(BinOp::BitAnd),
                    "|" => Some(BinOp::BitOr),
                    "==" | "=" => Some(BinOp::Eq),
                    "<" => Some(BinOp::Lt),
                    ">" => Some(BinOp::Gt),
                    "<=" => Some(BinOp::Le),
                    ">=" => Some(BinOp::Ge),
                    _ => None,
                };
                if let Some(op) = binop {
                    return Expr::BinOp(op, left, right);
                }
            }
            Expr::Int(0)
        }
        _ => Expr::Int(0),
    }
}
