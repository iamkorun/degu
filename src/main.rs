use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use degu::{infer_from_values, parse_json, render, EmitOptions, Format};
use is_terminal::IsTerminal;
use std::io::{self, Read, Write};
use std::path::PathBuf;

/// degu — infer JSON Schema, TypeScript, or Zod from sample JSON.
#[derive(Debug, Parser)]
#[command(name = "degu", version, about, long_about = None)]
struct Cli {
    /// Input JSON file(s). Omit or pass `-` to read from stdin.
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Output format.
    #[arg(short, long, value_enum, default_value_t = FormatArg::JsonSchema)]
    format: FormatArg,

    /// Strict mode: all fields required, additionalProperties=false.
    #[arg(long)]
    strict: bool,

    /// Root type name used for TypeScript and Zod output.
    #[arg(long, default_value = "Root")]
    name: String,

    /// Suppress informational stderr output.
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum FormatArg {
    #[value(name = "json-schema", alias = "jsonschema")]
    JsonSchema,
    #[value(name = "typescript", alias = "ts")]
    TypeScript,
    #[value(name = "zod")]
    Zod,
}

impl From<FormatArg> for Format {
    fn from(f: FormatArg) -> Self {
        match f {
            FormatArg::JsonSchema => Format::JsonSchema,
            FormatArg::TypeScript => Format::TypeScript,
            FormatArg::Zod => Format::Zod,
        }
    }
}

fn main() {
    if let Err(e) = run() {
        let use_color = io::stderr().is_terminal();
        let prefix = if use_color {
            "\x1b[31merror:\x1b[0m"
        } else {
            "error:"
        };
        eprintln!("{prefix} {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let inputs = read_inputs(&cli.files)?;
    if inputs.is_empty() {
        bail!("no input provided (pass a file or pipe JSON to stdin)");
    }

    let mut values = Vec::with_capacity(inputs.len());
    for (label, text) in &inputs {
        let v = parse_json(text).with_context(|| format!("while parsing {label}"))?;
        values.push(v);
    }

    let shape = infer_from_values(values);
    let opts = EmitOptions {
        format: cli.format.into(),
        strict: cli.strict,
        root_name: cli.name,
    };
    let rendered = render(&shape, &opts);

    let mut stdout = io::stdout().lock();
    stdout.write_all(rendered.as_bytes())?;
    Ok(())
}

fn read_inputs(files: &[PathBuf]) -> Result<Vec<(String, String)>> {
    let use_stdin = files.is_empty() || files.iter().any(|p| p.as_os_str() == "-");
    let mut out = Vec::new();
    if use_stdin {
        if io::stdin().is_terminal() && files.is_empty() {
            bail!("no input provided (pass a file or pipe JSON to stdin)");
        }
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .context("failed to read stdin")?;
        if !buf.trim().is_empty() {
            out.push(("<stdin>".to_string(), buf));
        }
    }
    for p in files {
        if p.as_os_str() == "-" {
            continue;
        }
        let text = std::fs::read_to_string(p)
            .with_context(|| format!("failed to read {}", p.display()))?;
        out.push((p.display().to_string(), text));
    }
    Ok(out)
}
