// Copyright (C) 2025 Mahmud Nabil
// Portions derived from the M17N library
//   Copyright (C) 2003–2012 AIST (H15PRO112)
// Licensed under the GNU Lesser General Public License v2.1 or later.
// See the LICENSE file for full terms.

#[derive(Debug, Clone)]
pub enum Element {
    List(Vec<Element>),
    Str(String),
    Int(i64),
    Symbol(String),
}

/// Parse a Lisp-like expression from a string
pub fn parse_mim(input: &str) -> Element {
    let mut chars = input.chars().peekable();
    let mut result = Vec::new();

    while let Some(_) = chars.peek() {
        skip_whitespace_and_comments(&mut chars);
        if chars.peek().is_none() {
            break;
        }
        if let Some(node) = parse_element(&mut chars) {
            result.push(node);
        }
    }
    Element::List(result)
}

fn parse_element<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Option<Element> {
    // Skip whitespace and comments
    skip_whitespace_and_comments(chars);

    let c = chars.next()?;

    match c {
        '(' => parse_list(chars),
        '"' => parse_string(chars),
        '0'..='9' | '-' | '?' | '#' => parse_integer(chars, c),
        ')' => None,
        _ => parse_symbol(chars, c),
    }
}

fn skip_whitespace_and_comments<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) {
    loop {
        // Skip whitespace
        while chars.peek().map_or(false, |&c| c <= ' ') {
            chars.next();
        }

        // Skip comments (from ';' to end of line)
        if chars.peek() == Some(&';') {
            chars.next();
            while chars.peek().map_or(false, |&c| c != '\n') {
                chars.next();
            }
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
        } else {
            break;
        }
    }
}

fn parse_list<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Option<Element> {
    let mut elements = Vec::new();

    while let Some(element) = parse_element(chars) {
        elements.push(element);
    }

    Some(Element::List(elements))
}

fn parse_string<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Option<Element> {
    let mut result = String::new();

    while let Some(c) = chars.next() {
        if c == '"' {
            return Some(Element::Str(result));
        }

        if c == '\\' {
            if let Some(escaped) = chars.next() {
                if escaped == '\n' {
                    continue; // Skip escaped newlines
                }

                if escaped == 'x' || escaped == 'u' {
                    // Parse hex unicode
                    let code = parse_hex_number(chars);
                    if let Some(ch) = char::from_u32(code as u32) {
                        result.push(ch);
                    }
                    // Skip optional space after hex
                    if chars.peek() == Some(&' ') {
                        chars.next();
                    }
                } else {
                    result.push(unescape_char(escaped));
                }
            }
        } else {
            result.push(c);
        }
    }

    Some(Element::Str(result))
}

fn parse_integer<I: Iterator<Item = char>>(
    chars: &mut std::iter::Peekable<I>,
    first: char,
) -> Option<Element> {
    let num = match first {
        '#' => {
            if chars.peek() == Some(&'x') {
                chars.next();
                parse_hex_number(chars) as i64
            } else {
                // Not a hex number, parse as symbol instead
                return parse_symbol(chars, first);
            }
        }
        '0' => {
            if chars.peek() == Some(&'x') {
                chars.next();
                parse_hex_number(chars) as i64
            } else {
                parse_decimal(chars, '0')
            }
        }
        '?' => {
            // Character literal
            parse_char_literal(chars)
        }
        '-' => {
            if let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    chars.next();
                    -parse_decimal(chars, c)
                } else {
                    return parse_symbol(chars, first);
                }
            } else {
                return parse_symbol(chars, first);
            }
        }
        _ => parse_decimal(chars, first),
    };

    Some(Element::Int(num))
}

fn parse_decimal<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>, first: char) -> i64 {
    let mut num = (first as u8 - b'0') as i64;

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            chars.next();
            num = num * 10 + (c as u8 - b'0') as i64;
        } else {
            break;
        }
    }

    num
}

fn parse_hex_number<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> u32 {
    let mut num = 0u32;

    while let Some(&c) = chars.peek() {
        let digit = match c {
            '0'..='9' => c as u32 - '0' as u32,
            'A'..='F' => c as u32 - 'A' as u32 + 10,
            'a'..='f' => c as u32 - 'a' as u32 + 10,
            _ => break,
        };
        chars.next();
        num = (num << 4) | digit;
    }

    num
}

fn parse_char_literal<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> i64 {
    if let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(escaped) = chars.next() {
                unescape_char(escaped) as i64
            } else {
                '\\' as i64
            }
        } else {
            c as i64
        }
    } else {
        0
    }
}

fn parse_symbol<I: Iterator<Item = char>>(
    chars: &mut std::iter::Peekable<I>,
    first: char,
) -> Option<Element> {
    let mut result = String::new();
    result.push(if first == '\\' {
        chars.next().map(unescape_char).unwrap_or('\\')
    } else {
        first
    });

    while let Some(&c) = chars.peek() {
        if c <= ' ' || c == ')' || c == '(' || c == '"' {
            break;
        }

        chars.next();

        if c == '\\' {
            if let Some(escaped) = chars.next() {
                result.push(unescape_char(escaped));
            }
        } else {
            result.push(c);
        }
    }

    Some(Element::Symbol(result))
}

fn unescape_char(c: char) -> char {
    match c {
        'e' => '\x1b',
        'b' => '\x08',
        'f' => '\x0c',
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        _ => c,
    }
}
