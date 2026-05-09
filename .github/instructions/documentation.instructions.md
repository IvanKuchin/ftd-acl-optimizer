---
description: "Use when writing, updating, or adding documentation — README files, feature docs, architecture docs, CLI reference, or anything inside the docs/ folder. Covers folder organization, file naming, and content structure."
applyTo: "docs/**/*.md"
---

# Documentation Guidelines

## Folder Structure

All documentation lives under `./docs/` and is organized into topic subfolders. Never place new documentation directly in `docs/` root (except the top-level `README.md` which is the entry point / index).

```
docs/
├── README.md               ← index / overview only; links to subfolders
├── architecture/           ← system design, module relationships, data flow
│   └── overview.md
├── cli/                    ← CLI reference: commands, flags, examples
│   └── commands.md
├── concepts/               ← domain concepts (ACE, ACL, shadow rules, optimization)
│   └── ace-calculation.md
├── guides/                 ← how-to guides for end users
│   └── getting-started.md
└── development/            ← contributor docs: build, test, release
    └── contributing.md
```

Add a new subfolder when content does not clearly belong to an existing category. Name folders with lowercase kebab-case.

## File Naming

- Lowercase kebab-case: `ace-calculation.md`, `getting-started.md`.
- Be specific: prefer `network-object-optimization.md` over `optimization.md`.

## docs/README.md — Index Only

The root `docs/README.md` must remain a **navigational index**. It should:
- Briefly state what the project does (1–3 sentences max).
- Contain a table or bulleted list linking to every subfolder's primary document.
- Not contain detailed explanations — those belong in the subfolders.

## Content Structure per Document

Every document should follow this skeleton (omit sections that are not applicable):

```markdown
# Title

Brief one-paragraph description of what this document covers.

## Overview / Background

Why this exists and context needed to understand it.

## <Main Content Sections>

...

## Examples

Concrete, runnable examples (prefer CLI snippets and sample output).

## See Also

Links to related documents within docs/.
```

## Writing Rules

- Write in plain English. Short sentences. Active voice.
- Code blocks must specify the language: ` ```rust `, ` ```bash `, ` ```text `.
- CLI examples use the actual binary name `ftd-acl-optimizer` with realistic flags and input.
- Do not duplicate content from code comments or `--help` output — link or reference it instead.
- Keep each document focused on one topic. If a document grows beyond ~200 lines, split it.
- Update `docs/README.md` whenever a new document or subfolder is added.

## Keeping Docs in Sync

- When a new CLI subcommand is added, update `docs/cli/commands.md`.
- When a new optimization strategy is added, update or create a doc in `docs/concepts/`.
- When the module structure changes, update `docs/architecture/overview.md`.
