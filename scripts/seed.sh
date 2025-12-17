#!/usr/bin/env bash
set -euo pipefail

# Seed script for House Management System
# Populates the database with sample data for local development

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
ROOT_DIR=$(dirname "$SCRIPT_DIR")

echo "🌱 Seeding database with sample data..."

cd "$ROOT_DIR/api"

# Run the seed binary
cargo run --bin seed

echo "✅ Database seeded successfully!"
echo ""
echo "Sample users created:"
echo "  - admin@example.com / password123 (Admin)"
echo "  - manager@example.com / password123 (Manager)"
echo "  - owner1@example.com / password123 (Homeowner)"
echo "  - owner2@example.com / password123 (Homeowner)"
echo "  - renter1@example.com / password123 (Renter)"
echo ""
echo "Sample buildings: 3 buildings with 2-4 apartments each"
echo ""
echo "Use these credentials to log in and explore the system!"
