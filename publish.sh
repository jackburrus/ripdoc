#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
CARGO_TOML="$ROOT/Cargo.toml"
CRATE_DIR="$ROOT/crates/ripdoc-python"
ENV_FILE="$ROOT/.env"

# Load PyPI token
if [[ -f "$ENV_FILE" ]]; then
  source "$ENV_FILE"
fi

if [[ -z "${MATURIN_PYPI_TOKEN:-}" ]]; then
  echo "Error: MATURIN_PYPI_TOKEN not set. Add it to .env or export it."
  exit 1
fi

# Read current version
CURRENT=$(grep '^version' "$CARGO_TOML" | head -1 | sed 's/.*"\(.*\)"/\1/')
echo "Current version: $CURRENT"

# Parse semver
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

# Prompt for bump type
echo ""
echo "Bump type:"
echo "  1) patch  → $MAJOR.$MINOR.$((PATCH + 1))"
echo "  2) minor  → $MAJOR.$((MINOR + 1)).0"
echo "  3) major  → $((MAJOR + 1)).0.0"
echo ""
read -rp "Choose [1/2/3]: " CHOICE

case "$CHOICE" in
  1) NEW="$MAJOR.$MINOR.$((PATCH + 1))" ;;
  2) NEW="$MAJOR.$((MINOR + 1)).0" ;;
  3) NEW="$((MAJOR + 1)).0.0" ;;
  *) echo "Invalid choice"; exit 1 ;;
esac

echo ""
echo "Bumping $CURRENT → $NEW"

# Update version in Cargo.toml
sed -i '' "s/^version = \"$CURRENT\"/version = \"$NEW\"/" "$CARGO_TOML"

# Commit, tag, push
cd "$ROOT"
git add "$CARGO_TOML"
git commit -m "Bump version to $NEW"
git tag "v$NEW"
git push origin main --tags

# Publish to PyPI
echo ""
echo "Publishing to PyPI..."
cd "$CRATE_DIR"
export MATURIN_PYPI_TOKEN
maturin publish

echo ""
echo "Done! ripdoc $NEW is live on PyPI and tagged on GitHub."
