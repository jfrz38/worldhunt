# ADR 0004: Use 196 Playable Countries

Status: Accepted  
Date: 2026-08-26

## Context

The source dataset contains 256 country and territory records, only 237 unique
non-empty ISO3 values, and several non-sovereign or exceptional records. It
cannot define the answer set without a product policy.

Common game catalogs range from 195 sovereign states to broader lists that add
partially recognized states, dependencies, and territories.

## Decision

The MVP uses a curated catalog of 196 playable countries: 193 United Nations
member states plus Palestine, Vatican City, and Western Sahara. This catalog is
the authority for targets, accepted guesses, stable identifiers, canonical
English names, and explicit English aliases.

Dataset records outside the catalog may be rendered as neutral land but are not
valid targets or guesses. Separate dependency records are not automatically
merged into a sovereign country's gameplay geometry.

## Consequences

- The target set is stable and easy to document.
- Country naming and aliases are independent of source dataset labels.
- Kosovo, Taiwan, dependencies, and other territories are not playable in the
  MVP.
- Non-playable land requires an explicit neutral rendering state.
- Catalog mappings and exceptional geometries require validation and occasional
  curation.
- Supporting other catalogs later will require a new product decision.

## Alternatives Considered

### 198-country catalog

Adding Kosovo and Taiwan is common in geography games, but introduces a policy
change beyond the selected United Nations-based catalog.

### Every source record

This would expose dependencies, disputed territories, duplicate ownership
semantics, and records without useful country codes as if they were equivalent
answers.

### Infer sovereign states from dataset status

Source status fields are not a sufficiently stable product contract and do not
solve aliases, duplicate records, or exceptional mappings.
