use boa_engine::{Context, Source};
use ciborium::{de::from_reader, ser::into_writer};
use wasm_minimal_protocol::*;

initiate_protocol!();

const MERMAID_JS: &str = include_str!("mermaid.min.js");

#[derive(serde::Deserialize)]
struct MermaidInput {
    diagram: String,
    #[serde(default)]
    theme: Option<String>,
}

#[wasm_func]
pub fn render_mermaid(input: &[u8]) -> Result<Vec<u8>, String> {
    // Deserialize the input
    let mermaid_input: MermaidInput =
        from_reader(input).map_err(|e| format!("Failed to deserialize input: {}", e))?;

    // Create a Boa context
    let mut context = Context::default();

    // Load Mermaid library
    context
        .eval(Source::from_bytes(MERMAID_JS))
        .map_err(|e| format!("Failed to load Mermaid library: {}", e))?;

    // Initialize Mermaid with configuration
    let theme = mermaid_input.theme.as_deref().unwrap_or("default");
    let init_code = format!(
        r#"
        mermaid.initialize({{
            startOnLoad: false,
            theme: '{}',
            securityLevel: 'strict',
            logLevel: 'error'
        }});
        "#,
        theme
    );
    context
        .eval(Source::from_bytes(&init_code))
        .map_err(|e| format!("Failed to initialize Mermaid: {}", e))?;

    // Escape the diagram content for JavaScript
    let escaped_diagram = mermaid_input
        .diagram
        .replace('\\', r"\\")
        .replace('`', r"\`")
        .replace('\n', r"\n")
        .replace('\r', "");

    // Render the diagram to SVG using synchronous API
    let render_code = format!(
        r#"
        (function() {{
            try {{
                var result = mermaid.render('mermaid-diagram', `{}`);
                if (result && result.svg) {{
                    return result.svg;
                }} else {{
                    throw new Error('No SVG returned from mermaid.render');
                }}
            }} catch (error) {{
                throw new Error('Mermaid render error: ' + error.message);
            }}
        }})()
        "#,
        escaped_diagram
    );

    let result = context
        .eval(Source::from_bytes(&render_code))
        .map_err(|e| format!("Failed to render diagram: {}", e))?;

    let svg = result
        .as_string()
        .ok_or_else(|| "Result is not a string".to_string())?
        .to_std_string()
        .map_err(|e| format!("Failed to convert to string: {}", e))?;

    // Serialize the output
    let mut output = Vec::new();
    into_writer(&svg, &mut output).map_err(|e| format!("Failed to serialize output: {}", e))?;

    Ok(output)
}

#[wasm_func]
pub fn render_mermaid_simple(input: &[u8]) -> Result<Vec<u8>, String> {
    // Simple version that just takes a string diagram (UTF-8 bytes)
    let diagram =
        String::from_utf8(input.to_vec()).map_err(|e| format!("Invalid UTF-8 input: {}", e))?;

    let mut context = Context::default();

    // Load Mermaid library
    context
        .eval(Source::from_bytes(MERMAID_JS))
        .map_err(|e| format!("Failed to load Mermaid library: {}", e))?;

    // Initialize Mermaid with default settings
    context
        .eval(Source::from_bytes(
            r#"
            mermaid.initialize({
                startOnLoad: false,
                theme: 'default',
                securityLevel: 'strict'
            });
            "#,
        ))
        .map_err(|e| format!("Failed to initialize Mermaid: {}", e))?;

    // Escape the diagram content
    let escaped_diagram = diagram
        .replace('\\', r"\\")
        .replace('`', r"\`")
        .replace('\n', r"\n")
        .replace('\r', "");

    // Render the diagram
    let render_code = format!(
        r#"
        (function() {{
            var result = mermaid.render('diagram', `{}`);
            return result.svg;
        }})()
        "#,
        escaped_diagram
    );

    let result = context
        .eval(Source::from_bytes(&render_code))
        .map_err(|e| format!("Failed to render diagram: {}", e))?;

    let svg = result
        .as_string()
        .ok_or_else(|| "Result is not a string".to_string())?
        .to_std_string()
        .map_err(|e| format!("Failed to convert to string: {}", e))?;

    Ok(svg.into_bytes())
}
