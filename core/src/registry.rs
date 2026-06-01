use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ElementDefinition {
    pub kind: String,
    pub plantuml: PlantUmlShape,
    pub label: String,
}

#[derive(Debug, Clone)]
pub enum PlantUmlShape {
    Actor,
    Component,
    Database,
    Queue,
    Storage,
    Cloud,
    Rectangle,
}

#[derive(Debug, Clone)]
pub struct ElementRegistry {
    definitions: HashMap<String, ElementDefinition>,
}

impl ElementRegistry {
    pub fn standard() -> Self {
        let mut registry = Self {
            definitions: HashMap::new(),
        };

        registry.register("actor", PlantUmlShape::Actor, "Actor");
        registry.register("user", PlantUmlShape::Actor, "User");
        registry.register("team", PlantUmlShape::Actor, "Team");

        registry.register("system", PlantUmlShape::Component, "System");
        registry.register("application", PlantUmlShape::Component, "Application");
        registry.register("frontend", PlantUmlShape::Component, "Frontend");
        registry.register("backend", PlantUmlShape::Component, "Backend");
        registry.register("service", PlantUmlShape::Component, "Service");
        registry.register("microservice", PlantUmlShape::Component, "Microservice");
        registry.register("api", PlantUmlShape::Component, "API");
        registry.register("gateway", PlantUmlShape::Component, "Gateway");
        registry.register("worker", PlantUmlShape::Component, "Worker");
        registry.register("cron", PlantUmlShape::Component, "Cron Job");
        registry.register("mobile_app", PlantUmlShape::Component, "Mobile App");
        registry.register("desktop_app", PlantUmlShape::Component, "Desktop App");
        registry.register("library", PlantUmlShape::Component, "Library");

        registry.register("database", PlantUmlShape::Database, "Database");
        registry.register("cache", PlantUmlShape::Database, "Cache");
        registry.register("search_index", PlantUmlShape::Database, "Search Index");
        registry.register("vector_db", PlantUmlShape::Database, "Vector Database");

        registry.register("queue", PlantUmlShape::Queue, "Queue");
        registry.register("event_bus", PlantUmlShape::Queue, "Event Bus");
        registry.register("topic", PlantUmlShape::Queue, "Topic");
        registry.register("message_broker", PlantUmlShape::Queue, "Message Broker");

        registry.register("storage", PlantUmlShape::Storage, "Storage");
        registry.register("bucket", PlantUmlShape::Storage, "Bucket");
        registry.register("file_storage", PlantUmlShape::Storage, "File Storage");
        registry.register("object_storage", PlantUmlShape::Storage, "Object Storage");

        registry.register("cloud", PlantUmlShape::Cloud, "Cloud");
        registry.register("cdn", PlantUmlShape::Cloud, "CDN");

        registry.register("external", PlantUmlShape::Rectangle, "External System");
        registry.register("external_api", PlantUmlShape::Rectangle, "External API");
        registry.register("identity_provider", PlantUmlShape::Rectangle, "Identity Provider");
        registry.register("monitoring", PlantUmlShape::Rectangle, "Monitoring");
        registry.register("logging", PlantUmlShape::Rectangle, "Logging");
        registry.register("tracing", PlantUmlShape::Rectangle, "Tracing");

        registry
    }

    pub fn get(&self, kind: &str) -> Option<&ElementDefinition> {
        self.definitions.get(kind)
    }

    pub fn contains(&self, kind: &str) -> bool {
        self.definitions.contains_key(kind)
    }

    fn register(&mut self, kind: &str, plantuml: PlantUmlShape, label: &str) {
        self.definitions.insert(
            kind.to_string(),
            ElementDefinition {
                kind: kind.to_string(),
                plantuml,
                label: label.to_string(),
            },
        );
    }
}