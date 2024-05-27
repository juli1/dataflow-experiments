use std::collections::HashMap;
use tree_sitter;
use tree_sitter::{Tree, TreeCursor};

pub fn get_code_for_node<'a>(node: &tree_sitter::Node, code: &'a str) -> String {

    let slice = &code[node.start_byte()..node.end_byte()];
    slice.to_string()
}

/// Returns recursively if a node or one of the sub-node contains a given identifier.
/// The [node] is the top tree-sitter node of the tree.
/// [node_value] is the value of the node in the code
/// [code] is the code string
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

// /// Returns all the nodes of a given type through the tree below a given node.
// TODO: uncomment to see lifetime issues.
// fn get_nodes_of_type_rec(node: &tree_sitter::Node, nodeType: &str, acc: &mut Vec<tree_sitter::Node>) {
//
//     if node.grammar_name() == nodeType {
//         // TODO: uncomment to see lifetime issues.
//         //acc.push(node.clone());
//     }
// }
//
// pub fn get_nodes_of_type(node: &tree_sitter::Node, nodeType: &str) -> Vec<tree_sitter::Node> {
//     let mut res: Vec<tree_sitter::Node> = vec![];
//
//     return res;
// }