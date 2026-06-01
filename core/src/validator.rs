use crate::diagnostics::Diagnostic;
use crate::model::{Element, Project};
use crate::registry::ElementRegistry;
use std::collections::HashSet;

pub fn validate(project: &Project) -> Vec<Diagnostic> {
    let registry = ElementRegistry::standard();
    validate_with_registry(project, &registry)
}

pub fn validate_with_registry(project: &Project, registry: &ElementRegistry) -> Vec<Diagnostic> {
    let mut diagnostics = vec![];
    let mut ids = HashSet::new();

    for element in &project.elements {
        validate_element(element, registry, &mut diagnostics);

        if !ids.insert(element.id.clone()) {
            diagnostics.push(Diagnostic {
                line: 0,
                column: 0,
                message: format!("Duplicate element id: {}", element.id),
            });
        }
    }

    for boundary in &project.boundaries {
        for element in &boundary.elements {
            validate_element(element, registry, &mut diagnostics);

            if !ids.insert(element.id.clone()) {
                diagnostics.push(Diagnostic {
                    line: 0,
                    column: 0,
                    message: format!("Duplicate element id: {}", element.id),
                });
            }
        }
    }

    for domain in &project.domains {
        for element in &domain.elements {
            validate_element(element, registry, &mut diagnostics);

            if !ids.insert(element.id.clone()) {
                diagnostics.push(Diagnostic {
                    line: 0,
                    column: 0,
                    message: format!("Duplicate element id: {}", element.id),
                });
            }
        }
    }

    for environment in &project.environments {
        for element in &environment.elements {
            validate_element(element, registry, &mut diagnostics);

            if !ids.insert(element.id.clone()) {
                diagnostics.push(Diagnostic {
                    line: 0,
                    column: 0,
                    message: format!("Duplicate element id: {}", element.id),
                });
            }
        }
    }

    for relation in &project.relations {
        if !ids.contains(&relation.from) {
            diagnostics.push(Diagnostic {
                line: 0,
                column: 0,
                message: format!("Unknown relation source: {}", relation.from),
            });
        }

        if !ids.contains(&relation.to) {
            diagnostics.push(Diagnostic {
                line: 0,
                column: 0,
                message: format!("Unknown relation target: {}", relation.to),
            });
        }
    }

    diagnostics
}

fn validate_element(
    element: &Element,
    registry: &ElementRegistry,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if element.id.trim().is_empty() {
        diagnostics.push(Diagnostic {
            line: 0,
            column: 0,
            message: format!("Missing element id for element '{}'", element.name),
        });
    }

    if !registry.contains(&element.kind) {
        diagnostics.push(Diagnostic {
            line: 0,
            column: 0,
            message: format!(
                "Unknown element type '{}' for element '{}'",
                element.kind, element.name
            ),
        });
    }
}