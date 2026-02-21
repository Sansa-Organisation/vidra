# Vidra MCP Server stdout Contamination Bug Report

## Overview
When connecting the Vidra MCP Server (`@sansavision/vidra mcp`) to standard MCP clients (such as the Antigravity assistant extension), the client fails to parse the initialization message and drops the connection with the following fatal error:

```
Error: calling "initialize": invalid character '\x1b' looking for beginning of value.
```

## Root Cause
The root cause is **ANSI escape codes (colorized text) and standard log messages being incorrectly piped to `stdout` instead of `stderr`.**

By the strict design of the Model Context Protocol (MCP) using a standard input/output (stdio) transport, the `stdout` stream is strictly reserved for valid JSON-RPC payloads. Any non-JSON text output to `stdout` immediately breaks the JSON parsers of communicating clients.

When the `vidra mcp` command starts, it immediately prints a standard lifecycle log line to `stdout`:
```
2026-02-21T17:58:22.998851Z  INFO vidra::mcp: Starting MCP Server...
```

To make matters worse, this log message includes ASCII color escape codes natively (`\x1b[2m`, `\x1b[32m`, etc.). The strict JSON parser inside the Antigravity MCP client attempts to parse the stream, encounters the `\x1b` escape sequence character on the very first byte, and crashes out immediately since it isn't an open curly brace `{` expected of JSON-RPC.

## Expected Behaviour
The `vidra mcp` subcommand must ensure that logging libraries (such as `tracing` or `env_logger` in Rust, or `console.log` in TypeScript) are configured strictly to write to `stderr` and **never** to `stdout` when the process is executing an MCP stdio transport session.

MCP clients will ignore data written to `stderr` or correctly redirect it as auxiliary logs.

### The Model Context Protocol Specification States:
> *"The Host and Client communicate using JSON-RPC 2.0. Messages MUST be sent containing JSON objects terminating with a newline character. Stdio transports MUST output messages purely to `stdout`. All supplemental diagnostic or logging information from the Server MUST be relegated to `stderr`."*

## Workarounds
Currently, any `bunx --bun @sansavision/vidra mcp` command run by an IDE extension fails immediately.
As a temporary local workaround until patched, developers can silence the logs by overriding the server configuration `args` inside their IDE's `mcp.json` config file with a wrapper bash command that redirects `stdout` logic perfectly cleanly. However, fixing the logging sink upstream is the proper solution.
