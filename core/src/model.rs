use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub name: String,
    pub elements: Vec<Element>,
    pub boundaries: Vec<Boundary>,
    pub domains: Vec<Domain>,
    pub environments: Vec<Environment>,
    pub relations: Vec<Relation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Boundary {
    pub name: String,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Domain {
    pub name: String,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Environment {
    pub name: String,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Element {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub properties: BTreeMap<String, String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Relation {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub properties: BTreeMap<String, String>,
    pub tags: Vec<String>,
}