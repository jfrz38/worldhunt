#!/usr/bin/env sh
set -eu

package_files="$(cargo package --list --allow-dirty --locked -p worldhunt | tr '\\' '/')"

required_files='
Cargo.lock
Cargo.toml
LICENSE
README.md
THIRD_PARTY_NOTICES.md
docs/architecture.md
docs/releasing.md
docs/world-data.md
assets/map-details-v1.bin
assets/world-v2.bin
assets/country-map-v1/0_0_0.pbf.gz
assets/country-map-v1/1_0_0.pbf.gz
assets/country-map-v1/1_0_1.pbf.gz
assets/country-map-v1/1_1_0.pbf.gz
assets/country-map-v1/1_1_1.pbf.gz
assets/country-map-v1/anchors-v1.bin
data/countries.toml
data/source/world-boundaries.metadata.toml
data/source/world-details.metadata.toml
data/source/openstreetmap/0_0_0.metadata.toml
data/source/openstreetmap/0_0_0.pbf.gz
data/source/openstreetmap/1.metadata.toml
data/source/openstreetmap/1_0_0.pbf.gz
data/source/openstreetmap/1_0_1.pbf.gz
data/source/openstreetmap/1_1_0.pbf.gz
data/source/openstreetmap/1_1_1.pbf.gz
src/main.rs
'

for file in $required_files; do
    if ! printf '%s\n' "$package_files" | grep -Fqx "$file"; then
        printf 'Missing required package file: %s\n' "$file" >&2
        exit 1
    fi
done

while IFS= read -r file; do
    case "$file" in
        .cargo_vcs_info.json|Cargo.lock|Cargo.toml|Cargo.toml.orig|LICENSE|README.md|THIRD_PARTY_NOTICES.md|docs/architecture.md|docs/releasing.md|docs/world-data.md|src/*|assets/world-v2.bin|assets/map-details-v1.bin|assets/country-map-v1/0_0_0.pbf.gz|assets/country-map-v1/1_0_0.pbf.gz|assets/country-map-v1/1_0_1.pbf.gz|assets/country-map-v1/1_1_0.pbf.gz|assets/country-map-v1/1_1_1.pbf.gz|assets/country-map-v1/anchors-v1.bin|data/countries.toml|data/source/world-boundaries.metadata.toml|data/source/world-details.metadata.toml|data/source/openstreetmap/0_0_0.metadata.toml|data/source/openstreetmap/0_0_0.pbf.gz|data/source/openstreetmap/1.metadata.toml|data/source/openstreetmap/1_0_0.pbf.gz|data/source/openstreetmap/1_0_1.pbf.gz|data/source/openstreetmap/1_1_0.pbf.gz|data/source/openstreetmap/1_1_1.pbf.gz)
            ;;
        *)
            printf 'Unexpected package file: %s\n' "$file" >&2
            exit 1
            ;;
    esac
done <<EOF
$package_files
EOF
