use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn mcp_stdout_is_pure_jsonrpc() {
    let bin = std::env::var("CARGO_BIN_EXE_vidra").unwrap_or_else(|_| {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/debug/vidra")
            .to_string_lossy()
            .to_string()
    });

    let mut child = Command::new(bin)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn vidra mcp");

    let req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;
    {
        let stdin = child.stdin.as_mut().expect("child stdin missing");
        stdin
            .write_all(req.as_bytes())
            .expect("failed to write request");
        stdin.write_all(b"\n").expect("failed to write newline");
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed waiting for child");

    assert!(output.status.success(), "mcp process failed: {:?}", output.status);

    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr not utf-8");

    assert!(
        !stdout.contains("Starting MCP Server"),
        "stdout contaminated with log text: {stdout}"
    );
    assert!(
        !stdout.contains('\u{1b}'),
        "stdout contaminated with ANSI escape codes: {stdout:?}"
    );

    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(
        lines.len(),
        1,
        "stdout should contain exactly one JSON-RPC line, got: {stdout:?}"
    );

    let value: serde_json::Value =
        serde_json::from_str(lines[0]).expect("stdout line is not valid JSON");
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["id"], 1);
    assert!(value.get("result").is_some(), "response missing result: {value}");

    assert!(
        stderr.contains("Starting MCP Server"),
        "expected startup log in stderr, got: {stderr}"
    );
    assert!(
        !stderr.contains('\u{1b}'),
        "stderr should not contain ANSI escape codes in MCP mode: {stderr:?}"
    );
}
