# ADR 0002: Embed Preprocessed World Data

Status: Accepted  
Date: 2026-08-26

## Context

The source boundary JSON is approximately 8.61 MiB and contains more than
217,000 coordinate positions. Parsing and transforming it on every launch is
feasible on a desktop computer, but it adds startup work, runtime geographic
dependencies, and behavior that is difficult to validate reproducibly.

The terminal renderer needs country identifiers, borders, anchors, metadata,
and pairwise distances rather than arbitrary source JSON.

## Decision

A dedicated Rust tool will transform the source dataset and curated catalog
into a compact, versioned binary asset. The generated asset will be committed
and embedded in the executable. Runtime code will decode it once and will not
need the source JSON or network access.

The source snapshot, provenance, license, and regeneration instructions will
remain in the repository.

## Consequences

- Startup is fast and deterministic.
- Release binaries are self-contained.
- Geographic libraries remain outside the runtime dependency graph.
- CI can verify that committed generated data is current.
- Asset format changes require a version update and regeneration.
- The repository contains both source and generated data.
- Data attribution must remain distinct from the MIT code license.

## Alternatives Considered

### Parse embedded JSON at startup

This is simple initially, but preserves unnecessary precision and metadata,
uses more memory, and repeats fixed work for every game launch.

### Ship the JSON beside the executable

This keeps the binary smaller but complicates installation, path resolution,
and guarantees that data matches the executable.

### Generate data in `build.rs`

Automatic generation makes clean builds expensive and turns normal compilation
into a geographic processing step. An explicit generator and committed asset
make changes reviewable and builds predictable.
