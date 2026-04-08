//! Emitters: render a Shape as JSON Schema / TypeScript / Zod.

pub mod json_schema;
pub mod typescript;
pub mod zod;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    JsonSchema,
    TypeScript,
    Zod,
}

#[derive(Debug, Clone)]
pub struct EmitOptions {
    pub format: Format,
    /// Strict mode: all observed fields become required, no additional
    /// properties allowed, no loose `any` fallbacks.
    pub strict: bool,
    /// Name of the root type (used by TS and Zod emitters).
    pub root_name: String,
}

impl Default for EmitOptions {
    fn default() -> Self {
        Self {
            format: Format::JsonSchema,
            strict: false,
            root_name: "Root".to_string(),
        }
    }
}
