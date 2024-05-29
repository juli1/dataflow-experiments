use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tree_sitter;
use tree_sitter::Tree;

use crate::dataflow::common::{get_code_for_node, get_nodes_of_type};
use crate::dataflow::model::{Container, ContainerKind, DataFlow, Node, NodeKind};

struct WalkContext<'a> {
    code: &'a str,
}


fn add_flow(source: &String, dest: &String, container: &mut Container, dataflow: &mut DataFlow, context: &WalkContext) {
    println!("attempt adding flow from {} to {} in container {}", source, dest, container.name.clone().unwrap_or("no name".to_string()));
    let source_opt = container.nodes_by_name.get(source);
    let dest_opt = container.nodes_by_name.get(dest);

    if source == dest {
        return;
    }

    if let (Some(source), Some(dest)) = (source_opt, dest_opt) {
        {
            source.outbound.write().unwrap().push(dest.clone());
        }
        {
            dest.inbound.write().unwrap().push(source.clone());
        }
    }
}

/// Returns all the assignment we need to get when there is a function call
/// on the right hand side of an assignment
pub fn get_identifiers_from_assignment(node: tree_sitter::Node) -> Vec<tree_sitter::Node> {
    let mut res: Vec<tree_sitter::Node> = vec![];
    if node.grammar_name() == "identifier" {
        res.push(node.clone());
    }
    if node.grammar_name() == "method_invocation" {
        let object_opt = node.child_by_field_name("object");
        let arguments_opt = node.child_by_field_name("arguments");
        if let Some(object) = object_opt {
            if object.grammar_name() == "identifier" {
                res.push(object);
            }
        }

        if let Some(arguments) = arguments_opt {
            let mut cursor = node.walk();
            let args = arguments.children(&mut cursor);
            for arg in args {
                res.extend(get_identifiers_from_assignment(arg));

            }
        }
    }

    if node.grammar_name() == "object_creation_expression" {
        let arguments_opt = node.child_by_field_name("arguments");
        
        if let Some(arguments) = arguments_opt {
            let mut cursor = node.walk();
            let args = arguments.children(&mut cursor);
            for arg in args {
                res.extend(get_identifiers_from_assignment(arg));

            }
        }
    }

    if node.grammar_name() == "binary_expression" {
        let left_opt = node.child_by_field_name("left");
        let right_opt = node.child_by_field_name("right");

        if let Some(left) = left_opt {
            res.extend(get_identifiers_from_assignment(left));
        }
        if let Some(right) = right_opt {
            res.extend(get_identifiers_from_assignment(right));
        }
    }

    return res;
}

fn walk_assignment_expression<'a, 'b>(node: tree_sitter::Node<'a>, container: &'b mut Container<'a>, dataflow: &mut DataFlow, context: &WalkContext) {
    let left_opt = node.child_by_field_name("left");
    let right_opt = node.child_by_field_name("right");

    if left_opt.is_none() || right_opt.is_none() {
        return;
    }

    let left = left_opt.unwrap();
    if left.grammar_name() == "identifier" {
        let left_identifier = get_code_for_node(left, context.code);

        let mut variable_node = Arc::new(Node {
            name: Some(left_identifier.clone()),
            kind: NodeKind::VARIABLE,
            inbound: RwLock::new(vec![]),
            outbound: RwLock::new(vec![]),
            ts_node: Arc::new(left),
        });

        if !container.nodes_by_name.contains_key(&left_identifier) {
            container.nodes.push(variable_node.clone());
            container.nodes_by_name.insert(left_identifier.clone(), variable_node.clone());
        }

        let right_identifiers = get_identifiers_from_assignment(right_opt.unwrap());

        for right_identifier in right_identifiers {
            let right_identifier_value = get_code_for_node(right_identifier, context.code);
            add_flow(&right_identifier_value, &left_identifier, container, dataflow, context);
        }
    }
}

fn walk_local_variable_declaration<'a, 'b>(node: tree_sitter::Node<'a>, container: &'b mut Container<'a>, dataflow: &mut DataFlow, context: &WalkContext) {
    let declarator_opt = node.child_by_field_name("declarator");

    if declarator_opt.is_none() {
        return;
    }

    let declarator = declarator_opt.unwrap();

    let left_opt = declarator.child_by_field_name("name");

    let right_opt = declarator.child_by_field_name("value");

    if left_opt.is_none() || right_opt.is_none() {
        return;
    }

    let left = left_opt.unwrap();
    if left.grammar_name() == "identifier" {
        let left_identifier = get_code_for_node(left, context.code);

        let mut variable_node = Arc::new(Node {
            name: Some(left_identifier.clone()),
            kind: NodeKind::VARIABLE,
            inbound: RwLock::new(vec![]),
            outbound: RwLock::new(vec![]),
            ts_node: Arc::new(left),
        });

        if !container.nodes_by_name.contains_key(&left_identifier) {
            container.nodes.push(variable_node.clone());
            container.nodes_by_name.insert(left_identifier.clone(), variable_node.clone());
        }

        let right_identifiers = get_identifiers_from_assignment(right_opt.unwrap());

        for right_identifier in right_identifiers {
            let right_identifier_value = get_code_for_node(right_identifier, context.code);
            add_flow(&right_identifier_value, &left_identifier, container, dataflow, context);
        }
    }
}

fn walk_method_declaration_content<'a, 'b>(node: tree_sitter::Node<'a>, container: &'b mut Container<'a>, dataflow: &mut DataFlow, context: &WalkContext) {
    if node.grammar_name() == "assignment_expression" {
        walk_assignment_expression(node, container, dataflow, context);
        return;
    }

    if node.grammar_name() == "local_variable_declaration" {
        walk_local_variable_declaration(node, container, dataflow, context);
        return;
    }

    if node.grammar_name() == "method_invocation" {
        let object_opt = node.child_by_field_name("object");
        let arguments_opt = node.child_by_field_name("arguments");
        if let Some(object) = object_opt {
            if object.grammar_name() == "identifier" {
                let object_name = get_code_for_node(object, context.code);


                if let Some(arguments) = arguments_opt {
                    let identifiers = get_nodes_of_type(arguments, "identifier");
                    for id in identifiers {
                        let arg_name = get_code_for_node(id, context.code);
                        add_flow(&arg_name, &object_name, container, dataflow, context);

                    }
                }
            }
        }

        return;
    }

    let mut cursor = node.walk();
    let children = node.children(&mut cursor);
    for child in children {
        if child.is_named() {
            // println!("[walk_method_declaration] type: {}", child.grammar_name());
            walk_method_declaration_content(child, container, dataflow, context);
        }
    }
}

fn walk_parameter_declaration<'a, 'b>(node: tree_sitter::Node<'a>, method_container: &'b mut Container<'a>, dataflow: &mut DataFlow, context: &WalkContext) {
    let name_opt = node.child_by_field_name("name");
    if let Some(name) = name_opt {
        let parameter_name = get_code_for_node(name, context.code);
        let mut param_node = Arc::new(Node {
            name: Some(parameter_name.clone()),
            kind: NodeKind::PARAMETER,
            inbound: RwLock::new(vec![]),
            outbound: RwLock::new(vec![]),
            ts_node: Arc::new(node),
        });
        method_container.nodes.push(param_node.clone());
        method_container.nodes_by_name.insert(parameter_name.clone(), param_node.clone());
    }
}

fn walk_method_declaration<'a, 'b>(node: tree_sitter::Node<'a>, class_container: &'b mut Container<'a>, dataflow: &mut DataFlow, context: &WalkContext) {
    let method_name_opt = node
        .child_by_field_name("name")
        .map(|n| get_code_for_node(n, context.code));

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
                    walk_parameter_declaration(child, &mut container, dataflow, context);
                }
            }
        }
    }

    let body_option = node.child_by_field_name("body");
    if let Some(body) = body_option {
        let mut cursor = body.walk();
        let children = body.children(&mut cursor);
        for child in children {
            walk_method_declaration_content(child, &mut container, dataflow, context);
        }
    }


    class_container.containers.push(Arc::new(container));


    // walk_method_declaration_content(&node, context);
}

fn walk_node_class_body<'a, 'b>(node: tree_sitter::Node<'a>, class_container: &'b mut Container<'a>, dataflow: &mut DataFlow, walk_context: &WalkContext) {
    let mut cursor = node.walk();
    let children = node.children(&mut cursor);
    for child in children {
        if child.grammar_name() == "method_declaration" {
            walk_method_declaration(child, class_container, dataflow, walk_context);
        }
    }
}

fn walk_node_class<'a, 'b>(node: tree_sitter::Node<'a>, file_container: &'b mut Container<'a>, dataflow: &mut DataFlow, context: &WalkContext) {
    let name_node = node.child_by_field_name("name");

    if name_node.is_none() {
        return;
    }

    let mut container = Container {
        name: name_node.map(|n| get_code_for_node(n, context.code)),
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
                walk_node_class_body(child, &mut container, dataflow, context);
            }
        }
    }

    file_container.containers.push(Arc::new(container));
}

pub fn walk_root<'a, 'b>(node: tree_sitter::Node<'a>, file_container: &'b mut Container<'a>, dataflow: &mut DataFlow, context: &WalkContext) {
    // println!("[walk_node] node type: {}", node.grammar_name());
    if node.grammar_name() == "class_declaration" {
        return walk_node_class(node, file_container, dataflow, context);
    }

    let mut cursor = node.walk();
    let children = node.children(&mut cursor);
    for child in children {
        if child.is_named() {
            // println!("  [walk_node] child type: {}", child.grammar_name());
            walk_root(child, file_container, dataflow, context);
        }
    }
}

pub fn build_graph(tree: &Tree, code: &str) {
    let context = WalkContext { code };
    let mut dataflow = DataFlow {
        containers: vec![],
        ts_node_to_df_node: HashMap::new(),
    };

    let mut container = Container {
        name: Some("myfile.java".to_string()),
        kind: ContainerKind::FILE,
        containers: vec![],
        nodes: vec![],
        nodes_by_name: HashMap::new(),
    };


    walk_root(tree.root_node(), &mut container, &mut dataflow, &context);
    dataflow.containers.push(Arc::new(container));
    dataflow.print_graph();
}
