# Package-owned Agent Skills context

- Contract: `SKILL-CONTEXT-001`; bounded acceptance `SKILL-009`; owning M20 package contribution and M30 context projection.
- Replaceable parser/resource adapter: `crates/adapters/src/skills/`.
- Source: [Agent Skills specification](https://agentskills.io/specification).

## Carrier and trust

A package declares a Skill directory containing `SKILL.md`: YAML frontmatter followed
by Markdown. A real bounded YAML parser supports quoted and block scalar descriptions
and string metadata, not a hand-written line parser. Required name matches the directory
and follows the 1..64 lowercase alphanumeric/hyphen rule, with no leading/trailing or
consecutive hyphens. Description is nonempty and at most 1024 characters. Optional
license, compatibility (at most 500 characters), metadata string map and experimental
allowed-tools follow the format. Unknown extensions may be preserved only as bounded
metadata or rejected explicitly; they never become executable hooks or permissions.

The source loader verifies the exact package-relative path and artifact digest before
exposing a checked document. Names are namespace-qualified by package in the application;
collisions cannot silently shadow another package. Discovery exposes name/description
only. Full body and declared reference text are loaded only on explicit selected-skill
intent or an admitted Agent read-context request. Every read rechecks current installation
and grant in the application. Revocation blocks new context loads, not historical records.

Skill instructions are lower-trust task guidance, never system policy, tool definitions,
SSO identity or approval. `allowed-tools` is advisory text and cannot mint authority.
The Agent receives bounded clearly labelled context content with provenance. It cannot
follow source instructions to change configuration, obtain credentials or execute scripts.

## Files and limits

At most 64 KiB per SKILL.md, 16 KiB YAML frontmatter, 32 metadata entries, 64 KiB per
reference and 256 KiB per selected context. UTF-8 is required. Inputs and YAML expansion
have explicit size/depth/alias limits; duplicate frontmatter keys reject. No includes,
environment substitution or YAML application constructors execute.

The resource reader accepts only declared package-relative text resources. It rejects
absolute paths, parent traversal, URL/UNC paths, symlinks and nonregular files; validates
resolved containment and digest; never recursively scans a host directory or reads
unlisted files. Bundled scripts/assets are descriptive only in this profile, not an
execution permission. Resource decode/lookup errors do not echo file contents or secrets.

## Verification

Positive standard examples cover multiline and quoted fields, metadata, CRLF and Unicode
body. Negative cases cover name/directory mismatch, duplicates, oversized input, aliases,
malformed/multiple YAML documents, unknown fields, invalid UTF-8, path traversal/symlink,
digest drift and unlisted resources. Application cases must prove namespacing, lazy loading,
owner/grant checks and disable/revoke-before-load. Parser success alone is not installation.

### Concrete resource adapter profile

At most 256 declared text resources and 1024 bytes per relative path are accepted.
YAML anchors, aliases, merge keys and includes have zero budget. The filesystem
adapter requires Unix descriptor-relative traversal with NOFOLLOW on every path
component. Non-Unix hosts return UnsupportedPlatform for filesystem resource reads;
portable SKILL.md parsing remains available. Windows clients can use a Linux/WSL
server; this is not evidence for a native Windows resource backend.

### Bounded application resource pages (PLUGIN-001)

The generated read-context tool accepts an optional `resource` string and optional
`offset` integer (default zero, UTF-8 byte offset). Omitting `resource` selects only
the exact checked `skillPath` entry; explicit null, empty strings and short names
are not aliases or requests to search. Explicit paths remain authorized only by
the checked package declaration at resource read time; the tool schema does not
enumerate them, so all 256 admitted resources and 1024-byte relative paths remain
usable. Unknown paths, negative/out-of-range offsets and offsets within a UTF-8
character reject as invalid arguments. An undeclared resource is an argument error,
not evidence that installation or grant review is required. Its tool failure is
recoverable by correcting the arguments, including omitting `resource` to read the
entry; it must not authorize a read or trigger an automatic retry. Artifact/digest
failures remain review-required failures. Each page first verifies the complete bounded resource and its
digest, then returns text without splitting a character. No undeclared file is read.

Results retain `kind`, `resource`, `text` and `instruction_authority: "none"`, with
`offset`, `total_bytes` and `next_offset` (null only at end). A page contains at most
16 KiB of source text and its complete JSON must fit the existing 60 KiB tool-result
budget after escaping; consumers explicitly follow next_offset for remaining text.
There is no silent truncation or automatic I/O for the next page. Skill tool metadata
uses a UTF-8-bounded description projection (at most 2048 bytes of the original
description plus bounded name/path and paging instruction); full metadata/body remain
in the checked source. A large valid Skill cannot invalidate unrelated tool definitions.

Pagination proves complete reconstruction through multiple explicit resource calls,
not that one finite Chat response can ingest every admitted file. The existing
Chat budget permits only two tool-call rounds before its final answer. The model
selects pages relevant to the task and must disclose any partial read and remaining
next_offset. The final model request offers no further tools and asks for an honest
answer from available evidence. A real bounded-Chat test covers two resource pages
followed by that final answer without increasing the Agent budget.
