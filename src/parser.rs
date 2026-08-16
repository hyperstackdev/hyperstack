use anyhow::{Ok, Result, anyhow};
use std::collections::{HashMap, HashSet};
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

const FUNCTION_DEFINITION_QUERY: &str = r#"
(function_declaration name: (identifier) @func_name) @func_body
(variable_declarator name: (identifier) @func_name value: (arrow_function)) @func_body
"#;

const FUNCTION_CALL_QUERY: &str = r#"
(call_expression function: (identifier) @call_target)
"#;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub body: String,
}

pub struct TypeScriptParser {
    parser: Parser,
    func_def_query: Query,
    func_call_query: Query,
}

pub struct SourceCode {
    pub func_defs: Vec<Function>,
    pub func_calls: HashMap<String, HashSet<String>>,
}

impl TypeScriptParser {
    pub fn new() -> Result<Self> {
        let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
        let mut parser = Parser::new();
        let func_def_query = Query::new(&language, FUNCTION_DEFINITION_QUERY)?;
        let func_call_query = Query::new(&language, FUNCTION_CALL_QUERY)?;

        parser
            .set_language(&language)
            .expect("Error loading TypeScript grammar");

        Ok(Self {
            parser,
            func_call_query,
            func_def_query,
        })
    }

    pub fn parse(&mut self, code: String) -> Result<SourceCode> {
        let mut func_defs: Vec<Function> = vec![];
        let mut func_calls: HashMap<String, HashSet<String>> = HashMap::new();
        let tree = self
            .parser
            .parse(code.to_string(), None)
            .ok_or(anyhow!("Failed to parse source code"))?;
        let source_bytes = code.as_bytes();

        let root = tree.root_node();
        let mut root_cursor = QueryCursor::new();
        let mut func_def_matches = root_cursor.matches(&self.func_def_query, root, source_bytes);

        while let Some(func_def_match) = func_def_matches.next() {
            let mut func_name = String::new();
            let mut func_body = None;

            for capture in func_def_match.captures {
                let capture_name = &self.func_def_query.capture_names()[capture.index as usize];
                match capture_name {
                    &"func_name" => func_name = capture.node.utf8_text(source_bytes)?.to_string(),
                    &"func_body" => func_body = Some(capture.node),
                    _ => {}
                }
            }

            if !func_name.is_empty() && func_body.is_some() {
                func_defs.push(Function {
                    name: func_name.to_string(),
                    body: func_body.unwrap().utf8_text(source_bytes)?.to_string(),
                });

                let mut body_cursor = QueryCursor::new();
                let mut func_call_matches =
                    body_cursor.matches(&self.func_call_query, func_body.unwrap(), source_bytes);

                while let Some(func_call_match) = func_call_matches.next() {
                    for capture in func_call_match.captures {
                        let target_name = capture.node.utf8_text(source_bytes)?.to_string();
                        if target_name != func_name {
                            func_calls
                                .entry(func_name.to_string())
                                .or_default()
                                .insert(target_name.to_string());
                        }
                    }
                }
            }
        }

        Ok(SourceCode {
            func_defs,
            func_calls,
        })
    }
}
