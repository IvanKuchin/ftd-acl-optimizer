# Getting Started

This guide walks through installing `ftd-acl-optimizer`, collecting the required data from an FTD device, and running your first analysis.

## Overview

`ftd-acl-optimizer` takes the text output of `show access-control-config` from a Cisco FTD device and reports the current ACE count for each rule alongside an optimized count. This helps you understand where policy cleanup will have the greatest impact before you touch a single rule on the device.

## Prerequisites

- Rust toolchain (`rustup` + `cargo`) — [install here](https://rustup.rs)
- SSH or console access to the FTD device CLI

## Installation

Clone the repository and build with Cargo:

```bash
git clone <repo-url>
cd ftd-acl-optimizer
cargo build --release
```

The binary is placed at `target/release/ftd-acl-optimizer`. Optionally copy it to a directory on your `$PATH`:

```bash
cp target/release/ftd-acl-optimizer ~/.local/bin/
```

## Collecting Data from FTD

1. SSH into the FTD CLI:
   ```bash
   ssh admin@<ftd-ip>
   ```

2. Drop into expert mode if needed:
   ```text
   > system support diagnostic-cli
   ```

3. Capture the access control config:
   ```text
   firepower# show access-control-config
   ```

4. Save the full terminal output to a file on your workstation, e.g. `policy.txt`.

## Running an Analysis

### Check overall policy capacity

```bash
ftd-acl-optimizer --file policy.txt get acp capacity
```

This prints each rule's current and optimized ACE count, and a summary for the whole policy.

### Find the top rules to optimize

```bash
ftd-acl-optimizer --file policy.txt get top-k by-optimization
```

Lists the 5 rules with the highest optimization ratio (current ACEs / optimized ACEs).

### Analyze a specific rule

```bash
ftd-acl-optimizer --file policy.txt get rule analysis "My_App_Rule"
```

Prints a detailed report showing which source/destination networks can be merged and why.

### Check one rule's capacity numbers only

```bash
ftd-acl-optimizer --file policy.txt get rule capacity "My_App_Rule"
```

## Next Steps

- [CLI Reference](../cli/commands.md) — all commands and flags explained
- [ACE Calculation](../concepts/ace-calculation.md) — understand how counts are computed
- [Optimization Types](../concepts/optimization-types.md) — shadow, overlap, and adjacency explained

## See Also

- [Architecture](../architecture/overview.md)
- [Contributing](../development/contributing.md)
