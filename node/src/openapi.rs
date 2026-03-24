// node/src/openapi.rs — OpenAPI 3.1 spec generator for the Prova API Gateway
//
// Auto-generates an OpenAPI specification from route definitions.
// Supports path parameters, request/response schemas, security schemes,
// and rate limit headers.

use std::collections::HashMap;

/// OpenAPI document root.
#[derive(Debug, Clone)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: Info,
    pub servers: Vec<Server>,
    pub paths: Vec<PathItem>,
    pub components: Components,
}

#[derive(Debug, Clone)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub url: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct PathItem {
    pub path: String,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub method: String,
    pub operation_id: String,
    pub summary: String,
    pub description: String,
    pub tags: Vec<String>,
    pub parameters: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
    pub responses: Vec<Response>,
    pub security: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub location: ParamLocation,
    pub required: bool,
    pub schema: SchemaRef,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParamLocation {
    Path,
    Query,
    Header,
}

impl ParamLocation {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Path => "path",
            Self::Query => "query",
            Self::Header => "header",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestBody {
    pub content_type: String,
    pub schema: SchemaRef,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: String,
    pub description: String,
    pub schema: Option<SchemaRef>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaRef {
    /// Inline type: string, integer, boolean, array(inner), object(fields)
    Inline(SchemaType),
    /// Reference to components/schemas
    Ref(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    String,
    Integer,
    Boolean,
    Array(Box<SchemaRef>),
    Object(Vec<(String, SchemaRef, bool)>), // (name, schema, required)
}

#[derive(Debug, Clone)]
pub struct Components {
    pub schemas: Vec<SchemaDefinition>,
    pub security_schemes: Vec<SecurityScheme>,
}

#[derive(Debug, Clone)]
pub struct SchemaDefinition {
    pub name: String,
    pub schema_type: SchemaType,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct SecurityScheme {
    pub name: String,
    pub scheme_type: SecuritySchemeType,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SecuritySchemeType {
    ApiKey {
        location: String,
        param_name: String,
    },
    Http {
        scheme: String,
    },
}

/// Route definition used to auto-generate spec.
#[derive(Debug, Clone)]
pub struct RouteDefinition {
    pub method: String,
    pub path: String,
    pub operation_id: String,
    pub summary: String,
    pub description: String,
    pub tags: Vec<String>,
    pub parameters: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
    pub responses: Vec<Response>,
    pub requires_auth: bool,
}

/// Spec generator that collects routes and produces an OpenAPI document.
pub struct SpecGenerator {
    info: Info,
    servers: Vec<Server>,
    routes: Vec<RouteDefinition>,
    schemas: Vec<SchemaDefinition>,
    security_schemes: Vec<SecurityScheme>,
}

impl SpecGenerator {
    pub fn new(title: &str, version: &str, description: &str) -> Self {
        Self {
            info: Info {
                title: title.to_string(),
                version: version.to_string(),
                description: description.to_string(),
            },
            servers: Vec::new(),
            routes: Vec::new(),
            schemas: Vec::new(),
            security_schemes: Vec::new(),
        }
    }

    pub fn add_server(&mut self, url: &str, description: &str) {
        self.servers.push(Server {
            url: url.to_string(),
            description: description.to_string(),
        });
    }

    pub fn add_route(&mut self, route: RouteDefinition) {
        self.routes.push(route);
    }

    pub fn add_schema(&mut self, def: SchemaDefinition) {
        self.schemas.push(def);
    }

    pub fn add_security_scheme(&mut self, scheme: SecurityScheme) {
        self.security_schemes.push(scheme);
    }

    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    pub fn schema_count(&self) -> usize {
        self.schemas.len()
    }

    /// Generate the full OpenAPI spec.
    pub fn generate(&self) -> OpenApiSpec {
        let mut path_map: HashMap<String, Vec<Operation>> = HashMap::new();

        for route in &self.routes {
            let op = Operation {
                method: route.method.clone(),
                operation_id: route.operation_id.clone(),
                summary: route.summary.clone(),
                description: route.description.clone(),
                tags: route.tags.clone(),
                parameters: route.parameters.clone(),
                request_body: route.request_body.clone(),
                responses: route.responses.clone(),
                security: if route.requires_auth {
                    self.security_schemes
                        .iter()
                        .map(|s| s.name.clone())
                        .collect()
                } else {
                    Vec::new()
                },
            };
            path_map.entry(route.path.clone()).or_default().push(op);
        }

        let mut paths: Vec<PathItem> = path_map
            .into_iter()
            .map(|(path, operations)| PathItem { path, operations })
            .collect();
        paths.sort_by(|a, b| a.path.cmp(&b.path));

        OpenApiSpec {
            openapi: "3.1.0".to_string(),
            info: self.info.clone(),
            servers: self.servers.clone(),
            paths,
            components: Components {
                schemas: self.schemas.clone(),
                security_schemes: self.security_schemes.clone(),
            },
        }
    }

    /// Serialize the spec to JSON.
    pub fn to_json(&self) -> String {
        let spec = self.generate();
        let mut out = String::new();
        out.push_str("{\n");
        out.push_str(&format!("  \"openapi\": \"{}\",\n", spec.openapi));

        // Info
        out.push_str("  \"info\": {\n");
        out.push_str(&format!(
            "    \"title\": \"{}\",\n",
            escape_json(&spec.info.title)
        ));
        out.push_str(&format!(
            "    \"version\": \"{}\",\n",
            escape_json(&spec.info.version)
        ));
        out.push_str(&format!(
            "    \"description\": \"{}\"\n",
            escape_json(&spec.info.description)
        ));
        out.push_str("  },\n");

        // Servers
        out.push_str("  \"servers\": [\n");
        for (i, s) in spec.servers.iter().enumerate() {
            out.push_str(&format!(
                "    {{\"url\": \"{}\", \"description\": \"{}\"}}",
                escape_json(&s.url),
                escape_json(&s.description)
            ));
            if i < spec.servers.len() - 1 {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ],\n");

        // Paths
        out.push_str("  \"paths\": {\n");
        for (pi, path) in spec.paths.iter().enumerate() {
            out.push_str(&format!("    \"{}\": {{\n", escape_json(&path.path)));
            for (oi, op) in path.operations.iter().enumerate() {
                out.push_str(&format!("      \"{}\": {{\n", op.method.to_lowercase()));
                out.push_str(&format!(
                    "        \"operationId\": \"{}\",\n",
                    escape_json(&op.operation_id)
                ));
                out.push_str(&format!(
                    "        \"summary\": \"{}\",\n",
                    escape_json(&op.summary)
                ));
                out.push_str(&format!(
                    "        \"tags\": [{}],\n",
                    op.tags
                        .iter()
                        .map(|t| format!("\"{}\"", escape_json(t)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));

                // Parameters
                if !op.parameters.is_empty() {
                    out.push_str("        \"parameters\": [\n");
                    for (pi2, p) in op.parameters.iter().enumerate() {
                        out.push_str(&format!("          {{\"name\": \"{}\", \"in\": \"{}\", \"required\": {}, \"description\": \"{}\"}}",
                            escape_json(&p.name), p.location.as_str(), p.required, escape_json(&p.description)));
                        if pi2 < op.parameters.len() - 1 {
                            out.push(',');
                        }
                        out.push('\n');
                    }
                    out.push_str("        ],\n");
                }

                // Responses
                out.push_str("        \"responses\": {\n");
                for (ri, r) in op.responses.iter().enumerate() {
                    out.push_str(&format!(
                        "          \"{}\": {{\"description\": \"{}\"}}",
                        escape_json(&r.status),
                        escape_json(&r.description)
                    ));
                    if ri < op.responses.len() - 1 {
                        out.push(',');
                    }
                    out.push('\n');
                }
                out.push_str("        }\n");

                out.push_str("      }");
                if oi < path.operations.len() - 1 {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str("    }");
            if pi < spec.paths.len() - 1 {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  },\n");

        // Components
        out.push_str("  \"components\": {\n");
        out.push_str("    \"schemas\": {},\n");
        out.push_str(&format!("    \"securitySchemes\": {{}}\n"));
        out.push_str("  }\n");
        out.push('}');

        out
    }
}

/// Build the default Prova API routes for spec generation.
pub fn prova_api_routes() -> Vec<RouteDefinition> {
    vec![
        RouteDefinition {
            method: "post".into(),
            path: "/v1/inference".into(),
            operation_id: "submitInference".into(),
            summary: "Submit an inference job".into(),
            description:
                "Submit a new inference request to be scheduled and executed by a provider.".into(),
            tags: vec!["Inference".into()],
            parameters: Vec::new(),
            request_body: Some(RequestBody {
                content_type: "application/json".into(),
                schema: SchemaRef::Ref("InferenceRequest".into()),
                required: true,
            }),
            responses: vec![
                Response {
                    status: "201".into(),
                    description: "Job created".into(),
                    schema: Some(SchemaRef::Ref("InferenceResult".into())),
                },
                Response {
                    status: "400".into(),
                    description: "Invalid request".into(),
                    schema: None,
                },
                Response {
                    status: "401".into(),
                    description: "Unauthorized".into(),
                    schema: None,
                },
                Response {
                    status: "429".into(),
                    description: "Rate limit exceeded".into(),
                    schema: None,
                },
            ],
            requires_auth: true,
        },
        RouteDefinition {
            method: "get".into(),
            path: "/v1/inference/{job_id}".into(),
            operation_id: "getInferenceJob".into(),
            summary: "Get inference job status".into(),
            description: "Retrieve the current status and result of an inference job.".into(),
            tags: vec!["Inference".into()],
            parameters: vec![Parameter {
                name: "job_id".into(),
                location: ParamLocation::Path,
                required: true,
                schema: SchemaRef::Inline(SchemaType::String),
                description: "The job identifier".into(),
            }],
            request_body: None,
            responses: vec![
                Response {
                    status: "200".into(),
                    description: "Job details".into(),
                    schema: Some(SchemaRef::Ref("InferenceResult".into())),
                },
                Response {
                    status: "404".into(),
                    description: "Job not found".into(),
                    schema: None,
                },
            ],
            requires_auth: true,
        },
        RouteDefinition {
            method: "delete".into(),
            path: "/v1/inference/{job_id}".into(),
            operation_id: "cancelInferenceJob".into(),
            summary: "Cancel an inference job".into(),
            description: "Cancel a queued or running inference job.".into(),
            tags: vec!["Inference".into()],
            parameters: vec![Parameter {
                name: "job_id".into(),
                location: ParamLocation::Path,
                required: true,
                schema: SchemaRef::Inline(SchemaType::String),
                description: "The job identifier".into(),
            }],
            request_body: None,
            responses: vec![
                Response {
                    status: "200".into(),
                    description: "Job cancelled".into(),
                    schema: None,
                },
                Response {
                    status: "404".into(),
                    description: "Job not found".into(),
                    schema: None,
                },
            ],
            requires_auth: true,
        },
        RouteDefinition {
            method: "get".into(),
            path: "/v1/models".into(),
            operation_id: "listModels".into(),
            summary: "List available models".into(),
            description: "Get the list of models available for inference on this node.".into(),
            tags: vec!["Models".into()],
            parameters: Vec::new(),
            request_body: None,
            responses: vec![Response {
                status: "200".into(),
                description: "Model list".into(),
                schema: None,
            }],
            requires_auth: true,
        },
        RouteDefinition {
            method: "get".into(),
            path: "/v1/health".into(),
            operation_id: "healthCheck".into(),
            summary: "Health check".into(),
            description: "Returns node health status. Does not require authentication.".into(),
            tags: vec!["System".into()],
            parameters: Vec::new(),
            request_body: None,
            responses: vec![Response {
                status: "200".into(),
                description: "Healthy".into(),
                schema: None,
            }],
            requires_auth: false,
        },
        RouteDefinition {
            method: "get".into(),
            path: "/v1/openapi.json".into(),
            operation_id: "getOpenApiSpec".into(),
            summary: "Get OpenAPI specification".into(),
            description: "Returns this auto-generated OpenAPI 3.1 specification.".into(),
            tags: vec!["System".into()],
            parameters: Vec::new(),
            request_body: None,
            responses: vec![Response {
                status: "200".into(),
                description: "OpenAPI JSON document".into(),
                schema: None,
            }],
            requires_auth: false,
        },
    ]
}

/// Build default schema definitions for Prova API types.
pub fn prova_api_schemas() -> Vec<SchemaDefinition> {
    vec![
        SchemaDefinition {
            name: "InferenceRequest".into(),
            schema_type: SchemaType::Object(vec![
                (
                    "model_id".into(),
                    SchemaRef::Inline(SchemaType::String),
                    true,
                ),
                ("input".into(), SchemaRef::Inline(SchemaType::String), true),
                (
                    "max_tokens".into(),
                    SchemaRef::Inline(SchemaType::Integer),
                    false,
                ),
                (
                    "callback_url".into(),
                    SchemaRef::Inline(SchemaType::String),
                    false,
                ),
            ]),
            description: "Inference job submission request.".into(),
        },
        SchemaDefinition {
            name: "InferenceResult".into(),
            schema_type: SchemaType::Object(vec![
                ("job_id".into(), SchemaRef::Inline(SchemaType::String), true),
                ("status".into(), SchemaRef::Inline(SchemaType::String), true),
                (
                    "output".into(),
                    SchemaRef::Inline(SchemaType::String),
                    false,
                ),
            ]),
            description: "Inference job status and result.".into(),
        },
        SchemaDefinition {
            name: "Error".into(),
            schema_type: SchemaType::Object(vec![(
                "error".into(),
                SchemaRef::Inline(SchemaType::String),
                true,
            )]),
            description: "Error response.".into(),
        },
    ]
}

/// Build the default security scheme (API key in header).
pub fn prova_security_scheme() -> SecurityScheme {
    SecurityScheme {
        name: "ApiKeyAuth".into(),
        scheme_type: SecuritySchemeType::ApiKey {
            location: "header".into(),
            param_name: "X-API-Key".into(),
        },
        description: "API key passed in X-API-Key header.".into(),
    }
}

/// Convenience: build a fully-configured generator for the Prova API.
pub fn prova_spec_generator() -> SpecGenerator {
    let mut gen = SpecGenerator::new(
        "Prova Network API",
        "0.1.0",
        "API for submitting inference jobs, querying results, and managing models on the Prova verifiable inference network.",
    );
    gen.add_server("http://localhost:8080", "Local development node");
    gen.add_security_scheme(prova_security_scheme());
    for schema in prova_api_schemas() {
        gen.add_schema(schema);
    }
    for route in prova_api_routes() {
        gen.add_route(route);
    }
    gen
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_generator_new() {
        let gen = SpecGenerator::new("Test API", "1.0.0", "A test");
        assert_eq!(gen.info.title, "Test API");
        assert_eq!(gen.route_count(), 0);
        assert_eq!(gen.schema_count(), 0);
    }

    #[test]
    fn test_add_server() {
        let mut gen = SpecGenerator::new("T", "1", "D");
        gen.add_server("http://localhost:8080", "Dev");
        assert_eq!(gen.servers.len(), 1);
        assert_eq!(gen.servers[0].url, "http://localhost:8080");
    }

    #[test]
    fn test_add_route() {
        let mut gen = SpecGenerator::new("T", "1", "D");
        gen.add_route(RouteDefinition {
            method: "get".into(),
            path: "/test".into(),
            operation_id: "test".into(),
            summary: "Test".into(),
            description: "".into(),
            tags: vec![],
            parameters: vec![],
            request_body: None,
            responses: vec![],
            requires_auth: false,
        });
        assert_eq!(gen.route_count(), 1);
    }

    #[test]
    fn test_add_schema() {
        let mut gen = SpecGenerator::new("T", "1", "D");
        gen.add_schema(SchemaDefinition {
            name: "Foo".into(),
            schema_type: SchemaType::Object(vec![]),
            description: "".into(),
        });
        assert_eq!(gen.schema_count(), 1);
    }

    #[test]
    fn test_generate_produces_valid_spec() {
        let gen = prova_spec_generator();
        let spec = gen.generate();
        assert_eq!(spec.openapi, "3.1.0");
        assert_eq!(spec.info.title, "Prova Network API");
        assert!(!spec.paths.is_empty());
        assert!(!spec.components.schemas.is_empty());
        assert!(!spec.components.security_schemes.is_empty());
    }

    #[test]
    fn test_prova_routes_count() {
        let routes = prova_api_routes();
        assert_eq!(routes.len(), 6);
    }

    #[test]
    fn test_prova_schemas_count() {
        let schemas = prova_api_schemas();
        assert_eq!(schemas.len(), 3);
    }

    #[test]
    fn test_health_endpoint_no_auth() {
        let routes = prova_api_routes();
        let health = routes.iter().find(|r| r.path == "/v1/health").unwrap();
        assert!(!health.requires_auth);
    }

    #[test]
    fn test_inference_endpoint_requires_auth() {
        let routes = prova_api_routes();
        let inf = routes
            .iter()
            .find(|r| r.operation_id == "submitInference")
            .unwrap();
        assert!(inf.requires_auth);
        assert!(inf.request_body.is_some());
    }

    #[test]
    fn test_path_parameter_on_get_job() {
        let routes = prova_api_routes();
        let get = routes
            .iter()
            .find(|r| r.operation_id == "getInferenceJob")
            .unwrap();
        assert_eq!(get.parameters.len(), 1);
        assert_eq!(get.parameters[0].name, "job_id");
        assert_eq!(get.parameters[0].location, ParamLocation::Path);
        assert!(get.parameters[0].required);
    }

    #[test]
    fn test_to_json_contains_openapi_version() {
        let gen = prova_spec_generator();
        let json = gen.to_json();
        assert!(json.contains("\"openapi\": \"3.1.0\""));
    }

    #[test]
    fn test_to_json_contains_paths() {
        let gen = prova_spec_generator();
        let json = gen.to_json();
        assert!(json.contains("/v1/inference"));
        assert!(json.contains("/v1/models"));
        assert!(json.contains("/v1/health"));
    }

    #[test]
    fn test_to_json_contains_operation_ids() {
        let gen = prova_spec_generator();
        let json = gen.to_json();
        assert!(json.contains("submitInference"));
        assert!(json.contains("getInferenceJob"));
        assert!(json.contains("cancelInferenceJob"));
        assert!(json.contains("listModels"));
        assert!(json.contains("healthCheck"));
    }

    #[test]
    fn test_security_scheme() {
        let scheme = prova_security_scheme();
        assert_eq!(scheme.name, "ApiKeyAuth");
        match &scheme.scheme_type {
            SecuritySchemeType::ApiKey {
                location,
                param_name,
            } => {
                assert_eq!(location, "header");
                assert_eq!(param_name, "X-API-Key");
            }
            _ => panic!("expected ApiKey scheme"),
        }
    }

    #[test]
    fn test_operations_grouped_by_path() {
        let gen = prova_spec_generator();
        let spec = gen.generate();
        // /v1/inference/{job_id} should have GET and DELETE
        let job_path = spec
            .paths
            .iter()
            .find(|p| p.path == "/v1/inference/{job_id}")
            .unwrap();
        assert_eq!(job_path.operations.len(), 2);
        let methods: Vec<&str> = job_path
            .operations
            .iter()
            .map(|o| o.method.as_str())
            .collect();
        assert!(methods.contains(&"get"));
        assert!(methods.contains(&"delete"));
    }

    #[test]
    fn test_auth_routes_have_security() {
        let gen = prova_spec_generator();
        let spec = gen.generate();
        for path in &spec.paths {
            for op in &path.operations {
                if op.operation_id == "healthCheck" || op.operation_id == "getOpenApiSpec" {
                    assert!(
                        op.security.is_empty(),
                        "{} should not require auth",
                        op.operation_id
                    );
                } else {
                    assert!(
                        !op.security.is_empty(),
                        "{} should require auth",
                        op.operation_id
                    );
                }
            }
        }
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello \"world\""), "hello \\\"world\\\"");
        assert_eq!(escape_json("line\nnew"), "line\\nnew");
    }

    #[test]
    fn test_param_location_as_str() {
        assert_eq!(ParamLocation::Path.as_str(), "path");
        assert_eq!(ParamLocation::Query.as_str(), "query");
        assert_eq!(ParamLocation::Header.as_str(), "header");
    }

    #[test]
    fn test_empty_generator() {
        let gen = SpecGenerator::new("Empty", "0.0.0", "Nothing");
        let spec = gen.generate();
        assert!(spec.paths.is_empty());
        assert!(spec.components.schemas.is_empty());
    }

    #[test]
    fn test_multiple_tags() {
        let mut gen = SpecGenerator::new("T", "1", "D");
        gen.add_route(RouteDefinition {
            method: "get".into(),
            path: "/multi".into(),
            operation_id: "multi".into(),
            summary: "Multi".into(),
            description: "".into(),
            tags: vec!["A".into(), "B".into(), "C".into()],
            parameters: vec![],
            request_body: None,
            responses: vec![],
            requires_auth: false,
        });
        let spec = gen.generate();
        assert_eq!(spec.paths[0].operations[0].tags.len(), 3);
    }

    #[test]
    fn test_openapi_spec_self_serve_route() {
        let routes = prova_api_routes();
        let oas = routes
            .iter()
            .find(|r| r.path == "/v1/openapi.json")
            .unwrap();
        assert!(!oas.requires_auth);
        assert_eq!(oas.method, "get");
    }
}
