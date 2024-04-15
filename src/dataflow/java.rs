use tree_sitter;

pub struct DataflowNode<'node> {
    pub node: tree_sitter::Node<'node>,
}



pub fn foo() {
    println!("Hello, world!");
}