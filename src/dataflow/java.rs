use std::collections::HashMap;
use std::sync::Arc;

use tree_sitter;
use tree_sitter::Tree;

use crate::dataflow::common::get_code_for_node;
use crate::dataflow::model::{Container, ContainerKind, DataFlow, Node, NodeKind};

pub struct DataflowNode<'node> {
    pub node: tree_sitter::Node<'node>,
}

pub struct DataflowMap {
    pub map: HashMap<String, DataflowMap>,
}

struct WalkContext<'a> {
    code: &'a str,
}


fn add_flow(source: String, dest: String, container: &mut Container, dataflow: &mut DataFlow, context: &WalkContext) {
    println!("attempt adding flow from {} to {} in container {}", source, dest, container.name.clone().unwrap_or("no name".to_string()))
}

fn walk_method_declaration_content(node: &tree_sitter::Node, container: &mut Container, dataflow: &mut DataFlow, context: &WalkContext) {
    if node.grammar_name() == "assignment_expression" {
        let left_opt = node.child_by_field_name("left");
        let right_opt = node.child_by_field_name("right");

        if left_opt.is_none() && right_opt.is_none() {
            return;
        }

        let left = left_opt.unwrap();
        println!("left type {}", left.grammar_name());
        if left.grammar_name() == "identifier" {
            let left_identifier = get_code_for_node(&left, context.code);

            let mut param_node = Node {
                name: Some(left_identifier),
                kind: NodeKind::VARIABLE,
                inbound: vec![],
                outbound: vec![],
                ts_node: None,
            };
            container.nodes.push(Arc::new(param_node));
        }
    }

    let mut cursor = node.walk();
    let children = node.children(&mut cursor);
    for child in children {
        if child.is_named() {
            // println!("[walk_method_declaration] type: {}", child.grammar_name());
            walk_method_declaration_content(&child, container, dataflow, context);
        }
    }
}

fn walk_parameter_declaration<'a, 'b>(node: &'a tree_sitter::Node, method_container: &'b mut Container, dataflow: &mut DataFlow, context: &WalkContext) {
    let name_opt = node.child_by_field_name("name");
    if let Some(name) = name_opt {
        let parameter_name = get_code_for_node(&name, context.code);
        let mut param_node = Node {
            name: Some(parameter_name),
            kind: NodeKind::PARAMETER,
            inbound: vec![],
            outbound: vec![],
            ts_node: None,
        };
        method_container.nodes.push(Arc::new(param_node));
    }
}

fn walk_method_declaration<'a, 'b>(node: &'a tree_sitter::Node, class_container: &'b mut Container, dataflow: &mut DataFlow, context: &WalkContext) {
    let method_name_opt = node
        .child_by_field_name("name")
        .map(|n| get_code_for_node(&n, context.code));

    if method_name_opt.is_none() {
        return;
    }

    let mut container = Container {
        name: method_name_opt,
        kind: ContainerKind::FUNCTION,
        containers: vec![],
        nodes: vec![],
        nodes_by_name: HashMap::new(),
    };


    let parameters_opt = node.child_by_field_name("parameters");

    if let Some(parameters) = parameters_opt {
        let mut cursor = parameters.walk();
        let children = parameters.children(&mut cursor);
        for child in children {
            if child.is_named() {
                if child.grammar_name() == "formal_parameter" {
                    walk_parameter_declaration(&child, &mut container, dataflow, context);
                }
            }
        }
    }

    let bodyOption = node.child_by_field_name("body");
    if let Some(body) = bodyOption {
        let mut cursor = body.walk();
        let children = body.children(&mut cursor);
        for child in children {
            walk_method_declaration_content(&child, &mut container, dataflow, context);
        }
    }


    class_container.containers.push(Arc::new(container));


    // walk_method_declaration_content(&node, context);
}

fn walk_node_class_body<'a, 'b>(node: &'a tree_sitter::Node, class_container: &'b mut Container, dataflow: &mut DataFlow, walk_context: &WalkContext) {
    let mut cursor = node.walk();
    let children = node.children(&mut cursor);
    for child in children {
        if child.is_named() {
            println!("[walk_node_class_body] child type: {}", child.grammar_name());
        }

        if child.grammar_name() == "method_declaration" {
            walk_method_declaration(&child, class_container, dataflow, walk_context);
        }
    }
}

fn walk_node_class<'a>(node: &'a tree_sitter::Node, dataflow: &mut DataFlow, context: &WalkContext) {
    let nameNode = node.child_by_field_name("name");

    if nameNode.is_none() {
        return;
    }

    let mut container = Container {
        name: nameNode.map(|n| get_code_for_node(&n, context.code)),
        kind: ContainerKind::CLASS,
        containers: vec![],
        nodes: vec![],
        nodes_by_name: HashMap::new(),
    };

    let mut cursor = node.walk();
    let children = node.children(&mut cursor);
    for child in children {
        if child.is_named() {
            if child.grammar_name() == "class_body" {
                walk_node_class_body(&child, &mut container, dataflow, context);
            }
        }
    }

    dataflow.containers.push(Arc::new(container));
}

pub fn walk_node<'a>(node: &'a tree_sitter::Node, dataflow: &mut DataFlow, context: &WalkContext) {
    // println!("[walk_node] node type: {}", node.grammar_name());
    if node.grammar_name() == "class_declaration" {
        return walk_node_class(node, dataflow, context);
    }

    let mut cursor = node.walk();
    let children = node.children(&mut cursor);
    for child in children {
        if child.is_named() {
            // println!("  [walk_node] child type: {}", child.grammar_name());
            walk_node(&child, dataflow, context);
        }
    }
}

pub fn build_graph<'a>(tree: &'a Tree, code: &str) {
    let context = WalkContext { code };
    let mut dataflow = DataFlow {
        containers: vec![],
        ts_node_to_df_node: HashMap::new(),
    };

    walk_node(&tree.root_node(), &mut dataflow, &context);
    dataflow.print_graph();
}
