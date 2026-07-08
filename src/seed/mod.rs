//! Seed build: local bronze → silver DuckDB → parquet export + MANIFEST
//! No network required for gate2 pilot path.

pub mod gate2;
pub mod export;
pub mod manifest;
pub mod lookup;
pub mod usda_catalog;
