mod im_info;
mod lisp_parser;

use crate::im_info::*;
use crate::lisp_parser::*;
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
        input.to_string()
    }
}

impl M17nMim {}
