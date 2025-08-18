#!/bin/bash

# Fix geo types implementation

# 1. Add GeoType import to type_parser.rs
sed -i 's/Column, ColumnType, DataEnum, EnumMember, EnumValue, FloatType, IntType, Nested,/Column, ColumnType, DataEnum, EnumMember, EnumValue, FloatType, GeoType, IntType, Nested,/' apps/framework-cli/src/infrastructure/olap/clickhouse/type_parser.rs

# 2. Replace the geo type conversion error with proper implementation
cat > geo_conversion.txt << 'EOF'
        ClickHouseTypeNode::Geo(geo_type) => {
            let geo_type_variant = match geo_type.as_str() {
                "Point" => GeoType::Point,
                "Ring" => GeoType::Ring,
                "Polygon" => GeoType::Polygon,
                "MultiPolygon" => GeoType::MultiPolygon,
                "LineString" => GeoType::LineString,
                "MultiLineString" => GeoType::MultiLineString,
                _ => return Err(ConversionError::UnsupportedType {
                    type_name: geo_type.clone(),
                }),
            };
            Ok((ColumnType::Geo(geo_type_variant), false))
        },
EOF

# Replace the existing geo conversion
sed -i '/ClickHouseTypeNode::Geo(geo_type) => Err(ConversionError::UnsupportedType {/,/}),/{
    r geo_conversion.txt
    d
}' apps/framework-cli/src/infrastructure/olap/clickhouse/type_parser.rs

rm geo_conversion.txt

echo "Geo type conversion implementation updated"