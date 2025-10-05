// Copyright (C) 2025 Mahmud Nabil
// Portions derived from the M17N library
//   Copyright (C) 2003–2012 AIST (H15PRO112)
// Licensed under the GNU Lesser General Public License v2.1 or later.
// See the LICENSE file for full terms.

use crate::lisp_parser::Element;
use fst::Map;
use std::collections::HashMap;

#[derive(Debug)]
pub struct KeySeqMap {
    pub fst: Map<Vec<u8>>,
    pub values: Vec<Element>,
}

#[derive(Debug)]
pub struct State {
    pub name: String,
    pub actions: Vec<Element>,
}

pub struct ImInfo {
    pub lang: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub maps: HashMap<String, KeySeqMap>,
    pub states: Vec<State>,
}

pub fn load_im_info(parsed_mim: Element) -> ImInfo {
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

    ImInfo {
        lang,
        name,
        title,
        description,
        maps,
        states,
    }
}

fn parse_maps(map_elements: &[Element]) -> HashMap<String, KeySeqMap> {
    let mut maps = HashMap::new();

    for element in map_elements {
        if let Element::List(map_def) = element {
            if map_def.is_empty() {
                continue;
            }

            if let Element::Symbol(map_name) = &map_def[0] {
                let mut keys = Vec::new();
                let mut values = Vec::new();

                for rule in &map_def[1..] {
                    if let Element::List(rule_parts) = rule {
                        if rule_parts.is_empty() {
                            continue;
                        }

                        // KEYSEQ is the first element
                        let keyseq = element_to_keyseq(&rule_parts[0]);

                        // MAP-ACTIONs are the remaining elements
                        let actions = Element::List(rule_parts[1..].to_vec());

                        keys.push(keyseq);
                        values.push(actions);
                    }
                }

                // Build FST from keys
                if !keys.is_empty() {
                    let mut builder = fst::MapBuilder::memory();

                    // Sort keys for FST building
                    let mut indexed_keys: Vec<_> = keys.into_iter().enumerate().collect();
                    indexed_keys.sort_by(|a, b| a.1.cmp(&b.1));

                    for (idx, key) in indexed_keys {
                        builder.insert(&key, idx as u64).unwrap();
                    }

                    let fst_bytes = builder.into_inner().unwrap();
                    let fst = Map::new(fst_bytes).unwrap();

                    maps.insert(map_name.clone(), KeySeqMap { fst, values });
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
                // Store branches as a list
                let branches = state_def[1..].to_vec();
                states.push(State{ name: state_name.to_string(), actions: branches });
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
            // Concatenate all elements in the list
            let mut result = Vec::new();
            for item in list {
                result.extend_from_slice(&element_to_keyseq(item));
            }
            result
        }
        Element::Int(i) => i.to_string().as_bytes().to_vec(),
    }
}
