# CLI Reference

Complete reference for all `ftd-acl-optimizer` commands and flags.

## Global Flags

| Flag | Short | Required | Description |
|------|-------|----------|-------------|
| `--file <PATH>` | `-f` | Yes | Path to the file containing `show access-control-config` output |
| `--help` | `-h` | No | Print help |
| `--version` | `-V` | No | Print version |

```bash
ftd-acl-optimizer --file <PATH> <COMMAND>
```

## Command Tree

```
ftd-acl-optimizer
└── get
    ├── acp
    │   ├── capacity     — ACE counts for every rule + whole-policy summary
    │   └── analysis     — Full optimization report for the whole policy
    ├── top-k
    │   ├── by-capacity      — Top 5 rules by raw ACE count
    │   └── by-optimization  — Top 5 rules by optimization ratio
    └── rule
        ├── capacity <NAME>  — ACE counts for one named rule
        └── analysis <NAME>  — Optimization report for one named rule
```

---

## `get acp capacity`

Print the current and optimized ACE count for every rule, followed by a whole-policy summary.

```bash
ftd-acl-optimizer --file policy.txt get acp capacity
```

**Example output:**

```text
==== Rules analysis ====
Rule: My_App_Rule        current: 16    optimized: 4
Rule: Web_Tier           current: 48    optimized: 12
...

==== Access Control Policy ====
# of rules found: 12
acp capacity: 1024
acp optimized capacity: 256
acp optimization ratio: 75.00%
```

---

## `get acp analysis`

Print the full optimization report for every rule in the policy (per-subnet merge suggestions).

```bash
ftd-acl-optimizer --file policy.txt get acp analysis
```

Output is the same as running `get rule analysis` for each rule in sequence.

---

## `get top-k by-capacity`

List the 5 rules consuming the most ACEs today.

```bash
ftd-acl-optimizer --file policy.txt get top-k by-capacity
```

Use this to find rules that are large in absolute terms, regardless of whether they can be optimized.

---

## `get top-k by-optimization`

List the 5 rules with the highest optimization ratio (current ACEs ÷ optimized ACEs).

```bash
ftd-acl-optimizer --file policy.txt get top-k by-optimization
```

Use this to find the rules where cleanup effort yields the greatest ACE reduction.

---

## `get rule capacity <NAME>`

Print the current and optimized ACE count for a single named rule.

```bash
ftd-acl-optimizer --file policy.txt get rule capacity "My_App_Rule"
```

**Example output:**

```text
Rule: My_App_Rule    current: 16    optimized: 4
```

---

## `get rule analysis <NAME>`

Print a detailed optimization report for a single named rule, including which source and destination networks can be merged.

```bash
ftd-acl-optimizer --file policy.txt get rule analysis "My_App_Rule"
```

**Example output:**

```text
Rule: My_App_Rule    current: 16    optimized: 4

Source Networks optimization report:
  192.168.168.0/25 ADJOINS 192.168.168.128/25  →  192.168.168.0/24

Destination Networks optimization report:
  10.11.12.0/24 ADJOINS 10.11.13.0/24  →  10.11.12.0/23
```

The report uses three relationship labels — `ADJOINS`, `SHADOWS`, and `PARTIALLY OVERLAPS` — explained in [Optimization Types](../concepts/optimization-types.md).

---

## See Also

- [Getting Started](../guides/getting-started.md)
- [ACE Calculation](../concepts/ace-calculation.md)
- [Optimization Types](../concepts/optimization-types.md)
