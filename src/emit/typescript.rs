//! TypeScript interface emitter.

use crate::emit::EmitOptions;
use crate::shape::Shape;

pub fn render(shape: &Shape, opts: &EmitOptions) -> String {
    // Only emit an `interface` for the root if it's an object; otherwise
    // emit a type alias.
    if let Some(obj) = &shape.object {
        if shape.variant_count() == 1 && !shape.nullable {
            let mut out = String::new();
            out.push_str(&format!("export interface {} {{\n", opts.root_name));
            render_object_body(obj, opts, 1, &mut out);
            out.push_str("}\n");
            return out;
        }
    }
    let t = render_type(shape, opts, 0);
    format!("export type {} = {};\n", opts.root_name, t)
}

fn render_object_body(
    obj: &crate::shape::ObjectShape,
    opts: &EmitOptions,
    indent: usize,
    out: &mut String,
) {
    let pad = "  ".repeat(indent);
    for (name, field) in &obj.fields {
        let optional = !opts.strict && field.count < obj.total;
        let q = if optional { "?" } else { "" };
        let ident = ts_ident(name);
        let ty = render_type(&field.shape, opts, indent);
        out.push_str(&format!("{pad}{ident}{q}: {ty};\n"));
    }
}

fn render_type(shape: &Shape, opts: &EmitOptions, indent: usize) -> String {
    let mut parts: Vec<String> = Vec::new();
    if shape.scalars.boolean {
        parts.push("boolean".to_string());
    }
    if shape.scalars.integer || shape.scalars.float {
        parts.push("number".to_string());
    }
    if shape.scalars.string {
        parts.push("string".to_string());
    }
    if let Some(obj) = &shape.object {
        let pad = "  ".repeat(indent + 1);
        let close_pad = "  ".repeat(indent);
        let mut body = String::from("{\n");
        for (name, field) in &obj.fields {
            let optional = !opts.strict && field.count < obj.total;
            let q = if optional { "?" } else { "" };
            let ident = ts_ident(name);
            let ty = render_type(&field.shape, opts, indent + 1);
            body.push_str(&format!("{pad}{ident}{q}: {ty};\n"));
        }
        body.push_str(&format!("{close_pad}}}"));
        parts.push(body);
    }
    if let Some(arr) = &shape.array {
        let inner = render_type(arr, opts, indent);
        let wrapped = if arr.variant_count() + (arr.nullable as usize) > 1 {
            format!("Array<{}>", inner)
        } else {
            format!("{}[]", inner)
        };
        parts.push(wrapped);
    }
    if shape.nullable {
        parts.push("null".to_string());
    }
    if parts.is_empty() {
        if opts.strict {
            "never".to_string()
        } else {
            "unknown".to_string()
        }
    } else if parts.len() == 1 {
        parts.remove(0)
    } else {
        parts.join(" | ")
    }
}

fn ts_ident(name: &str) -> String {
    let safe = !name.is_empty()
        && name
            .chars()
            .next()
            .map(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
            .unwrap_or(false)
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
    if safe {
        name.to_string()
    } else {
        format!("\"{}\"", name.replace('"', "\\\""))
    }
}
