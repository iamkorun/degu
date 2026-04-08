//! Zod schema emitter.

use crate::emit::EmitOptions;
use crate::shape::Shape;

pub fn render(shape: &Shape, opts: &EmitOptions) -> String {
    let mut out = String::from("import { z } from \"zod\";\n\n");
    let body = render_expr(shape, opts, 0);
    out.push_str(&format!("export const {} = {};\n", opts.root_name, body));
    out.push_str(&format!(
        "export type {} = z.infer<typeof {}>;\n",
        opts.root_name, opts.root_name
    ));
    out
}

fn render_expr(shape: &Shape, opts: &EmitOptions, indent: usize) -> String {
    let base = base_expr(shape, opts, indent);
    if shape.nullable {
        format!("{}.nullable()", base)
    } else {
        base
    }
}

fn base_expr(shape: &Shape, opts: &EmitOptions, indent: usize) -> String {
    let mut parts: Vec<String> = Vec::new();
    if shape.scalars.boolean {
        parts.push("z.boolean()".to_string());
    }
    if shape.scalars.integer && !shape.scalars.float {
        parts.push("z.number().int()".to_string());
    } else if shape.scalars.float || shape.scalars.integer {
        parts.push("z.number()".to_string());
    }
    if shape.scalars.string {
        parts.push("z.string()".to_string());
    }
    if let Some(obj) = &shape.object {
        let pad = "  ".repeat(indent + 1);
        let close_pad = "  ".repeat(indent);
        let mut body = String::from("z.object({\n");
        for (name, field) in &obj.fields {
            let optional = !opts.strict && field.count < obj.total;
            let expr = render_expr(&field.shape, opts, indent + 1);
            let expr = if optional {
                format!("{}.optional()", expr)
            } else {
                expr
            };
            body.push_str(&format!("{pad}{}: {},\n", key_literal(name), expr));
        }
        body.push_str(&format!("{close_pad}}})"));
        if opts.strict {
            body.push_str(".strict()");
        }
        parts.push(body);
    }
    if let Some(arr) = &shape.array {
        let inner = render_expr(arr, opts, indent);
        parts.push(format!("z.array({})", inner));
    }
    if parts.is_empty() {
        return if opts.strict {
            "z.never()".to_string()
        } else {
            "z.unknown()".to_string()
        };
    }
    if parts.len() == 1 {
        parts.remove(0)
    } else {
        format!("z.union([{}])", parts.join(", "))
    }
}

fn key_literal(name: &str) -> String {
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
