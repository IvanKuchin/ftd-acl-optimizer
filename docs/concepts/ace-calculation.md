# ACE Calculation

This document explains what an Access Control Entry (ACE) is, how the FTD platform expands access rules into ACEs, and how `ftd-acl-optimizer` counts them.

## Overview

Cisco FTD translates each access policy rule into one or more low-level ACEs in the underlying ASA access list. The total ACE count drives platform capacity limits, not the rule count. Understanding how ACEs are counted tells you why a single rule with many objects can consume thousands of entries.

## Background

Every FTD platform has a maximum supported ACE count. Approaching that limit does not immediately break policy enforcement, but you typically have only a 5–10% buffer beyond the stated limit before behavior becomes unpredictable.

Approximate limits as of 2020 (higher on newer hardware):

| Platform | Max ACEs |
|----------|----------|
| FPR1010  | ~100 K   |
| FPR2100  | ~500 K   |
| FPR4100  | ~2 M     |
| FPR9300 SM-56 | ~6 M |

Use `show access-list element-count` on the device for the current live count.

## How ACEs Are Counted

A single rule's ACE count is the **product** of all object dimensions:

```
ACEs = src_subnets × dst_subnets × src_ports × dst_ports
```

### Example

A rule with:
- 2 source subnets
- 3 destination subnets
- 4 source TCP ports (non-consecutive)
- 5 destination TCP ports (non-consecutive)

Produces:

```
2 × 3 × 4 × 5 = 120 ACEs
```

The total policy ACE count is the sum across all rules.

### How to verify on the device

```text
firepower# show access-list element-count
firepower# show access-list
```

## IP Range Expansion

FTD automatically converts IP ranges inside network objects into the minimal set of subnets that covers the range. Each resulting subnet counts as one unit.

**Example:**

```text
192.168.0.0 - 192.168.0.5
```

Expands to:

```text
192.168.0.0/30   (covers .0–.3)
192.168.0.4/31   (covers .4–.5)
```

That is 2 subnets, not 1.

## Port Adjacency (Built-in FTD Optimization)

FTD automatically merges adjacent or overlapping port ranges inside protocol objects:

```text
SSH TCP/22  +  FTP TCP/21  →  TCP/21-22
```

`ftd-acl-optimizer` accounts for this when computing optimized capacity: the tool applies the same merging logic to port objects before calculating the optimized ACE count.

## Capacity vs. Optimized Capacity

`ftd-acl-optimizer` reports two numbers per rule:

| Metric | Meaning |
|--------|---------|
| `current` | ACE count based on objects as they appear in the policy today |
| `optimized` | ACE count after merging adjacent, overlapping, and shadowed subnets |

The difference between the two is the headroom you can reclaim by cleaning up the network objects in that rule.

## Realistic Scale Example

A single app rule spanning two on-prem data centers and three cloud providers:

| Dimension | Count |
|-----------|-------|
| On-prem source subnets (2 DCs × 10) | 20 |
| Cloud destination subnets (3 CSPs × 10) | 30 |
| Source TCP/UDP ports | 20 |
| Destination TCP/UDP ports | 25 |

```
20 × 30 × 20 × 25 = 300,000 ACEs
```

That single rule consumes 5% of the largest FPR9300 SM-56 platform capacity.

## See Also

- [Optimization Types](optimization-types.md)
- [Getting Started](../guides/getting-started.md)
- [CLI Reference](../cli/commands.md)
