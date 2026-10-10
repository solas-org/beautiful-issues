<h1 align="center"><strong>Beautiful Issues (bissue)</strong></h1>

<h2 align="center"><strong>TL;DR</strong></h1>

Beautiful Issues is a Rust CLI tool that makes debugging and reporting problems simple and structured. It lets you create rich issues, capture crashes with diagnostics, generate tickets, package them for distribution, and push reports to remotes — all with a polished TUI and support for any framework through hooks.

<h2 align="center"><strong>Project Goal</strong></h1>

Provide developers with an elegant, framework-agnostic way to:

- Define custom issues (errors, warnings, fatals) with codes, messages, notes, and advice
- Automatically capture crashes with rich diagnostic context
- Turn crashes into actionable tickets
- Package and share issue definitions
- Report tickets to custom servers or GitHub Issues

The tool is written in Rust but integrates with other ecosystems via hooks, so it can be used from virtually any framework.

<h2 align="center"><strong>Examples</strong></h1>

```bash
# Initialize a project
bissue init

# Create an issue interactively (TUI wizard)
bissue issue new

# Or create one from CLI
bissue issue new --code ERR001 --message "Null reference detected" --advice "Check object initialization"

# Capture a crash
bissue crash new "ERR001|src/main.rs|12:5:10|trace1,trace2" --framework rust

# Create a ticket from a crash (interactive or CLI)
bissue ticket new

# List everything
bissue issue list
bissue crash list
bissue ticket list

# Push tickets to remotes (GitHub or custom)
bissue ticket push

# Generate and manage packages
bissue package gen
bissue package add rust my-lib 1.0.0 ./bissue.tar.zstd
```
