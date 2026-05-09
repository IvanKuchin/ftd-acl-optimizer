# FTD Access Control Rule Optimizer

Cisco FTD translates each access policy rule into low-level ACEs (Access Control Entries). Every platform has a hard ACE limit, and when policies grow organically over time — extra subnets, duplicated objects, overlapping ranges — that limit can creep up faster than expected.

`ftd-acl-optimizer` reads the output of `show access-control-config` and tells you, rule by rule, how many ACEs you are using today and how many you would use after merging adjacent, overlapping, and shadowed subnets. No device changes, no FMC access required — just a text file and the tool.

## Quick Start

**1. Build**

```bash
cargo build --release
```

**2. Collect data from the FTD CLI**

```text
firepower# show access-control-config
```

Save the output to a file (e.g. `policy.txt`).

**3. See overall policy capacity**

```bash
ftd-acl-optimizer --file policy.txt get acp capacity
```

**4. Find the rules with the most optimization potential**

```bash
ftd-acl-optimizer --file policy.txt get top-k by-optimization
```

**5. Get a detailed report for one rule**

```bash
ftd-acl-optimizer --file policy.txt get rule analysis "My_App_Rule"
```

## What It Optimizes

The tool detects three relationships between subnets within a rule's network objects:

| Type | Example | Result |
|------|---------|--------|
| **Adjacency** | `10.0.0.0/25` + `10.0.0.128/25` | `10.0.0.0/24` |
| **Shadow** | `192.168.1.0/24` inside `192.168.0.0/16` | remove the /24 |
| **Overlap** | two IP ranges sharing addresses | merged subnet |

Port/protocol optimization (adjacent TCP/UDP ranges) is handled natively by FTD and is accounted for in the reported numbers.

## Documentation

Full documentation is in this [`docs/` index](README.md):

- [Getting Started](guides/getting-started.md)
- [CLI Reference](cli/commands.md)
- [ACE Calculation](concepts/ace-calculation.md)
- [Optimization Types](concepts/optimization-types.md)
- [Architecture](architecture/overview.md)
- [Contributing](development/contributing.md)
