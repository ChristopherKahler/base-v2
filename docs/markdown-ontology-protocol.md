---
ontology: true
type: protocol
domain: open-ontologies
tags: [markdown, extraction, convention, frontmatter]
summary: Defines the frontmatter convention for markdown files intended for ontology extraction
---

# Markdown Ontology Protocol (MOP)

Standard frontmatter convention for markdown files that should be extracted into the ontological graph via `onto_ast.py`.

## Gate Field

```yaml
ontology: true
```

Only files with `ontology: true` in frontmatter receive rich extraction. All other markdown files get today's behavior (headings + code blocks only).

## Required Fields

| Field | Type | Purpose | Graph Mapping |
|-------|------|---------|---------------|
| `ontology` | boolean | Gate — must be `true` | Extraction trigger (not emitted as triple) |
| `type` | string | Document classification | `rdf:type` → `ops:{PascalCase(type)}` |
| `domain` | string | Knowledge domain | `ops:domain` literal |
| `summary` | string | One-line description | `ops:summary` literal |

## Optional Fields

| Field | Type | Purpose | Graph Mapping |
|-------|------|---------|---------------|
| `tags` | list[string] | Categorization labels | `ops:tag` literal (one per tag) |
| `related` | list[string] | Cross-doc links (file stems or paths) | `ops:relatedTo` edge → target document node |
| `severity` | string | For quirks/issues: blocking, warning, info | `ops:severity` literal |
| `status` | string | Current state: active, deprecated, superseded | `ops:status` literal |
| `discovered` | date string | When this knowledge was learned | `ops:discovered` literal |
| `verified` | boolean | Has this been confirmed? | `ops:verified` literal |
| `supersedes` | string | File stem this replaces | `ops:supersedes` edge → target document node |
| `workspace` | string | Originating workspace | `ops:workspace` literal |

Any additional frontmatter key not in this list becomes an `ops:{key}` literal property on the document node. Nothing is silently dropped.

## Type Vocabulary

Types map to RDF classes via PascalCase conversion: `platform-quirk` → `ops:PlatformQuirk`.

### Starter types (extend as needed)

| Type | Use For |
|------|---------|
| `platform-quirk` | API bugs, undocumented behavior, gotchas |
| `capability` | What a platform/tool can do |
| `limitation` | What a platform/tool cannot do |
| `pattern` | Reusable approach or template |
| `best-practice` | Proven approach to follow |
| `gotcha` | Non-obvious trap |
| `trigger-type` | Workflow trigger documentation |
| `action-type` | Workflow action documentation |
| `workflow-spec` | Full workflow specification |
| `decision` | Architectural or strategic decision |
| `protocol` | Convention or standard (like this doc) |
| `reference` | Pointer to external resource |

## Related Field Convention

The `related` field links documents to each other. Values can be:

- **File stem:** `custom-field-api-limitations` — resolved within same directory
- **Relative path:** `../ghl-workflows/engagement-tagger-pattern` — resolved from file location
- **Qualified:** `ghl-platform/custom-field-api-limitations` — domain-qualified

These become `ops:relatedTo` edges in the graph, enabling cross-document traversal queries.

## Example: Platform Quirk

```yaml
---
ontology: true
type: platform-quirk
domain: ghl-custom-fields
summary: GHL API cannot create dropdown custom fields
tags: [api, custom-fields, dropdowns]
severity: blocking
discovered: 2026-05-27
verified: true
related: []
---
```

## Example: Workflow Spec

```yaml
---
ontology: true
type: workflow-spec
domain: ghl-workflows
summary: Engagement tagger sets Active/Warm/Dormant/Dead tier on contacts
tags: [segmentation, engagement, lifecycle]
status: planned
related:
  - custom-field-api-limitations
  - engagement-tier-thresholds
workspace: extendly
---
```

## Example: Cross-Workspace Decision

```yaml
---
ontology: true
type: decision
domain: content-strategy
summary: Siloed data fields prevent legacy system interference
tags: [architecture, isolation, migration]
discovered: 2026-05-28
workspace: extendly
related:
  - ../chris-ai-systems/knowledge/data-isolation-pattern
---
```

## Extraction Behavior

1. Parser detects `---` fenced YAML frontmatter
2. Checks `ontology: true` — if absent or false, falls through to standard heading extraction
3. Parses all frontmatter keys
4. Emits document node with `rdf:type` from `type` field (PascalCase mapped)
5. Emits literal properties for all scalar fields
6. Emits `ops:tag` triples for each tag
7. Emits `ops:relatedTo` edges for each related entry
8. Continues with standard heading/code block extraction (body content still gets structural nodes)

## Querying Examples

After extraction and loading into Oxigraph:

```sparql
# All platform quirks across all workspaces
SELECT ?doc ?summary WHERE {
  ?doc a ops:PlatformQuirk ;
       ops:summary ?summary .
}

# Everything related to a specific document
SELECT ?related ?type ?summary WHERE {
  ?doc rdfs:label "custom-field-api-limitations.md" .
  ?doc ops:relatedTo ?related .
  ?related a ?type ;
           ops:summary ?summary .
}

# All blocking issues
SELECT ?doc ?domain ?summary WHERE {
  ?doc ops:severity "blocking" ;
       ops:domain ?domain ;
       ops:summary ?summary .
}

# Cross-workspace knowledge for a domain
SELECT ?doc ?workspace ?summary WHERE {
  ?doc ops:domain "ghl-workflows" ;
       ops:workspace ?workspace ;
       ops:summary ?summary .
}
```
