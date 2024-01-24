use regex::Regex;
use serde::{Serialize};
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Clone, Serialize)]
struct Block {
    block_name: Option<String>,
    attrs: HashMap<String, Value>,
    inner_blocks: Vec<Block>,
    inner_html: String,
    inner_content: Vec<String>,
}

#[derive(Debug)]
struct Frame {
    block: Block,
    prev_offset: usize,
}

// Freeform and inner block helpers
impl Block {
    // Assuming there's a new associated function to create a Block from a &str
    fn new(content: &str) -> Self {
        Block {
            block_name: None,
            inner_html: content.to_string(),
            inner_blocks: Vec::new(),
            inner_content: Vec::new(),
            attrs: HashMap::new(),
        }
    }
}

#[wasm_bindgen]
pub fn parse(document: &str) -> JsValue {
    let mut offset = 0;
    let mut output: Vec<Block> = Vec::new();
    let mut stack: Vec<Frame> = Vec::new();

    while let Some(_) = proceed(&document, &mut offset, &mut output, &mut stack) {}

    serde_wasm_bindgen::to_value(&output).unwrap()
}

fn proceed(
    document: &str,
    offset: &mut usize,
    output: &mut Vec<Block>,
    stack: &mut Vec<Frame>,
) -> Option<()> {
    let tokenizer = regex::Regex::new(r"<!--\s+(\\/)?wp:([a-z][a-z0-9_-]*\\/)?([a-z][a-z0-9_-]*)\s+([^}]*\s+)?(\\/)?-->").unwrap();
    let stack_len = stack.len();
    let token = next_token(&document, &tokenizer, *offset);

    let (token_type, block_name, attrs, token_start, token_length) = token;
    let s = if token_start > *offset { Some(*offset) } else { None };

    match token_type.as_ref() {
        "no-more-tokens" => {
            if stack_len == 0 {
                add_freeform(document, output, *offset, Some(document.len()));
                *offset = document.len();
                output.push( Block::new("asd") );
            }
            None
        }
        "void-block" => {
            if stack_len > 0 {
                add_inner_block(
                    document,
                    output,
                    token_start,
                    token_length,
                    block_name.clone(),
                    &attrs,
                );
                *offset = token_start + token_length;
            } else {
                if let Some(s) = s {
                    add_freeform(document, output, s, Some(token_start));
                }
                output.push(Block {
                    block_name: Some(block_name),
                    attrs,
                    inner_blocks: Vec::new(),
                    inner_html: String::new(),
                    inner_content: Vec::new(),
                });
                *offset = token_start + token_length;
            }
            Some(())
        }
        "block-opener" => {
            if stack_len > 0 {
                add_inner_block(
                    document,
                    output,
                    token_start,
                    token_length,
                    block_name.clone(),
                    &attrs,
                );
                stack.push(Frame {
                    block: Block {
                        block_name: Some(block_name),
                        attrs: attrs.clone(),
                        inner_blocks: Vec::new(),
                        inner_html: String::new(),
                        inner_content: Vec::new(),
                    },
                    prev_offset: token_start + token_length,
                });
            } else {
                add_freeform(document, output, *offset, Some(token_start));
                stack.push(Frame {
                    block: Block {
                        block_name: Some(block_name),
                        attrs,
                        inner_blocks: Vec::new(),
                        inner_html: String::new(),
                        inner_content: Vec::new(),
                    },
                    prev_offset: token_start + token_length,
                });
            }
            *offset = token_start + token_length;
            Some(())
        }
        "block-closer" => {
            if stack_len > 0 {
                let mut stack_top = stack.pop().unwrap();
                let html = &document[stack_top.prev_offset..token_start];
                stack_top.block.inner_html.push_str(html);
                stack_top.block.inner_content.push(html.to_string());
                add_inner_block(
                    document,
                    output,
                    token_start,
                    token_length,
                    block_name.clone(),
                    &attrs, // pass as reference to attrs
                );
                output.push(stack_top.block);
                *offset = token_start + token_length;
            } else {
                // Error: Block closer without corresponding opener
                // Handle error logic or ignore
                *offset = token_start + token_length;
            }
            Some(())
        }
        _ => {
            add_freeform(document, output, *offset, Some(token_start));
            None
        }
    }
}

// Next token helpers
fn next_token(document: &str, tokenizer: &Regex, last_offset: usize) -> (String, String, HashMap<String, Value>, usize, usize) {
    if let Some(caps) = tokenizer.captures(&document[last_offset..]) {
        let token_start = last_offset + caps.get(0).unwrap().start();
        let token_end = last_offset + caps.get(0).unwrap().end();
        let token_length = token_end - token_start;

        let is_closing = caps.get(1).is_some();
        let is_self_closing = caps.get(6).is_some();
        let namespace = caps.get(2).map_or("core/", |m| m.as_str());
        let name = caps.get(3).unwrap().as_str();
        let attrs_json = caps.get(4).map_or("", |m| m.as_str().trim());

        let block_name = format!("{}{}", namespace, name);
        let attrs = parse_json(attrs_json);

        if is_self_closing {
            return ("void-block".to_string(), block_name, attrs, token_start, token_length);
        } else if is_closing {
            return ("block-closer".to_string(), block_name, attrs, token_start, token_length);
        } else {
            return ("block-opener".to_string(), block_name, attrs, token_start, token_length);
        }
    } else {
        return ("no-more-tokens".to_string(), String::new(), HashMap::new(), 0, 0);
    }
}

fn parse_json(json_str: &str) -> HashMap<String, Value> {
    match serde_json::from_str(json_str) {
        Ok(val) => match val {
            Value::Object(obj) => obj.into_iter().collect::<HashMap<String, Value>>(),
            _ => HashMap::new(),
        },
        Err(_) => HashMap::new(),
    }
}
fn add_freeform(document: &str, output: &mut Vec<Block>, offset: usize, raw_length: Option<usize>) {
    let length = raw_length.unwrap_or_else(|| document.len() - offset);

    if length == 0 {
        return;
    }

    // Use Block::new or your equivalent constructor method
    output.push(Block::new(&document[offset..offset + length]));
}

fn extract_inner_content(document: &str, start: usize, end: usize) -> String {
    let inner_content_regex = regex::Regex::new(&format!(r"{}([\s\S]*){}", &start, &end)).unwrap();
    if let Some(caps) = inner_content_regex.captures(document) {
        return caps.get(1).unwrap().as_str().to_string();
    }
    String::new()
}

// Inner block helpers
fn add_inner_block(
    document: &str,
    output: &mut Vec<Block>,
    token_start: usize,
    token_length: usize,
    block_name: String,
    attrs: &HashMap<String, Value>,
) {
    let inner_content = extract_inner_content(document, token_start, token_start + token_length);
    let mut inner_block = Block {
        block_name: Some(block_name),
        attrs: attrs.clone(),
        inner_blocks: Vec::new(),
        inner_html: inner_content.clone(),
        inner_content: vec![inner_content],
    };

    // Remove leading and trailing whitespace from inner block HTML
    inner_block.inner_html = inner_block.inner_html.trim().to_string();

    output.push(inner_block);
}