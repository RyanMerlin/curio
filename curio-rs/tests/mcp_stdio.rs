use std::io::{Read, Write};
use std::process::{Command, Stdio};

#[test]
fn stdio_server_initializes_discovers_searches_and_fetches() {
    let demo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/wiki-demo");
    let requests = [
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),
        serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
        serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"search","arguments":{"query":"demo","limit":1}}}),
        serde_json::json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"fetch","arguments":{"id":"local:8bef743759250442"}}}),
    ];
    let mut child = Command::new(env!("CARGO_BIN_EXE_curio-mcp"))
        .args(["--kb-dir", demo.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut stdin = child.stdin.take().unwrap();
        for request in requests {
            writeln!(stdin, "{}", request).unwrap();
        }
    }
    let mut output = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    let status = child.wait().unwrap();
    assert!(status.success());
    let lines: Vec<serde_json::Value> = output
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(lines.len(), 4);
    let tools = lines[1]["result"]["tools"].as_array().unwrap();
    for expected in ["search", "fetch", "list_categories", "knowledge_status"] {
        assert!(tools.iter().any(|tool| tool["name"] == expected));
    }
    assert!(lines[2].to_string().contains("Demo Publish Checklist"));
    assert!(lines[3].to_string().contains("Demo Publish Checklist"));
}
