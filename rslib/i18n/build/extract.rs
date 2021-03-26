// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::gather::TranslationsByLang;
use fluent_syntax::ast::{Entry, Expression, InlineExpression, Pattern, PatternElement};
use fluent_syntax::parser::Parser;
use serde::Serialize;
use std::{collections::HashSet, fmt::Write};
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Serialize)]
pub struct Module {
    pub name: String,
    pub translations: Vec<Translation>,
    pub index: usize,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Serialize)]
pub struct Translation {
    pub key: String,
    pub text: String,
    pub variables: Vec<Variable>,
    pub index: usize,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Serialize)]
pub struct Variable {
    name: String,
    kind: VariableKind,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Serialize)]
enum VariableKind {
    Int,
    // Float,
    // Number,
    // String,
    Any,
}

pub fn get_modules(data: &TranslationsByLang) -> Vec<Module> {
    let mut output = vec![];

    for (module, text) in &data["templates"] {
        output.push(Module {
            name: module.to_string(),
            translations: extract_metadata(text),
            index: 0,
        });
    }

    output.sort_unstable();

    for (module_idx, module) in output.iter_mut().enumerate() {
        module.index = module_idx;
        for (entry_idx, entry) in module.translations.iter_mut().enumerate() {
            entry.index = entry_idx;
        }
    }

    output
}

fn extract_metadata(ftl_text: &str) -> Vec<Translation> {
    let res = Parser::new(ftl_text).parse().unwrap();
    let mut output = vec![];

    for entry in res.body {
        if let Entry::Message(m) = entry {
            if let Some(pattern) = m.value {
                let mut visitor = Visitor::default();
                visitor.visit_pattern(&pattern);
                let key = m.id.name.to_string();

                let (text, variables) = visitor.into_output();

                output.push(Translation {
                    key,
                    text,
                    variables,
                    index: 0,
                })
            }
        }
    }

    output.sort_unstable();

    output
}

/// Gather variable names and (rough) text from Fluent AST.
#[derive(Default)]
struct Visitor {
    text: String,
    variables: HashSet<String>,
}

impl Visitor {
    fn into_output(self) -> (String, Vec<Variable>) {
        let mut vars: Vec<_> = self.variables.into_iter().map(Into::into).collect();
        vars.sort_unstable();
        (self.text, vars)
    }

    fn visit_pattern(&mut self, pattern: &Pattern<&str>) {
        for element in &pattern.elements {
            match element {
                PatternElement::TextElement { value } => self.text.push_str(value),
                PatternElement::Placeable { expression } => self.visit_expression(expression),
            }
        }
    }

    fn visit_inline_expression(&mut self, expr: &InlineExpression<&str>) {
        match expr {
            InlineExpression::VariableReference { id } => {
                write!(self.text, "{{${}}}", id.name).unwrap();
                self.variables.insert(id.name.to_string());
            }
            InlineExpression::Placeable { expression } => {
                self.visit_expression(expression);
            }
            _ => {}
        }
    }

    fn visit_expression(&mut self, expression: &Expression<&str>) {
        match expression {
            Expression::SelectExpression { selector, variants } => {
                self.visit_inline_expression(&selector);
                self.visit_pattern(&variants.last().unwrap().value)
            }
            Expression::InlineExpression(expr) => self.visit_inline_expression(expr),
        }
    }
}

impl From<String> for Variable {
    fn from(name: String) -> Self {
        let kind = match name.as_str() {
            "cards" | "notes" | "count" | "amount" => VariableKind::Int,
            _ => VariableKind::Any,
        };
        Variable { name, kind }
    }
}