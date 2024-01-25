
use std::collections::HashMap;
use regex::Regex;
use serde::Serialize;
use serde_json::value::Value;
use wasm_bindgen_test::console_log;

#[derive(Debug, Clone, Serialize)]
pub struct Block {
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

pub fn parser(document: &str) -> Vec<Block> {
    let mut offset = 0;
    let mut output: Vec<Block> = Vec::new();
    let mut stack: Vec<Frame> = Vec::new();

    while let Some(_) = proceed(&document, &mut offset, &mut output, &mut stack) {}

    return output
}

fn proceed(
    document: &str,
    offset: &mut usize,
    output: &mut Vec<Block>,
    stack: &mut Vec<Frame>,
) -> Option<()> {
    let tokenizer = Regex::new(r"(?x)<!--\s+(?P<closer>/)?wp:(?P<namespace>[a-z][a-z0-9_-]*/)?(?P<name>[a-z][a-z0-9_-]*)\s+(?P<attrs>\{(?:(?:[^}]+|}+)*)?})?(?P<void>/)?\s*-->").unwrap();
    let stack_len = stack.len();
    let token = next_token(&document, &tokenizer, *offset);

    let (token_type, block_name, attrs, token_start, token_length) = token;

    console_log!("Token type: {:?}", token_type);
    console_log!("Block name: {:?}", block_name);
    console_log!("Attributes: {:?}", attrs);
    console_log!("Token start: {:?}", token_start);
    console_log!("Token length: {:?}", token_length);

    match token_type.as_ref() {
        "no-more-tokens" => {
            if stack_len == 0 {
                add_freeform(document, output, *offset);
            }
            *offset = document.len();
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
                    stack
                );
            } else {
                add_freeform(document, output, token_start);
                output.push(Block {
                    block_name: Some(block_name),
                    attrs,
                    inner_blocks: Vec::new(),
                    inner_html: String::new(),
                    inner_content: Vec::new(),
                });
            }
            *offset = token_start + token_length;
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
                    stack
                );
            } else {
                add_freeform(document, output, *offset);
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
                    token_start,
                    token_length,
                    block_name.clone(),
                    &attrs, // pass as reference to attrs
                    stack
                );
                output.push(stack_top.block);
            }
            *offset = token_start + token_length;
            Some(())
        }
        _ => {
            add_freeform(document, output, *offset);
            *offset = token_start + token_length;
            None
        }
    }
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
        wasm_bindgen_test::console_log!("Inner Document: {:?}", inner_document);

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
        wasm_bindgen_test::console_log!("Adding inner block: {:?}", block_name);
        wasm_bindgen_test::console_log!("Token start: {}, Token length: {}", token_start, token_length);
        //wasm_bindgen_test::console_log!("HTML: {:?}", html);

        output.push(block);
    } else {
        // Debug prints
        wasm_bindgen_test::console_log!("Error: Token position out of bounds");
        wasm_bindgen_test::console_log!("Token start: {}, Token length: {}", token_start, token_length);
        wasm_bindgen_test::console_log!("Document length: {}", document.len());
    }
}

// Next token helpers
fn next_token(document: &str, tokenizer: &Regex, last_offset: usize) -> (String, String, HashMap<String, Value>, usize, usize) {
    if let Some(caps) = tokenizer.captures(&document[last_offset..]) {
        let token_start = last_offset;
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

fn add_freeform(document: &str, output: &mut Vec<Block>, offset: usize) {
    // Check if the offset is within bounds
    if offset > document.len() {
        // Handle the error or log it as needed
        wasm_bindgen_test::console_log!("Error: Offset out of bounds");
        wasm_bindgen_test::console_log!("Offset: {}, Document length: {}", offset, document.len());
        return;
    }

    // Use Block::new or your equivalent constructor method
    // Ensure the slice is within the bounds of the document
    output.push(Block::new(&document[offset..offset]));
}
