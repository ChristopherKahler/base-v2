"""Convert extraction dicts ({nodes, edges}) to Turtle strings for ops:code graph."""

from pathlib import Path

_CONFIG_PATH = Path.home() / ".open-ontologies" / "config.toml"
_ns_cache: dict[str, str] | None = None


def _load_namespaces() -> dict[str, str]:
    global _ns_cache
    if _ns_cache is not None:
        return _ns_cache
    defaults = {
        "ontology": "http://ops-sys.local/ontology#",
        "code": "http://ops-sys.local/code#",
    }
    if _CONFIG_PATH.exists():
        try:
            import tomllib
        except ModuleNotFoundError:
            import tomli as tomllib  # type: ignore[no-redef]
        try:
            with open(_CONFIG_PATH, "rb") as f:
                cfg = tomllib.load(f)
            if "namespaces" in cfg:
                defaults.update(cfg["namespaces"])
        except Exception:
            pass
    _ns_cache = defaults
    return _ns_cache


NOISE_PATH_SEGMENTS = frozenset({
    "node_modules", "vendor", ".git", "__pycache__", ".next", "dist",
    ".nuxt", ".output", "build", ".cache", ".tox", ".mypy_cache",
    ".pytest_cache", "coverage", ".nyc_output", "bower_components",
    ".yarn", ".pnpm", "target/debug", "target/release",
})


def is_noise_path(path: str) -> bool:
    for segment in NOISE_PATH_SEGMENTS:
        if f"/{segment}/" in path or path.endswith(f"/{segment}"):
            return True
    return False


def _build_prefixes() -> str:
    ns = _load_namespaces()
    return (
        f'@prefix ops: <{ns["ontology"]}> .\n'
        f'@prefix code: <{ns["code"]}> .\n'
        '@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n'
        '@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n'
        '@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n'
    )

TYPE_MAP = {
    "module": "Module",
    "class": "Class",
    "function": "Function",
    "method": "Method",
    "import": "Import",
}

RELATION_MAP = {
    "calls": "calls",
    "imports": "imports",
    "contains": "contains",
    "relatedTo": "relatedTo",
    "supersedes": "supersedes",
}

ONTOLOGY_SKIP_KEYS = frozenset({"ontology", "type", "tags", "related", "supersedes"})


def sanitize_iri(name: str) -> str:
    return (
        name.replace("-", "_")
        .replace(" ", "_")
        .replace(".", "_")
        .replace("/", "_")
        .replace("\\", "_")
        .replace(":", "_")
        .replace("(", "")
        .replace(")", "")
        .replace(",", "")
        .replace("'", "")
        .replace('"', "")
    )


def _escape_literal(s: str) -> str:
    return s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")


def _extract_line(node: dict) -> int:
    if "line" in node:
        return node["line"]
    loc = node.get("source_location", "")
    if loc and loc.startswith("L"):
        try:
            return int(loc[1:])
        except ValueError:
            pass
    return 0


def _infer_type(node: dict) -> str:
    ft = node.get("file_type", "")
    if ft == "code":
        return "function"
    return "function"


def serialize(extraction: dict, project: str, source_file: str, language: str) -> str:
    if is_noise_path(source_file):
        return ""
    lines = [_build_prefixes()]
    project_clean = sanitize_iri(project)

    module_iri = f"code:{project_clean}_{sanitize_iri(source_file)}"
    lines.append(f"{module_iri} a ops:Module ;")
    lines.append(f'    rdfs:label "{_escape_literal(source_file)}" ;')
    lines.append(f'    ops:sourceFile "{_escape_literal(source_file)}" ;')
    lines.append(f'    ops:language "{language}" .')
    lines.append("")

    for node in extraction.get("nodes", []):
        meta = node.get("ontology_meta")
        if meta and node.get("file_type") == "ontology_document":
            iri = f"code:{project_clean}_{sanitize_iri(node['id'])}"
            label = _escape_literal(node.get("label", node["id"]))
            onto_type = node.get("ontology_type", "Document")
            lines.append(f"{iri} a ops:{onto_type} ;")
            lines.append(f'    rdfs:label "{label}" ;')
            lines.append(f'    ops:sourceFile "{_escape_literal(source_file)}" ;')
            lines.append(f'    ops:language "{language}" ;')
            for key, val in meta.items():
                if key in ONTOLOGY_SKIP_KEYS:
                    continue
                if isinstance(val, bool):
                    lines.append(f'    ops:{key} {str(val).lower()} ;')
                elif isinstance(val, (int, float)):
                    lines.append(f'    ops:{key} {val} ;')
                elif isinstance(val, str):
                    lines.append(f'    ops:{key} "{_escape_literal(val)}" ;')
                elif isinstance(val, list):
                    for item in val:
                        lines.append(f'    ops:{key} "{_escape_literal(str(item))}" ;')
            for tag in meta.get("tags", []) or []:
                lines.append(f'    ops:tag "{_escape_literal(str(tag))}" ;')
            lines[-1] = lines[-1].rstrip(" ;") + " ."
            lines.append("")
            continue

        node_type = node.get("type", "function")
        if node_type not in TYPE_MAP:
            node_type = _infer_type(node)
        ops_type = TYPE_MAP.get(node_type, "Function")
        iri = f"code:{project_clean}_{sanitize_iri(node['id'])}"
        label = _escape_literal(node.get("label", node["id"]))
        line_num = _extract_line(node)
        signature = node.get("signature", "")

        lines.append(f"{iri} a ops:{ops_type} ;")
        lines.append(f'    rdfs:label "{label}" ;')
        lines.append(f'    ops:sourceFile "{_escape_literal(source_file)}" ;')
        lines.append(f"    ops:sourceLine {line_num} ;")
        lines.append(f'    ops:language "{language}" ;')
        lines.append(f"    ops:definedIn {module_iri} ;")
        if signature:
            lines.append(f'    ops:signature "{_escape_literal(signature)}" ;')
        if node_type == "method" and node.get("parent"):
            parent_iri = f"code:{project_clean}_{sanitize_iri(node['parent'])}"
            lines.append(f"    ops:definedIn {parent_iri} ;")
        lines[-1] = lines[-1].rstrip(" ;") + " ."
        lines.append("")

    for edge in extraction.get("edges", []):
        relation = edge.get("relation", "calls")
        ops_rel = RELATION_MAP.get(relation)
        if not ops_rel:
            continue
        src_iri = f"code:{project_clean}_{sanitize_iri(edge['source'])}"
        tgt_iri = f"code:{project_clean}_{sanitize_iri(edge['target'])}"
        confidence = edge.get("confidence", "extracted").lower()

        lines.append(f"{src_iri} ops:{ops_rel} {tgt_iri} .")
        lines.append(f'<<{src_iri} ops:{ops_rel} {tgt_iri}>> ops:confidence "{confidence}" .')
        lines.append("")

    return "\n".join(lines)
