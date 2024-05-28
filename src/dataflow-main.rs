use std::collections::HashMap;
use std::process::exit;

use anyhow::Result;
use derive_builder::Builder;
use tree_sitter::{Node, Parser};

use crate::dataflow::java::build_graph;

mod dataflow;

const SOURCE_QUERY: &str = r#"(method_declaration
    name: (identifier) @name
    parameters: (formal_parameters

        (formal_parameter
            type: (type_identifier) @type
            name: (identifier)
        )
    )
    (#eq? @type "HttpServletRequest")
    (#any-of? @name "doGet" "doPost" "doPatch")
)"#;


#[derive(Clone, Debug, Builder)]
pub struct MatchNode<'node> {
    pub captures: HashMap<String, Node<'node>>,
}

fn get_query(query_code: &str, language: &tree_sitter::Language) -> Result<tree_sitter::Query> {
    Ok(tree_sitter::Query::new(&language, query_code)?)
}

fn get_query_nodes<'tree>(tree: &'tree tree_sitter::Tree, query: &tree_sitter::Query, code: &str) -> Vec<MatchNode<'tree>> {
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let mut matches = Vec::new();
    let query_result = query_cursor.matches(query, tree.root_node(), code.as_bytes());

    for query_match in query_result {
        let mut captures: MatchNode = MatchNode {
            captures: HashMap::new(),

        };

        for capture in query_match.captures.iter() {
            let capture_name_opt = query
                .capture_names()
                .get(usize::try_from(capture.index).unwrap());

            if let Some(capture_name) = capture_name_opt {
                captures.captures.insert(capture_name.to_string(), capture.node.clone());
            }
        }

        matches.push(captures);
    }

    return matches;
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];

    // read filename into a string
    let source_code = std::fs::read_to_string(filename).expect("error while reading file");


    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_java::language()).expect("error while loading Java language");
    let tree = parser.parse(&source_code, None).expect("error while parsing source code");
    let code_str = source_code.as_str();
    build_graph(&tree, code_str);


    let source_query = get_query(SOURCE_QUERY, &tree_sitter_java::language()).expect("get source query");
    let nodes = get_query_nodes(&tree, &source_query, code_str);

    if nodes.len() == 0 {
        println!("no node");
        exit(1);
    }

    let nodes_len = nodes.len();
    println!("Found {} matches", nodes_len);
}
