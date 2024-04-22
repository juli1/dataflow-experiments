use std::collections::HashMap;
use tree_sitter;
use tree_sitter::{Tree, TreeCursor};

pub fn get_code_for_node<'a>(node: &tree_sitter::Node, code: &'a str) -> String {

    let slice = &code[node.start_byte()..node.end_byte()];
    slice.to_string()
}


pub fn contains_identifier(node: &tree_sitter::Node, node_value: &String, code: &str) -> bool {
    if node.grammar_name() == "identifier" {
        return get_code_for_node(node, code) == *node_value
    }

    let mut cursor = node.walk();
    let children = node.children(&mut cursor);
    for child in children {
        if child.is_named() {
            let r = contains_identifier(&child, node_value, code);
            if r {
                return true
            }
        }
    }

    false
}