//! degu — infer JSON Schema, TypeScript, and Zod from sample JSON.

pub mod emit;
pub mod shape;

pub use emit::{EmitOptions, Format};
pub use shape::Shape;

use anyhow::{Context, Result};
use serde_json::Value;

/// Parse a JSON document from a string.
pub fn parse_json(input: &str) -> Result<Value> {
    serde_json::from_str(input).context("failed to parse JSON input")
}

/// Infer a shape from an iterator of sample JSON values.
///
/// If a top-level value is an array, its elements are treated as individual
/// samples (this is the common "array of records" case). Otherwise the value
/// itself is one sample.
pub fn infer_from_values<I: IntoIterator<Item = Value>>(values: I) -> Shape {
    let mut shape = Shape::default();
    for v in values {
        match v {
            Value::Array(items) if !items.is_empty() => {
                // Treat top-level arrays as a collection of samples if all
                // elements are objects; otherwise treat the whole array as
                // one sample.
                if items.iter().all(|i| i.is_object()) {
                    for item in items {
                        shape.absorb(&item);
                    }
                } else {
                    shape.absorb(&Value::Array(items));
                }
            }
            other => shape.absorb(&other),
        }
    }
    shape
}

/// Render a shape in the requested format.
pub fn render(shape: &Shape, opts: &EmitOptions) -> String {
    match opts.format {
        Format::JsonSchema => emit::json_schema::render(shape, opts),
        Format::TypeScript => emit::typescript::render(shape, opts),
        Format::Zod => emit::zod::render(shape, opts),
    }
}
