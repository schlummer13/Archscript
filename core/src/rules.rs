use crate::model::{Element, Project, Relation};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct RuleViolation {
    pub rule: String,
    pub element: Option<String>,
    pub relation: Option<String>,
    pub message: String,
}

pub fn evaluate(project: &Project) -> Vec<RuleViolation> {
    let elements = collect_elements(project);

    let mut violations = vec![];

    violations.extend(rule_no_frontend_to_database(project, &elements));
    violations.extend(rule_critical_requires_owner(&elements));
    violations.extend(rule_production_requires_monitoring(&elements));

    violations
}

fn collect_elements(project: &Project) -> HashMap<String, &Element> {
    let mut elements = HashMap::new();

    for element in &project.elements {
        elements.insert(element.id.clone(), element);
    }

    for boundary in &project.boundaries {
        for element in &boundary.elements {
            elements.insert(element.id.clone(), element);
        }
    }

    for domain in &project.domains {
        for element in &domain.elements {
            elements.insert(element.id.clone(), element);
        }
    }

    for environment in &project.environments {
        for element in &environment.elements {
            elements.insert(element.id.clone(), element);
        }
    }

    elements
}

fn rule_no_frontend_to_database(
    project: &Project,
    elements: &HashMap<String, &Element>,
) -> Vec<RuleViolation> {
    let mut violations = vec![];

    for relation in &project.relations {
        let Some(from) = elements.get(&relation.from) else {
            continue;
        };

        let Some(to) = elements.get(&relation.to) else {
            continue;
        };

        if from.kind == "frontend" && is_database_like(to.kind.as_str()) {
            violations.push(RuleViolation {
                rule: "no_frontend_to_database".to_string(),
                element: None,
                relation: Some(format!("{} -> {}", relation.from, relation.to)),
                message: format!(
                    "Frontend '{}' may not directly access database '{}'",
                    from.name, to.name
                ),
            });
        }
    }

    violations
}

fn rule_critical_requires_owner(elements: &HashMap<String, &Element>) -> Vec<RuleViolation> {
    let mut violations = vec![];

    for element in elements.values() {
        if has_tag(element, "critical") && !element.properties.contains_key("owner") {
            violations.push(RuleViolation {
                rule: "critical_requires_owner".to_string(),
                element: Some(element.id.clone()),
                relation: None,
                message: format!("Critical element '{}' requires owner property", element.name),
            });
        }
    }

    violations
}

fn rule_production_requires_monitoring(elements: &HashMap<String, &Element>) -> Vec<RuleViolation> {
    let mut violations = vec![];

    for element in elements.values() {
        if has_tag(element, "production") && !element.properties.contains_key("monitoring") {
            violations.push(RuleViolation {
                rule: "production_requires_monitoring".to_string(),
                element: Some(element.id.clone()),
                relation: None,
                message: format!(
                    "Production element '{}' requires monitoring property",
                    element.name
                ),
            });
        }
    }

    violations
}

fn has_tag(element: &Element, tag: &str) -> bool {
    element.tags.iter().any(|existing| existing == tag)
}

fn is_database_like(kind: &str) -> bool {
    matches!(
        kind,
        "database" | "cache" | "search_index" | "vector_db" | "storage" | "bucket"
    )
}