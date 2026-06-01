use crate::diagnostics::Diagnostic;
use crate::model::{Boundary, Domain, Element, Environment, Project, Relation};
use std::collections::BTreeMap;

pub struct ParseResult {
    pub project: Project,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(source: &str) -> ParseResult {
    let lines: Vec<&str> = source.lines().collect();

    let mut project = Project {
        name: "Untitled".to_string(),
        elements: vec![],
        boundaries: vec![],
        domains: vec![],
        environments: vec![],
        relations: vec![],
    };

    let mut diagnostics = vec![];
    let mut index = 0;

    while index < lines.len() {
        let line_number = index + 1;
        let line = lines[index].trim();

        if line.is_empty() || line.starts_with('#') {
            index += 1;
            continue;
        }

        if let Some(rest) = line.strip_prefix("project ") {
            project.name = rest.trim().to_string();
            index += 1;
            continue;
        }

        if let Some(rest) = line.strip_prefix("element ") {
            match parse_element(rest, &lines, &mut index, line_number) {
                Ok(element) => project.elements.push(element),
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("boundary ") {
            match parse_named_element_container(rest, &lines, &mut index, line_number, "boundary") {
                Ok(container) => project.boundaries.push(Boundary {
                    name: container.name,
                    elements: container.elements,
                }),
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("domain ") {
            match parse_named_element_container(rest, &lines, &mut index, line_number, "domain") {
                Ok(container) => project.domains.push(Domain {
                    name: container.name,
                    elements: container.elements,
                }),
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("environment ") {
            match parse_named_element_container(rest, &lines, &mut index, line_number, "environment") {
                Ok(container) => project.environments.push(Environment {
                    name: container.name,
                    elements: container.elements,
                }),
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("relation ") {
            match parse_relation(rest, &lines, &mut index, line_number) {
                Ok(relation) => project.relations.push(relation),
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
            continue;
        }

        diagnostics.push(Diagnostic {
            line: line_number,
            column: 1,
            message: format!("Unknown statement: {}", line),
        });

        index += 1;
    }

    ParseResult {
        project,
        diagnostics,
    }
}

struct NamedElementContainer {
    name: String,
    elements: Vec<Element>,
}

fn parse_named_element_container(
    rest: &str,
    lines: &[&str],
    index: &mut usize,
    line_number: usize,
    container_kind: &str,
) -> Result<NamedElementContainer, Diagnostic> {
    if !rest.ends_with('{') {
        *index += 1;
        return Err(Diagnostic {
            line: line_number,
            column: 1,
            message: format!("Expected: {} <name> {{", container_kind),
        });
    }

    let name = rest.trim_end_matches('{').trim();

    if name.is_empty() {
        *index += 1;
        return Err(Diagnostic {
            line: line_number,
            column: 1,
            message: format!("Expected {} name", container_kind),
        });
    }

    let mut container = NamedElementContainer {
        name: name.to_string(),
        elements: vec![],
    };

    *index += 1;

    while *index < lines.len() {
        let current_line_number = *index + 1;
        let line = lines[*index].trim();

        if line.is_empty() || line.starts_with('#') {
            *index += 1;
            continue;
        }

        if line == "}" {
            *index += 1;
            return Ok(container);
        }

        if let Some(rest) = line.strip_prefix("element ") {
            match parse_element(rest, lines, index, current_line_number) {
                Ok(element) => container.elements.push(element),
                Err(diagnostic) => return Err(diagnostic),
            }
            continue;
        }

        return Err(Diagnostic {
            line: current_line_number,
            column: 1,
            message: format!(
                "Only element statements are allowed inside {} blocks",
                container_kind
            ),
        });
    }

    Err(Diagnostic {
        line: line_number,
        column: 1,
        message: format!("Unclosed {} block", container_kind),
    })
}

fn parse_element(
    rest: &str,
    lines: &[&str],
    index: &mut usize,
    line_number: usize,
) -> Result<Element, Diagnostic> {
    let has_block = rest.ends_with('{');
    let header = rest.trim_end_matches('{').trim();
    let tokens = tokenize_header(header);

    if tokens.len() != 2 && tokens.len() != 4 {
        *index += 1;
        return Err(Diagnostic {
            line: line_number,
            column: 1,
            message: "Expected: element <type> <name> or element <type> <name> as <id>"
                .to_string(),
        });
    }

    if tokens.len() == 4 && tokens[2] != "as" {
        *index += 1;
        return Err(Diagnostic {
            line: line_number,
            column: 1,
            message: "Expected: element <type> <name> as <id>".to_string(),
        });
    }

    let kind = tokens[0].clone();
    let name = tokens[1].clone();
    let id = if tokens.len() == 4 {
        tokens[3].clone()
    } else {
        sanitize_identifier(&name)
    };

    let mut element = Element {
        id,
        kind,
        name,
        properties: BTreeMap::new(),
        tags: vec![],
    };

    if !has_block {
        *index += 1;
        return Ok(element);
    }

    *index += 1;

    while *index < lines.len() {
        let current_line_number = *index + 1;
        let line = lines[*index].trim();

        if line.is_empty() || line.starts_with('#') {
            *index += 1;
            continue;
        }

        if line == "}" {
            *index += 1;
            return Ok(element);
        }

        if let Some(tags) = parse_tags(line) {
            element.tags = tags;
            *index += 1;
            continue;
        }

        match parse_property(line) {
            Some((key, value)) => {
                element.properties.insert(key, value);
            }
            None => {
                return Err(Diagnostic {
                    line: current_line_number,
                    column: 1,
                    message: "Expected property: <key> \"<value>\" or tags [a, b]".to_string(),
                });
            }
        }

        *index += 1;
    }

    Err(Diagnostic {
        line: line_number,
        column: 1,
        message: "Unclosed element block".to_string(),
    })
}

fn parse_relation(
    rest: &str,
    lines: &[&str],
    index: &mut usize,
    line_number: usize,
) -> Result<Relation, Diagnostic> {
    let has_block = rest.ends_with('{');
    let header = rest.trim_end_matches('{').trim();

    let (left, label) = match header.split_once(':') {
        Some((relation_part, label_part)) => {
            (relation_part.trim(), Some(label_part.trim().to_string()))
        }
        None => (header.trim(), None),
    };

    let (from, to) = match left.split_once("->") {
        Some((from, to)) => (from.trim().to_string(), to.trim().to_string()),
        None => {
            *index += 1;
            return Err(Diagnostic {
                line: line_number,
                column: 1,
                message: "Expected: relation <from> -> <to> : <label>".to_string(),
            });
        }
    };

    let mut relation = Relation {
        from,
        to,
        label,
        properties: BTreeMap::new(),
        tags: vec![],
    };

    if !has_block {
        *index += 1;
        return Ok(relation);
    }

    *index += 1;

    while *index < lines.len() {
        let current_line_number = *index + 1;
        let line = lines[*index].trim();

        if line.is_empty() || line.starts_with('#') {
            *index += 1;
            continue;
        }

        if line == "}" {
            *index += 1;
            return Ok(relation);
        }

        if let Some(tags) = parse_tags(line) {
            relation.tags = tags;
            *index += 1;
            continue;
        }

        match parse_property(line) {
            Some((key, value)) => {
                relation.properties.insert(key, value);
            }
            None => {
                return Err(Diagnostic {
                    line: current_line_number,
                    column: 1,
                    message: "Expected relation property: <key> \"<value>\" or tags [a, b]"
                        .to_string(),
                });
            }
        }

        *index += 1;
    }

    Err(Diagnostic {
        line: line_number,
        column: 1,
        message: "Unclosed relation block".to_string(),
    })
}

fn parse_property(line: &str) -> Option<(String, String)> {
    let (key, raw_value) = line.split_once(' ')?;
    let value = raw_value.trim().trim_matches('"').to_string();

    Some((key.trim().to_string(), value))
}

fn parse_tags(line: &str) -> Option<Vec<String>> {
    let rest = line.strip_prefix("tags ")?;
    let rest = rest.trim();

    if !rest.starts_with('[') || !rest.ends_with(']') {
        return None;
    }

    let inner = rest.trim_start_matches('[').trim_end_matches(']').trim();

    if inner.is_empty() {
        return Some(vec![]);
    }

    Some(
        inner
            .split(',')
            .map(|tag| tag.trim().trim_matches('"').to_string())
            .filter(|tag| !tag.is_empty())
            .collect(),
    )
}

fn tokenize_header(input: &str) -> Vec<String> {
    let mut tokens = vec![];
    let mut current = String::new();
    let mut in_quotes = false;

    for character in input.chars() {
        match character {
            '"' => {
                in_quotes = !in_quotes;
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(character),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn sanitize_identifier(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_alphanumeric() || *character == '_')
        .collect()
}