# Optimization Types

This document explains the three types of subnet optimization that `ftd-acl-optimizer` detects and reports: **shadow**, **overlap**, and **adjacency**. All three are applied to network objects (source and destination subnets) within a single rule.

## Overview

`ftd-acl-optimizer` scans the subnets inside each rule's network objects and identifies pairs that can be merged into fewer subnets. The tool reports relationships between subnet pairs using three labels: `SHADOWS`, `PARTIALLY OVERLAPS`, and `ADJOINS`.

Optimization scope is **within a single rule only**. Cross-rule optimization is outside the project scope.

## Shadow

A subnet is shadowed when another subnet in the same object completely contains it. The smaller subnet is redundant — removing it does not change what traffic the rule matches.

**Example:**

```text
192.168.168.0/24   (shadowed)
192.168.0.0/16     (shadows it — contains the entire /24)
```

Removing `192.168.168.0/24` leaves the policy unchanged. ACE reduction factor: the shadowed subnet contributed multiplicatively to the total count, so removing it divides the total by the number of unique subnets in that object.

**Report label:** `SHADOWS`

## Overlap

Two subnets partially overlap when neither fully contains the other but their address ranges share addresses. Overlapping subnets should be merged or split to produce a clean, non-overlapping set.

**Example:**

```text
192.168.168.0-254   (IP range, expands to subnets)
192.168.168.1-255   (overlaps with the above)
```

After merging: `192.168.168.0/24` — one subnet instead of two expanded ranges.

**Report label:** `PARTIALLY OVERLAPS`

## Adjacency

Two subnets are adjacent when they share a boundary and together form a single larger subnet with a shorter prefix length.

**Example:**

```text
192.168.168.0/25    (covers .0–.127)
192.168.168.128/25  (covers .128–.255)
```

These two /25 subnets merge into one /24:

```text
192.168.168.0/24
```

ACE reduction factor: 2× per merged pair at each dimension.

**Report label:** `ADJOINS`

## Worked Example

Consider a rule with the following source and destination objects:

**Source networks:**
```text
192.168.168.0/25
192.168.168.128/25
```

**Destination networks:**
```text
10.11.12.0/24
10.11.13.0/24
```

**Source ports:** `ephemeral`, `FTP` (2 entries)  
**Destination ports:** `HTTPS`, `FTP` (2 entries)

Current ACE count: `2 × 2 × 2 × 2 = 16`

After applying adjacency optimization to both network objects:

| Object | Before | After |
|--------|--------|-------|
| Source networks | 2 subnets | 1 (`192.168.168.0/24`) |
| Destination networks | 2 subnets | 1 (`10.11.12.0/23`) |

Optimized ACE count: `1 × 1 × 2 × 2 = 4`

Optimization factor: **4×**

## Optimization Report Format

Running `get rule analysis` produces output like:

```text
Source Networks optimization report:
  192.168.168.0/25 ADJOINS 192.168.168.128/25  →  192.168.168.0/24

Destination Networks optimization report:
  10.11.12.0/24 ADJOINS 10.11.13.0/24  →  10.11.12.0/23
```

Each line names the two subnets, labels the relationship, and shows the merged result.

## What Is Not Optimized by This Tool

- Port/protocol objects — FTD handles port adjacency and overlap automatically.
- Cross-rule optimizations — merging rules together is out of scope.
- Object group naming or restructuring — the tool reports what to do; you apply changes in FMC.

## See Also

- [ACE Calculation](ace-calculation.md)
- [CLI Reference](../cli/commands.md)
- [Getting Started](../guides/getting-started.md)
