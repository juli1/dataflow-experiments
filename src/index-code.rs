mod dataflow;

use crate::dataflow::java::build_graph;
use anyhow::Result;
use derive_builder::Builder;
use std::env;
use std::process::exit;
use std::time::Instant;
use std::{collections::HashMap, path::PathBuf};
use tree_sitter::{Node, Parser, Tree};
use walkdir::WalkDir;

struct Parsers {
    java: Parser,
    python: Parser,
    go: Parser,
}

fn initialize_parsers() -> Parsers {
    let mut java_parser = Parser::new();
    java_parser
        .set_language(&tree_sitter_java::language())
        .expect("error while loading Java language");
    let mut go_parser = Parser::new();
    go_parser
        .set_language(&tree_sitter_go::language())
        .expect("error while loading go");
    let mut go_parser = Parser::new();
    go_parser
        .set_language(&tree_sitter_go::language())
        .expect("error while loading go");
    let mut python_parser = Parser::new();
    python_parser
        .set_language(&tree_sitter_python::language())
        .expect("error while loading python");
    Parsers {
        java: java_parser,
        python: python_parser,
        go: go_parser,
    }
}

fn parse_file(path: &PathBuf, parsers: &mut Parsers) -> Option<Tree> {
    // read filename into a string
    let source_code = std::fs::read_to_string(path);

    if source_code.is_err() {
        return None;
    }

    if let Some(ext) = path.extension() {
        if ext == "go" {
            return parsers.go.parse(source_code.unwrap(), None);
        }
        if ext == "java" {
            return parsers.java.parse(source_code.unwrap(), None);
        }
        if ext == "python" {
            return parsers.python.parse(source_code.unwrap(), None);
        }
    }

    return None;
}

pub fn get_files(directory: &str) -> Result<Vec<PathBuf>> {
    let mut files_to_return: Vec<PathBuf> = vec![];

    for entry in WalkDir::new(directory) {
        let dir_entry = entry?;
        let entry = dir_entry.path();

        // we only include if this is a file and not a symlink
        // we should NEVER follow symlink for security reason (an attacker could then
        // attempt to add a symlink outside the repo and read content outside of the
        // repo with a custom rule.
        let mut should_include = entry.is_file() && !entry.is_symlink();
        let path_buf = entry.to_path_buf();

        let relative_path_str = path_buf
            .strip_prefix(directory)
            .ok()
            .and_then(|p| p.to_str())
            .ok_or_else(|| anyhow::Error::msg("should get the path"))?;

        if should_include {
            files_to_return.push(entry.to_path_buf());
        }
    }

    Ok(files_to_return)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let directory = &args[1];
    println!("directory: {}", directory);
    let mut all_trees: Vec<Tree> = Vec::new();
    let mut parsers = initialize_parsers();

    let files = get_files(&directory).expect("cannot get files");

    let start = Instant::now();

    for file in files {
        let tree = parse_file(&file, &mut parsers);
        if tree.is_none() {
            // eprintln!("no tree for file {}", file.to_str().unwrap_or("default"));
        } else {
            // println!("got tree for file {}", file.to_str().unwrap_or("default"));
            all_trees.push(tree.unwrap());
        }
    }

    let elapsed_secs = start.elapsed().as_secs();

    println!(
        "took {} to get {} trees, waiting 200 seconds",
        elapsed_secs,
        all_trees.len()
    );
    std::thread::sleep(std::time::Duration::from_secs(200));
}
