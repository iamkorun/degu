//! JSON Schema (draft-07) emitter.

use crate::emit::EmitOptions;
use crate::shape::{ObjectShape, Shape};

pub fn render(shape: &Shape, opts: &EmitOptions) -> String {
    let mut out = String::new();
    out.push_str("{\n  \"$schema\": \"http://json-schema.org/draft-07/schema#\"");
    let body = render_shape(shape, opts, 1);
    if !body.is_empty() {
        out.push_str(",\n");
        out.push_str(&body);
    }
    out.push_str("\n}\n");
    out
}

fn render_shape(shape: &Shape, opts: &EmitOptions, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    let mut lines: Vec<String> = Vec::new();

    // Determine `type` field. If nothing observed and not nullable, skip.
    let mut types: Vec<&str> = Vec::new();
    if shape.scalars.boolean {
        types.push("boolean");
    }
    if shape.scalars.integer {
        types.push("integer");
    }
    if shape.scalars.float {
        types.push("number");
    }
    if shape.scalars.string {
        types.push("string");
    }
    if shape.object.is_some() {
        types.push("object");
    }
    if shape.array.is_some() {
        types.push("array");
    }
    if shape.nullable {
        types.push("null");
    }

    if !types.is_empty() {
        let rendered = if types.len() == 1 {
            format!("\"{}\"", types[0])
        } else {
            let joined: Vec<String> = types.iter().map(|t| format!("\"{}\"", t)).collect();
            format!("[{}]", joined.join(", "))
        };
        lines.push(format!("{pad}\"type\": {rendered}"));
    }

    if let Some(obj) = &shape.object {
        lines.push(render_object(obj, opts, indent));
    }
    if let Some(arr) = &shape.array {
        let inner_body = render_shape(arr, opts, indent + 1);
        let inner = if inner_body.is_empty() {
            format!("{pad}\"items\": {{}}")
        } else {
            format!("{pad}\"items\": {{\n{inner_body}\n{pad}}}")
        };
        lines.push(inner);
    }

    lines.join(",\n")
}

fn render_object(obj: &ObjectShape, opts: &EmitOptions, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    let inner_pad = "  ".repeat(indent + 1);

    let mut prop_lines: Vec<String> = Vec::new();
    let mut required: Vec<String> = Vec::new();
    for (name, field) in &obj.fields {
        let body = render_shape(&field.shape, opts, indent + 2);
        let block = if body.is_empty() {
            format!("{inner_pad}\"{name}\": {{}}")
        } else {
            format!("{inner_pad}\"{name}\": {{\n{body}\n{inner_pad}}}")
        };
        prop_lines.push(block);
        let is_required = opts.strict || field.count == obj.total;
        if is_required {
            required.push(name.clone());
        }
    }

    let mut out = String::new();
    out.push_str(&format!("{pad}\"properties\": {{\n"));
    out.push_str(&prop_lines.join(",\n"));
    out.push_str(&format!("\n{pad}}}"));

    if !required.is_empty() {
        let reqs: Vec<String> = required.iter().map(|r| format!("\"{}\"", r)).collect();
        out.push_str(&format!(",\n{pad}\"required\": [{}]", reqs.join(", ")));
    }
    if opts.strict {
        out.push_str(&format!(",\n{pad}\"additionalProperties\": false"));
    }
    out
}
