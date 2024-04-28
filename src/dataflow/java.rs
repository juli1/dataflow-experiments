use crate::dataflow::common::{contains_identifier, get_code_for_node};
use std::collections::HashMap;
use tree_sitter;
use tree_sitter::{Tree, TreeCursor};

pub struct DataflowNode<'node> {
    pub node: tree_sitter::Node<'node>,
}

pub struct DataflowMap {
    pub map: HashMap<String, DataflowMap>,
}

struct WalkContext<'a> {
    code: &'a str,
}

fn walk_method_declaration_content(node: &tree_sitter::Node, context: &WalkContext) {
    if node.grammar_name() == "assignment_expression" {
        let left_opt = node.child_by_field_name("left");
        let right_opt = node.child_by_field_name("right");

        if left_opt.is_none() && right_opt.is_none() {
            return;
        }

        let left = left_opt.unwrap();
        println!("left type {}", left.grammar_name());
        if left.grammar_name() != "identifier" {
            return;
        }

        let left_identifier = get_code_for_node(&left, context.code);
        let contains =
            contains_identifier(&right_opt.unwrap(), &"request".to_string(), context.code);

        if contains {
            println!("assignment in {}", left_identifier);
        }
    }

    let mut cursor = node.walk();
    let children = node.children(&mut cursor);
    for child in children {
        if child.is_named() {
            // println!("[walk_method_declaration] type: {}", child.grammar_name());
            walk_method_declaration_content(&child, context);
        }
    }
}

fn walk_method_declaration(node: &tree_sitter::Node, context: &WalkContext) {
    let method_name_opt = node
        .child_by_field_name("name")
        .map(|n| get_code_for_node(&n, context.code));

    if method_name_opt.is_none() {
        return;
    }

    let method_name = method_name_opt.unwrap();

    let parameters_opt = node.child_by_field_name("parameters");

    if let Some(parameters) = parameters_opt {
        let mut cursor = parameters.walk();
        let children = parameters.children(&mut cursor);
        for child in children {
            if child.is_named() {
                if child.grammar_name() == "formal_parameter" {
                    let name_opt = child.child_by_field_name("name");
                    if let Some(name) = name_opt {
                        let parameter_name = get_code_for_node(&name, context.code);
                        println!(
                            "method: {}, parameter name: {}",
                            method_name, parameter_name
                        );
                    }
                }

                // println!("[walk_method_declaration] type: {}", child.grammar_name());
                walk_method_declaration(&child, context);
            }
        }
    }

    walk_method_declaration_content(&node, context);
}

fn walk_node_class(node: &tree_sitter::Node, context: &WalkContext) {
    if node.grammar_name() == "method_declaration" {
        return walk_method_declaration(node, context);
    }

    let mut cursor = node.walk();
    let children = node.children(&mut cursor);
    for child in children {
        if child.is_named() {
            // println!("[walk_node_class] type: {}", child.grammar_name());
            walk_node_class(&child, context);
        }
    }
}

pub fn walk_node(node: &tree_sitter::Node, context: &WalkContext) {
    if node.grammar_name() == "class_declaration" {
        return walk_node_class(node, context);
    }

    let mut cursor = node.walk();
    let children = node.children(&mut cursor);
    for child in children {
        if child.is_named() {
            // println!("[walk_node] type: {}", child.grammar_name());
            walk_node(&child, context);
        }
    }
}

pub fn build_graph(tree: &Tree, code: &str) {
    let context = WalkContext { code };
    walk_node(&tree.root_node(), &context);
}
