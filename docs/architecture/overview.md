# Architecture Overview

This document describes the module structure of `ftd-acl-optimizer` and how data flows from an input file to a formatted optimization report.

## Overview

The codebase is organized into three layers: CLI, ACP (Access Control Policy), and domain objects. Each layer has a single responsibility and communicates downward through typed structs and `TryFrom` conversions.

## Module Tree

```
src/
├── main.rs                     — Entry point; routes CLI args to cli:: functions
├── cli/
│   ├── mod.rs                  — Public API: analyze_rule, analyze_acp, analyze_topk_*
│   ├── args.rs                 — Clap argument structs (AppArgs, Verb, Entity, …)
│   └── utils.rs                — File I/O and formatted output helpers
└── acp/
    ├── mod.rs                  — Acp struct: Vec<Rule> with capacity/lookup methods
    ├── reader.rs               — Line-by-line parser: splits raw text into rule blocks
    └── rule/
        ├── mod.rs              — Rule struct: name + src/dst networks + src/dst protocols
        ├── network_object/
        │   ├── mod.rs          — NetworkObject: parses and owns a list of NetworkObjectItems
        │   ├── network_object_item.rs      — Enum: ObjectGroup | PrefixList
        │   ├── network_object_optimized.rs — Optimized result (Builder pattern)
        │   ├── prefix_list_item_optimized.rs — Single optimized prefix with relationship label
        │   ├── utilities.rs    — Subnet math helpers (adjacency, shadow, overlap detection)
        │   └── group/          — Group and PrefixList parsing (nested object groups)
        └── protocol_object/
            ├── mod.rs          — ProtocolObject: parses and owns protocol items
            ├── protocol_object_item.rs     — Enum: ProtocolList | Group
            ├── protocol_list_optimized.rs  — Optimized port list
            ├── description.rs  — DescriptionType enum (Adjoins/Shadows/PartiallyOverlaps)
            └── group/          — Protocol group and port-list parsing (TCP/UDP/ICMP/other)
```

## Data Flow

```
Input file (show access-control-config output)
    │
    ▼
cli::utils::read_acp_from_file()     — reads file into Vec<String>
    │
    ▼
acp::Reader::next_rule()             — yields Vec<String> per rule block
    │
    ▼
acp::Rule::try_from(Vec<String>)     — parses one rule block
    │   ├── NetworkObject::try_from  — src/dst networks
    │   └── ProtocolObject::try_from — src/dst protocols
    │
    ▼
acp::Acp(Vec<Rule>)                  — the complete parsed policy
    │
    ▼
cli::{analyze_rule, analyze_acp, analyze_topk_*}
    │   ├── rule.capacity()           — current ACE count
    │   ├── rule.optimized_capacity() — post-optimization ACE count
    │   └── rule.get_optimized_networks() — merge suggestions
    │
    ▼
cli::utils::print_*()                — formatted terminal output
```

## Key Types

| Type | Location | Responsibility |
|------|----------|----------------|
| `AppArgs` | `cli/args.rs` | Clap-derived CLI argument tree |
| `Acp` | `acp/mod.rs` | Holds all parsed rules; provides `capacity()`, `rule_by_name()` |
| `Rule` | `acp/rule/mod.rs` | One access control rule; computes `capacity()` and `optimized_capacity()` |
| `NetworkObject` | `acp/rule/network_object/mod.rs` | Source or destination network object; owns `NetworkObjectItem`s |
| `NetworkObjectItem` | `network_object_item.rs` | Either an `ObjectGroup` or a `PrefixList` |
| `NetworkObjectOptimized` | `network_object_optimized.rs` | Result of merging a network object's subnets |
| `ProtocolObject` | `acp/rule/protocol_object/mod.rs` | Source or destination port/protocol object |
| `ProtocolListOptimized` | `protocol_list_optimized.rs` | Result of merging port ranges |

## Error Handling

All parsing errors use `thiserror`-derived enums and propagate upward via `?`:

```
ProtocolObject::PortObjectError
NetworkObject::NetworkObjectError
    └── Rule::RuleError
            └── Acp::AcpError
                    └── cli::CliError
                            └── AppError  (main)
```

Each variant carries a descriptive message and wraps the upstream error where applicable.

## Capacity Calculation

`Rule::capacity()` multiplies the item counts across all four dimensions:

```rust
src_subnets × dst_subnets × src_ports × dst_ports
```

Any dimension that is absent (e.g., no source port constraint) contributes a factor of 1. `Rule::optimized_capacity()` applies the same formula after merging adjacent, overlapping, and shadowed subnets in the network objects.

## See Also

- [ACE Calculation](../concepts/ace-calculation.md)
- [Optimization Types](../concepts/optimization-types.md)
- [CLI Reference](../cli/commands.md)
