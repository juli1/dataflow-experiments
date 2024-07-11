use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::{Duration, Instant};
use anyhow::Result;
use derive_builder::Builder;
use tree_sitter::{Node, Parser};
use walkdir::WalkDir;
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

pub fn get_files(
    directory: &str,
) -> Result<Vec<PathBuf>> {
    let mut files_to_return: Vec<PathBuf> = vec![];

    // This is the directory that contains the .git files, we do not need to keep them.
    let git_directory = format!("{}/.git", &directory);

    let directories_to_walk: Vec<String> = vec![directory.to_string()];

    for directory_to_walk in directories_to_walk {
        for entry in WalkDir::new(directory_to_walk.as_str()) {
            let dir_entry = entry?;
            let entry = dir_entry.path();

            // we only include if this is a file and not a symlink
            // we should NEVER follow symlink for security reason (an attacker could then
            // attempt to add a symlink outside the repo and read content outside of the
            // repo with a custom rule.
            let mut should_include = entry.is_file() && !entry.is_symlink();
            let path_buf = entry.to_path_buf();


            // do not include the git directory.
            if entry.starts_with(git_directory.as_str()) {
                should_include = false;
            }

            if should_include {
                files_to_return.push(entry.to_path_buf());
            }
        }
    }
    Ok(files_to_return)
}

fn match_extension(path: &Path, extensions: Vec<String>) -> bool {
    match path.extension() {
        Some(ext) => match ext.to_str() {
            Some(e) => extensions.contains(&e.to_string().to_lowercase()),
            None => false,
        },
        None => false,
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let files_in_repository = get_files(args[1].as_str()).expect("");
    let java_files: Vec<PathBuf> = files_in_repository.into_iter().filter(|f| match_extension(f, vec!["java".to_string()])).collect();
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_java::language()).expect("error while loading Java language");
    let mut total_duration = Duration::new(0, 0);
    for f in &java_files {

        // read filename into a string
        let source_code_res = std::fs::read_to_string(f);

        if let Ok(source_code) = source_code_res {

            let tree = parser.parse(&source_code, None).expect("error while parsing source code");
            let code_str = source_code.as_str();

            let now = Instant::now();
            build_graph(&tree, code_str);
            total_duration = total_duration + now.elapsed();
        }



    }


    println!("total time: {} secs", total_duration.as_secs());
    println!("number of files: {}", java_files.len());
}
