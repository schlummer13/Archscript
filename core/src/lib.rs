// core/src/lib.rs
pub mod diagnostics;
pub mod generators;
pub mod model;
pub mod parser;
pub mod registry;
pub mod rules;
pub mod validator;


use diagnostics::Diagnostic;
use model::Project;
use serde::Serialize;
use rules::RuleViolation;

#[derive(Debug, Serialize)]
pub struct CompileResult {
    pub success: bool,
    pub project: Option<Project>,
    pub plantuml: Option<String>,
    pub documentation: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
    pub violations: Vec<RuleViolation>,
}

pub fn compile(source: &str) -> CompileResult {
    let parse_result = parser::parse(source);

    if !parse_result.diagnostics.is_empty() {
        return CompileResult {
            success: false,
            project: None,
            plantuml: None,
            documentation: None,
            diagnostics: parse_result.diagnostics,
            violations: vec![],
        };
    }

    let project = parse_result.project;
    let diagnostics = validator::validate(&project);

    if !diagnostics.is_empty() {
        return CompileResult {
            success: false,
            project: Some(project),
            plantuml: None,
            documentation: None,
            diagnostics,
            violations: vec![],
        };
    }

    let violations = rules::evaluate(&project);
    let plantuml = generators::plantuml::generate(&project);
    let documentation = generators::markdown::generate(&project);

    CompileResult {
        success: true,
        project: Some(project),
        plantuml: Some(plantuml),
        documentation: Some(documentation),
        diagnostics: vec![],
        violations,
    }
}

#[cfg(test)]
mod tests {
    use super::compile;

    #[test]
    fn compiles_basic_project() {
        let source = r#"
project SolagoPortal

element actor Customer
element frontend WebApp
element backend API
element database PostgreSQL

relation Customer -> WebApp : uses
relation WebApp -> API : REST
relation API -> PostgreSQL : reads/writes
"#;

        let result = compile(source);

        assert!(result.success);
        assert!(result.plantuml.is_some());

        let plantuml = result.plantuml.unwrap();

        assert!(plantuml.contains("@startuml"));
        assert!(plantuml.contains("title SolagoPortal"));
        assert!(plantuml.contains("actor \"Customer\" as Customer"));
        assert!(plantuml.contains("component \"WebApp\" as WebApp"));
        assert!(plantuml.contains("database \"PostgreSQL\" as PostgreSQL"));
        assert!(plantuml.contains("WebApp --> API : REST"));
        assert!(plantuml.contains("@enduml"));
    }

    #[test]
    fn fails_on_unknown_statement() {
        let source = r#"
project Demo

foo bar
"#;

        let result = compile(source);

        assert!(!result.success);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0]
            .message
            .contains("Unknown statement"));
    }

    #[test]
    fn fails_on_unknown_relation_target() {
        let source = r#"
project Demo

element frontend WebApp

relation WebApp -> API : REST
"#;

        let result = compile(source);

        assert!(!result.success);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0]
            .message
            .contains("Unknown relation target"));
    }

    #[test]
    fn fails_on_duplicate_element() {
        let source = r#"
project Demo

element frontend WebApp
element backend WebApp
"#;

        let result = compile(source);

        assert!(!result.success);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0]
            .message
            .contains("Duplicate element"));
    }

    #[test]
    fn compiles_element_with_properties() {
        let source = r#"
project Demo

element frontend WebApp {
  tech "React"
  owner "Frontend Team"
  description "Customer Portal"
}
"#;

        let result = compile(source);

        assert!(result.success);

        let project = result.project.unwrap();
        let element = &project.elements[0];

        assert_eq!(element.properties.get("tech").unwrap(), "React");
        assert_eq!(element.properties.get("owner").unwrap(), "Frontend Team");
        assert_eq!(
            element.properties.get("description").unwrap(),
            "Customer Portal"
        );
    }

#[test]
fn compiles_boundaries() {
    let source = r#"
project SolagoPortal

boundary Public {
  element actor Customer
  element frontend WebApp
}

boundary Internal {
  element backend API
  element database PostgreSQL
}

relation Customer -> WebApp : uses
relation WebApp -> API : REST
relation API -> PostgreSQL : reads/writes
"#;

    let result = compile(source);

    assert!(result.success);

    let plantuml = result.plantuml.unwrap();

    assert!(plantuml.contains("package \"Boundary: Public\""));
    assert!(plantuml.contains("actor \"Customer\" as Customer"));
    assert!(plantuml.contains("component \"WebApp\" as WebApp"));

    assert!(plantuml.contains("package \"Boundary: Internal\""));
    assert!(plantuml.contains("component \"API\" as API"));
    assert!(plantuml.contains("database \"PostgreSQL\" as PostgreSQL"));

    assert!(plantuml.contains("WebApp --> API : REST"));
}

#[test]
fn fails_on_unclosed_boundary() {
    let source = r#"
project Demo

boundary Internal {
  element backend API
"#;

    let result = compile(source);

    assert!(!result.success);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0]
        .message
        .contains("Unclosed boundary block"));
}

#[test]
fn compiles_registry_element_types() {
    let source = r#"
project Demo

element vector_db Embeddings
element event_bus EventBus
element identity_provider Keycloak

relation Keycloak -> EventBus : emits
relation EventBus -> Embeddings : indexes
"#;

    let result = compile(source);

    assert!(result.success);

    let plantuml = result.plantuml.unwrap();

    assert!(plantuml.contains("database \"Embeddings\" as Embeddings"));
    assert!(plantuml.contains("queue \"EventBus\" as EventBus"));
    assert!(plantuml.contains("rectangle \"Keycloak\" as Keycloak"));
}

#[test]
fn fails_on_unknown_element_type() {
    let source = r#"
project Demo

element spaceship MillenniumFalcon
"#;

    let result = compile(source);

    assert!(!result.success);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0]
        .message
        .contains("Unknown element type"));
}

#[test]
fn compiles_relation_with_properties() {
    let source = r#"
project Demo

element frontend WebApp
element backend API

relation WebApp -> API {
  label "REST"
  protocol "HTTPS"
  description "Frontend calls the backend API"
}
"#;

    let result = compile(source);

    assert!(result.success);

    let project = result.project.unwrap();
    let relation = &project.relations[0];

    assert_eq!(relation.from, "WebApp");
    assert_eq!(relation.to, "API");
    assert_eq!(relation.properties.get("label").unwrap(), "REST");
    assert_eq!(relation.properties.get("protocol").unwrap(), "HTTPS");
    assert_eq!(
        relation.properties.get("description").unwrap(),
        "Frontend calls the backend API"
    );
}

#[test]
fn relation_properties_generate_label() {
    let source = r#"
project Demo

element frontend WebApp
element backend API

relation WebApp -> API {
  protocol "GraphQL"
}
"#;

    let result = compile(source);

    assert!(result.success);

    let plantuml = result.plantuml.unwrap();

    assert!(plantuml.contains("WebApp --> API : GraphQL"));
}

#[test]
fn fails_on_unclosed_relation_block() {
    let source = r#"
project Demo

element frontend WebApp
element backend API

relation WebApp -> API {
  protocol "REST"
"#;

    let result = compile(source);

    assert!(!result.success);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0]
        .message
        .contains("Unclosed relation block"));
}

#[test]
fn compiles_element_with_explicit_id() {
    let source = r#"
project Demo

element backend "Customer API" as api
"#;

    let result = compile(source);

    assert!(result.success);

    let project = result.project.unwrap();
    let element = &project.elements[0];

    assert_eq!(element.id, "api");
    assert_eq!(element.name, "Customer API");
    assert_eq!(element.kind, "backend");
}

#[test]
fn relation_uses_element_ids() {
    let source = r#"
project Demo

element frontend "Web App" as webapp
element backend "Customer API" as api

relation webapp -> api : REST
"#;

    let result = compile(source);

    assert!(result.success);

    let plantuml = result.plantuml.unwrap();

    assert!(plantuml.contains("component \"Web App\" as webapp"));
    assert!(plantuml.contains("component \"Customer API\" as api"));
    assert!(plantuml.contains("webapp --> api : REST"));
}

#[test]
fn compiles_element_tags() {
    let source = r#"
project Demo

element backend API {
  tags [critical, production, gdpr]
}
"#;

    let result = compile(source);

    assert!(result.success);

    let project = result.project.unwrap();
    let element = &project.elements[0];

    assert_eq!(element.tags, vec!["critical", "production", "gdpr"]);
}

#[test]
fn compiles_relation_tags() {
    let source = r#"
project Demo

element frontend WebApp
element backend API

relation WebApp -> API {
  protocol "REST"
  tags [sync, public]
}
"#;

    let result = compile(source);

    assert!(result.success);

    let project = result.project.unwrap();
    let relation = &project.relations[0];

    assert_eq!(relation.tags, vec!["sync", "public"]);
}

#[test]
fn generates_markdown_documentation() {
    let source = r#"
project Demo

element backend "Customer API" as api {
  tech "Laravel"
  owner "Backend Team"
  tags [critical, production]
  description "Handles customer requests"
}
"#;

    let result = compile(source);

    assert!(result.success);
    assert!(result.documentation.is_some());

    let documentation = result.documentation.unwrap();

    assert!(documentation.contains("# Demo Architecture"));
    assert!(documentation.contains("### Customer API"));
    assert!(documentation.contains("- ID: `api`"));
    assert!(documentation.contains("- Tags: `critical`, `production`"));
    assert!(documentation.contains("- Tech: Laravel"));
}

#[test]
fn compiles_domain() {
    let source = r#"
project Demo

domain Billing {
  element service "Billing Service" as billing_service
  element database "Billing Database" as billing_db
}

relation billing_service -> billing_db : PostgreSQL
"#;

    let result = compile(source);

    assert!(result.success);

    let plantuml = result.plantuml.unwrap();

    assert!(plantuml.contains("package \"Domain: Billing\""));
    assert!(plantuml.contains("component \"Billing Service\" as billing_service"));
    assert!(plantuml.contains("database \"Billing Database\" as billing_db"));
    assert!(plantuml.contains("billing_service --> billing_db : PostgreSQL"));
}

#[test]
fn compiles_environment() {
    let source = r#"
project Demo

environment Production {
  element frontend "Web App" as prod_webapp
  element backend "API" as prod_api
}

relation prod_webapp -> prod_api : HTTPS
"#;

    let result = compile(source);

    assert!(result.success);

    let plantuml = result.plantuml.unwrap();

    assert!(plantuml.contains("package \"Environment: Production\""));
    assert!(plantuml.contains("component \"Web App\" as prod_webapp"));
    assert!(plantuml.contains("component \"API\" as prod_api"));
    assert!(plantuml.contains("prod_webapp --> prod_api : HTTPS"));
}

#[test]
fn fails_on_unclosed_domain() {
    let source = r#"
project Demo

domain Billing {
  element service BillingService
"#;

    let result = compile(source);

    assert!(!result.success);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0]
        .message
        .contains("Unclosed domain block"));
}

#[test]
fn fails_on_unclosed_environment() {
    let source = r#"
project Demo

environment Production {
  element backend API
"#;

    let result = compile(source);

    assert!(!result.success);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0]
        .message
        .contains("Unclosed environment block"));
}

#[test]
fn documentation_includes_domain_and_environment() {
    let source = r#"
project Demo

domain Billing {
  element service "Billing Service" as billing_service
}

environment Production {
  element backend "Production API" as prod_api
}
"#;

    let result = compile(source);

    assert!(result.success);

    let documentation = result.documentation.unwrap();

    assert!(documentation.contains("## Domain: Billing"));
    assert!(documentation.contains("- Domain: `Billing`"));
    assert!(documentation.contains("## Environment: Production"));
    assert!(documentation.contains("- Environment: `Production`"));
}

#[test]
fn rule_detects_frontend_to_database() {
    let source = r#"
project Demo

element frontend WebApp
element database PostgreSQL

relation WebApp -> PostgreSQL : reads
"#;

    let result = compile(source);

    assert!(result.success);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].rule, "no_frontend_to_database");
    assert!(result.violations[0]
        .message
        .contains("may not directly access database"));
}

#[test]
fn rule_detects_critical_without_owner() {
    let source = r#"
project Demo

element service Billing {
  tags [critical]
}
"#;

    let result = compile(source);

    assert!(result.success);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].rule, "critical_requires_owner");
    assert!(result.violations[0]
        .message
        .contains("requires owner property"));
}

#[test]
fn rule_detects_production_without_monitoring() {
    let source = r#"
project Demo

element service Billing {
  owner "Platform Team"
  tags [production]
}
"#;

    let result = compile(source);

    assert!(result.success);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].rule, "production_requires_monitoring");
    assert!(result.violations[0]
        .message
        .contains("requires monitoring property"));
}

#[test]
fn rules_pass_when_architecture_is_valid() {
    let source = r#"
project Demo

element frontend WebApp {
  owner "Frontend Team"
  monitoring "Grafana"
  tags [production]
}

element service API {
  owner "Platform Team"
  monitoring "Grafana"
  tags [critical, production]
}

element database PostgreSQL {
  owner "Database Team"
  monitoring "Grafana"
  tags [production]
}

relation WebApp -> API : REST
relation API -> PostgreSQL : PostgreSQL
"#;

    let result = compile(source);

    assert!(result.success);
    assert_eq!(result.violations.len(), 0);
}

}