use crate::model::{Element, Project, Relation};

pub fn generate(project: &Project) -> String {
    let mut output = String::new();

    output.push_str(&format!("# {} Architecture\n\n", project.name));

    output.push_str("## Elements\n\n");

    for element in &project.elements {
        push_element(&mut output, element, None, None);
    }

    for boundary in &project.boundaries {
        output.push_str(&format!("## Boundary: {}\n\n", boundary.name));

        for element in &boundary.elements {
            push_element(&mut output, element, Some("Boundary"), Some(&boundary.name));
        }
    }

    for domain in &project.domains {
        output.push_str(&format!("## Domain: {}\n\n", domain.name));

        for element in &domain.elements {
            push_element(&mut output, element, Some("Domain"), Some(&domain.name));
        }
    }

    for environment in &project.environments {
        output.push_str(&format!("## Environment: {}\n\n", environment.name));

        for element in &environment.elements {
            push_element(
                &mut output,
                element,
                Some("Environment"),
                Some(&environment.name),
            );
        }
    }

    output.push_str("## Relations\n\n");

    if project.relations.is_empty() {
        output.push_str("_No relations defined._\n");
    }

    for relation in &project.relations {
        push_relation(&mut output, relation);
    }

    output
}

fn push_element(
    output: &mut String,
    element: &Element,
    container_type: Option<&str>,
    container_name: Option<&str>,
) {
    output.push_str(&format!("### {}\n\n", element.name));
    output.push_str(&format!("- ID: `{}`\n", element.id));
    output.push_str(&format!("- Type: `{}`\n", element.kind));

    if let (Some(container_type), Some(container_name)) = (container_type, container_name) {
        output.push_str(&format!("- {}: `{}`\n", container_type, container_name));
    }

    if !element.tags.is_empty() {
        output.push_str(&format!("- Tags: {}\n", format_tags(&element.tags)));
    }

    for (key, value) in &element.properties {
        output.push_str(&format!("- {}: {}\n", title_case(key), value));
    }

    output.push('\n');
}

fn push_relation(output: &mut String, relation: &Relation) {
    output.push_str(&format!("### `{}` -> `{}`\n\n", relation.from, relation.to));

    if let Some(label) = &relation.label {
        output.push_str(&format!("- Label: {}\n", label));
    }

    if !relation.tags.is_empty() {
        output.push_str(&format!("- Tags: {}\n", format_tags(&relation.tags)));
    }

    for (key, value) in &relation.properties {
        output.push_str(&format!("- {}: {}\n", title_case(key), value));
    }

    output.push('\n');
}

fn format_tags(tags: &[String]) -> String {
    tags.iter()
        .map(|tag| format!("`{}`", tag))
        .collect::<Vec<String>>()
        .join(", ")
}

fn title_case(input: &str) -> String {
    let mut characters = input.chars();

    match characters.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), characters.as_str()),
        None => String::new(),
    }
}