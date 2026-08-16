use crate::{example::TYPESCRIPT_SOURCE_CODE, graph::FunctionNode};
use std::io::{self, Write};

mod embedding;
mod example;
mod graph;
mod parser;

fn nodes_to_string(nodes: Vec<FunctionNode>) -> String {
    nodes
        .iter()
        .map(|n| format!("{}", n.name))
        .collect::<Vec<_>>()
        .join(", ")
}

fn main() {
    let mut code_graph = graph::CodeGraph::new().expect("Failed to create dependency graph");

    code_graph
        .load(TYPESCRIPT_SOURCE_CODE.to_string())
        .expect("Failed to load sample TypeScript code");

    loop {
        let mut query = String::new();

        print!("Enter query: ");
        io::stdout().flush().expect("Failed to flush stdout");
        io::stdin()
            .read_line(&mut query)
            .expect("Failed to readline");

        let nodes = code_graph
            .search(query, 1)
            .expect("Failed to query dependency graph");

        for node in nodes {
            println!("Matching Function: {}", node.name);

            let callers = nodes_to_string(code_graph.get_callers(&node.name));
            println!("Functions that call {}: {}", node.name, callers);

            let callees = nodes_to_string(code_graph.get_callees(&node.name));
            println!("Functions that called by {}: {}", node.name, callees);

            println!("****************\n");
        }
    }
}
