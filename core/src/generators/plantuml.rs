use crate::model::{Element, Project};
use crate::registry::{ElementRegistry, PlantUmlShape};

pub fn generate(project: &Project) -> String {
    let registry = ElementRegistry::standard();
    generate_with_registry(project, &registry)
}

pub fn generate_with_registry(project: &Project, registry: &ElementRegistry) -> String {
    let mut output = String::new();

    output.push_str("@startuml\n");
    output.push_str(&format!("title {}\n\n", project.name));

    for element in &project.elements {
        output.push_str(&format_element(element, registry));
        output.push('\n');
    }

    if !project.elements.is_empty() {
        output.push('\n');
    }

    for boundary in &project.boundaries {
        output.push_str(&format!("package \"Boundary: {}\" {{\n", boundary.name));

        for element in &boundary.elements {
            output.push_str("  ");
            output.push_str(&format_element(element, registry));
            output.push('\n');
        }

        output.push_str("}\n\n");
    }

    for domain in &project.domains {
        output.push_str(&format!("package \"Domain: {}\" {{\n", domain.name));

        for element in &domain.elements {
            output.push_str("  ");
            output.push_str(&format_element(element, registry));
            output.push('\n');
        }

        output.push_str("}\n\n");
    }

    for environment in &project.environments {
        output.push_str(&format!("package \"Environment: {}\" {{\n", environment.name));

        for element in &environment.elements {
            output.push_str("  ");
            output.push_str(&format_element(element, registry));
            output.push('\n');
        }

        output.push_str("}\n\n");
    }

    for relation in &project.relations {
        let label = relation
            .label
            .clone()
            .or_else(|| relation.properties.get("label").cloned())
            .or_else(|| relation.properties.get("protocol").cloned());

        match label {
            Some(label) => {
                output.push_str(&format!(
                    "{} --> {} : {}\n",
                    relation.from, relation.to, label
                ));
            }
            None => {
                output.push_str(&format!("{} --> {}\n", relation.from, relation.to));
            }
        }
    }

    output.push_str("\n@enduml\n");
    output
}

fn format_element(element: &Element, registry: &ElementRegistry) -> String {
    let label = match element.properties.get("description") {
        Some(description) => format!("{}\\n{}", element.name, description),
        None => element.name.clone(),
    };

    let shape = registry
        .get(&element.kind)
        .map(|definition| &definition.plantuml)
        .unwrap_or(&PlantUmlShape::Component);

    match shape {
        PlantUmlShape::Actor => format!("actor \"{}\" as {}", label, element.id),
        PlantUmlShape::Component => format!("component \"{}\" as {}", label, element.id),
        PlantUmlShape::Database => format!("database \"{}\" as {}", label, element.id),
        PlantUmlShape::Queue => format!("queue \"{}\" as {}", label, element.id),
        PlantUmlShape::Storage => format!("storage \"{}\" as {}", label, element.id),
        PlantUmlShape::Cloud => format!("cloud \"{}\" as {}", label, element.id),
        PlantUmlShape::Rectangle => format!("rectangle \"{}\" as {}", label, element.id),
    }
}