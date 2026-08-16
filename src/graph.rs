use anyhow::{Result, anyhow};
use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use std::fmt;

use crate::embedding::VectorStore;
use crate::parser::TypeScriptParser;

#[derive(Debug, Clone)]
pub struct FunctionNode {
    pub index: NodeIndex,
    pub name: String,
    pub body: String,
}

impl fmt::Display for FunctionNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.name, self.body)
    }
}

pub struct CodeGraph {
    graph: DiGraph<String, String>,
    nodes: HashMap<String, FunctionNode>,
    vector_store: VectorStore<FunctionNode>,
    parser: TypeScriptParser,
}

#[allow(dead_code)]
impl CodeGraph {
    pub fn new() -> Result<Self> {
        Ok(Self {
            graph: DiGraph::new(),
            nodes: HashMap::new(),
            vector_store: VectorStore::new(200_000)?,
            parser: TypeScriptParser::new()?,
        })
    }

    pub fn load(&mut self, code: String) -> Result<()> {
        let source = self.parser.parse(code)?;

        for func_def in source.func_defs {
            self.add_node(func_def.name, func_def.body)?;
        }

        for (caller, callees) in source.func_calls {
            for callee in callees {
                self.add_edge(&caller, &callee)?;
            }
        }

        Ok(())
    }

    pub fn add_node(&mut self, name: String, body: String) -> Result<FunctionNode> {
        if !self.nodes.contains_key(&name) {
            let idx = self.graph.add_node(name.to_string());
            let node = FunctionNode {
                index: idx,
                name: name.to_string(),
                body: body.to_string(),
            };
            self.nodes.insert(name.to_string(), node.clone());
            self.vector_store.embed(node)?;
        }

        self.nodes
            .get(&name)
            .cloned()
            .ok_or(anyhow!("Failed to add node {name} to dependency graph"))
    }

    pub fn get_node(&mut self, name: &str) -> Option<FunctionNode> {
        self.nodes.get(name).cloned()
    }

    pub fn add_edge(&mut self, caller: &str, callee: &str) -> Result<()> {
        let caller_node = self
            .get_node(caller)
            .ok_or(anyhow!("Caller node not registered in dependency graph"))?;
        let callee_node = self
            .get_node(callee)
            .ok_or(anyhow!("Callee node not registered in dependency graph"))?;

        self.graph
            .add_edge(caller_node.index, callee_node.index, "CALLS".to_string());

        Ok(())
    }

    fn get_neighbours(&mut self, name: &str, dir: Direction) -> Vec<FunctionNode> {
        if let Some(node) = self.get_node(name) {
            let mut nodes: Vec<FunctionNode> = vec![];

            let func_names = self
                .graph
                .neighbors_directed(node.index, dir)
                .map(|idx| self.graph[idx].clone());

            for func_name in func_names {
                if let Some(node) = self.nodes.get(&func_name) {
                    nodes.push(node.clone());
                }
            }

            nodes
        } else {
            vec![]
        }
    }

    pub fn get_callees(&mut self, name: &str) -> Vec<FunctionNode> {
        self.get_neighbours(name, Direction::Outgoing)
    }

    pub fn get_callers(&mut self, name: &str) -> Vec<FunctionNode> {
        self.get_neighbours(name, Direction::Incoming)
    }

    pub fn search(&mut self, q: String, limit: usize) -> Result<Vec<FunctionNode>> {
        self.vector_store.query(q, limit)
    }
}
