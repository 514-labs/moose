#!/bin/bash

# Comprehensive test script for geo types implementation
# This demonstrates the complete workflow from ClickHouse import to code generation

echo "🚀 Testing Geo Types Implementation"
echo "=================================="

# 1. Build the project
echo "1. Building project..."
source /usr/local/cargo/env
cargo build --release
if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi
echo "✅ Build successful"

# 2. Run clippy
echo "2. Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings
if [ $? -ne 0 ]; then
    echo "❌ Clippy failed"
    exit 1
fi
echo "✅ Clippy passed"

# 3. Run tests
echo "3. Running tests..."
cargo test -p moose-cli
if [ $? -ne 0 ]; then
    echo "❌ Tests failed"
    exit 1
fi
echo "✅ Tests passed"

# 4. Test specific geo functionality
echo "4. Testing geo type parsing..."
cargo test test_geo_type_conversion test_parse_geo_types
if [ $? -ne 0 ]; then
    echo "❌ Geo tests failed"
    exit 1
fi
echo "✅ Geo tests passed"

# 5. Test code generation
echo "5. Testing code generation..."
cargo test test_tables_to_python
if [ $? -ne 0 ]; then
    echo "❌ Code generation tests failed"
    exit 1
fi
echo "✅ Code generation tests passed"

echo ""
echo "🎉 All tests passed! Geo types implementation is working correctly."
echo ""
echo "📋 Summary:"
echo "  ✅ Compiles without errors"
echo "  ✅ Passes all Clippy checks"
echo "  ✅ All unit tests pass"
echo "  ✅ Geo type parsing works"
echo "  ✅ Type conversion works"
echo "  ✅ Code generation works"
echo ""
echo "🚀 Ready for production use!"
echo ""
echo "📁 Demo applications created:"
echo "  - geo_types_demo_typescript/ (TypeScript demo)"
echo "  - geo_types_demo_python/ (Python demo)"
echo ""
echo "📄 Documentation:"
echo "  - GEO_TYPES_IMPLEMENTATION_SUMMARY.md (Complete implementation details)"
echo "  - demo_clickhouse_geo_import.sql (ClickHouse usage example)"
echo ""
echo "🎯 Users can now:"
echo "  - Import ClickHouse tables with geo columns"
echo "  - Generate TypeScript/Python models with geo types"
echo "  - Use native ClickHouse geo functions"
echo "  - Build geospatial analytics applications"