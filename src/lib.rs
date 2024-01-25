use regex::Regex;
use serde::{Serialize};
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
extern crate console_error_panic_hook;

use console_error_panic_hook::set_once as set_panic_hook;

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
    block: Block
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
    set_panic_hook();
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
    let tokenizer = regex::Regex::new(r"(?x)<!--\s+(?P<closer>/)?wp:(?P<namespace>[a-z][a-z0-9_-]*/)?(?P<name>[a-z][a-z0-9_-]*)\s+(?P<attrs>\{(?:(?:[^}]+|}+)*)?})?(?P<void>/)?\s*-->").unwrap();
    let stack_len = stack.len();
    let token = next_token(&document, &tokenizer, *offset);

    let (token_type, block_name, attrs, token_start, token_length) = token;
    let size = if token_start > *offset { Some(*offset) } else { None };

    match token_type.as_ref() {
        "no-more-tokens" => {
            if stack_len == 0 {
                add_freeform(document, output, *offset, Some(document.len()));
                *offset = document.len();
            }
            None
        }
        "void-block" => {
            if stack_len > 0 {
                add_inner_block(
                    document,
                    output,
                    size.unwrap(),
                    token_length,
                    block_name.clone(),
                    &attrs,
                    stack
                );
                *offset = token_start + token_length;
            } else {
                if let Some(s) = size {
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
                    size.unwrap(),
                    token_length,
                    block_name.clone(),
                    &attrs,
                    stack
                );
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
                });
            }
            *offset = token_start + token_length;
            Some(())
        }
        "block-closer" => {
            if stack_len > 0 {
                let mut stack_top = stack.pop().unwrap();
                let html = &document[token_start..token_start +token_length];
                stack_top.block.inner_html.push_str(html);
                stack_top.block.inner_content.push(html.to_string());
                add_inner_block(
                    document,
                    output,
                    size.unwrap(),
                    token_length,
                    block_name.clone(),
                    &attrs, // pass as reference to attrs
                    stack
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
            *offset = token_start + token_length;
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

        let is_closing = caps.name("closer").is_some();
        let is_self_closing = caps.name("void").is_some();
        let namespace = caps.name("namespace").map_or("", |m| m.as_str());
        let name = caps.name("name").map_or("", |m| m.as_str());
        let attrs = caps.name("attrs").map_or("", |m| m.as_str());

        let block_name = format!("{}{}", namespace, name);

        if is_self_closing {
            return ("void-block".to_string(), block_name, parse_json(attrs), token_start, token_length);
        } else if is_closing {
            return ("block-closer".to_string(), block_name, parse_json(attrs), token_start, token_length);
        } else {
            return ("block-opener".to_string(), block_name, parse_json(attrs), token_start, token_length);
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
    // Check if the offset is within bounds
    if offset > document.len() {
        // Handle the error or log it as needed
        eprintln!("Error: Offset out of bounds");
        eprintln!("Offset: {}, Document length: {}", offset, document.len());
        return;
    }

    let length = raw_length.unwrap_or_else(|| document.len() - offset);

    if length == 0 {
        return;
    }

    // Calculate the end index and adjust the length if it's out of bounds
    let end_index = std::cmp::min(document.len(), offset + length);

    // Use Block::new or your equivalent constructor method
    // Ensure the slice is within the bounds of the document
    output.push(Block::new(&document[offset..end_index]));

    // Debug prints
    println!("Adding freeform block");
    println!("Offset: {}, Length: {}", offset, end_index - offset);
    println!("Content: {:?}", &document[offset..end_index]);
}


fn add_inner_block(
    document: &str,
    output: &mut Vec<Block>,
    token_start: usize,
    token_length: usize,
    block_name: String,
    attrs: &HashMap<String, Value>,
    stack: &mut Vec<Frame>,
) {
    // Ensure that token_start and token_length are within bounds
    if token_start < document.len() && token_start + token_length <= document.len() {
        let html = &document[token_start..token_start + token_length];

        let mut inner_blocks = Vec::new();
        let inner_document = &document[token_start + token_length..]; // TODO: Fix here

        // Debug prints
        println!("Inner Document: {:?}", inner_document);

        // Recursively parse inner blocks
        let mut inner_offset = 0;
        while let Some(_) = proceed(inner_document, &mut inner_offset, &mut inner_blocks, stack) {}

        let block = Block {
            block_name: Some(block_name.clone()),
            attrs: attrs.clone(),
            inner_blocks,
            inner_html: html.to_string(),
            inner_content: vec![html.to_string()],
        };

        // Debug prints
        eprintln!("Adding inner block: {:?}", block_name);
        eprintln!("Token start: {}, Token length: {}", token_start, token_length);
        eprintln!("HTML: {:?}", html);

        output.push(block);
    } else {
        // Debug prints
        eprintln!("Error: Token position out of bounds");
        eprintln!("Token start: {}, Token length: {}", token_start, token_length);
        eprintln!("Document length: {}", document.len());
    }
}