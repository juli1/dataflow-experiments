use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use tree_sitter;

const PRINT_INDENTATION: usize = 3;

#[derive(Debug)]
pub enum ContainerKind {
    CLASS,
    FUNCTION,
    FILE,
}

pub struct Container<'a> {
    pub name: Option<String>,
    pub kind: ContainerKind,
    pub containers: Vec<Arc<Container<'a>>>,
    pub nodes: Vec<Arc<Node<'a>>>,
    pub nodes_by_name: HashMap<String, Arc<Node<'a>>>,
    // pub parent: Option<Arc<Container<'a>>>,
}

impl Container<'_> {
    pub fn print(&self, indent: Option<usize>) {
        let name = self.name.clone().unwrap_or("<no name>".to_string());
        let indent = indent.unwrap_or(0);
        println!("{}[container] name={} kind={:?}", " ".repeat(indent), name, self.kind);

        for c in &self.containers {
            c.print(Some(indent + PRINT_INDENTATION))
        }

        for n in &self.nodes {
            n.print(Some(indent + PRINT_INDENTATION))
        }
    }
}

#[derive(Debug)]
pub enum NodeKind {
    PARAMETER,
    VARIABLE,
}

pub struct Node<'a> {
    pub name: Option<String>,
    pub kind: NodeKind,
    pub inbound: Vec<Arc<Node<'a>>>,
    pub outbound: Vec<Arc<Node<'a>>>,
    pub ts_node: Option<Arc<tree_sitter::Node<'a>>>,
    // pub parent: Option<Arc<Container<'a>>>,
}


impl Node<'_> {
    pub fn print(&self, indent: Option<usize>) {
        let name = self.name.clone().unwrap_or("<no name>".to_string());
        let indent = indent.unwrap_or(0);
        println!("{}[node] name={} kind={:?}", " ".repeat(indent), name, self.kind);
        for o in &self.inbound {
            println!("{} <- name={} kind={:?}", " ".repeat(indent + PRINT_INDENTATION), o.name.clone().unwrap_or("no name".to_string()), self.kind)
        }
        for o in &self.outbound {
            println!("{} -> name={} kind={:?}", " ".repeat(indent + PRINT_INDENTATION), o.name.clone().unwrap_or("no name".to_string()), self.kind)
        }
    }
}

pub struct DataFlow<'a> {
    pub containers: Vec<Arc<Container<'a>>>,
    pub ts_node_to_df_node: HashMap<tree_sitter::Node<'a>, Node<'a>>,
}


impl DataFlow<'_> {
    pub fn print_graph(&self) {
        for c in self.containers.iter() {
            c.print(None);
        }
    }
}