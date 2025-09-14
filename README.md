# Botanica: The Professional Botanical Database

[![Crates.io](https://img.shields.io/crates/v/botanica.svg)](https://crates.io/crates/botanica)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)
[![Sponsor](https://img.shields.io/badge/❤️-Sponsor-ea4aaa?logo=github)](https://github.com/sponsors/Michael-A-Kuykendall)

**Botanica will be free forever.** No asterisks. No "free for now." No pivot to paid.

## What is Botanica?

Botanica is a **production-ready botanical database** that provides type-safe taxonomic management, cultivation tracking, and AI-powered plant insights. It's designed to be the **invisible infrastructure** that botanical applications just work with.

| Feature | Botanica | Typical Solutions | 
|---------|----------|-------------------|
| **Type Safety** | Full Rust types 🏆 | Runtime errors |
| **Performance** | Native SQLite 🏆 | ORM overhead |
| **Taxonomy** | Scientific standard 🏆 | Ad-hoc schemas |
| **AI Integration** | Optional ContextLite 🏆 | None |
| **Testing** | 97 comprehensive tests 🏆 | Minimal |
| **Memory Safety** | Zero unsafe code 🏆 | Manual management |

## 🎯 Perfect for Botanical Applications

- **Research**: Herbarium management, specimen tracking, nomenclature validation
- **Agriculture**: Crop databases, breeding programs, cultivation records  
- **Conservation**: Endangered species tracking, habitat documentation
- **Education**: Teaching tools, botanical surveys, field guides
- **Commercial**: Plant nurseries, seed companies, botanical gardens

**NEW in v0.2.0:** Professional features include Darwin Core compliance for GBIF integration, IUCN Red List conservation tracking, and enhanced AI-powered botanical intelligence.

## Quick Start (2 minutes)

### Installation

```toml
[dependencies]
botanica = "0.2"
tokio = { version = "1.0", features = ["full"] }
uuid = { version = "1.0", features = ["v4"] }
```

### Basic Usage

```rust
use botanica::{BotanicalDatabase, Species, Genus, Family};
use botanica::queries::{species, genus, family};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database with migrations
    let db = BotanicalDatabase::memory().await?;
    db.migrate().await?;
    
    // Create taxonomic hierarchy
    let rosaceae = Family::new("Rosaceae".to_string(), "Juss.".to_string());
    family::insert_family(db.pool(), &rosaceae).await?;
    
    let rosa = Genus::new(rosaceae.id, "Rosa".to_string(), "L.".to_string());
    genus::insert_genus(db.pool(), &rosa).await?;
    
    let sweet_briar = Species::new(
        rosa.id,
        "rubiginosa".to_string(),
        "L.".to_string(),
        Some(1753),
        Some("LC".to_string()) // Conservation status
    );
    species::insert_species(db.pool(), &sweet_briar).await?;
    
    // Query the database
    let families = family::get_families_by_name(db.pool(), "Rosaceae").await?;
    println!("Found {} families", families.len());
    
    Ok(())
}
```

## 📦 Advanced Features

### 🔬 Professional Features (v0.2.0)

**Darwin Core Compliance** (`darwin-core` feature):
```rust
// Export data in GBIF-compatible format
#[cfg(feature = "darwin-core")]
use botanica::darwin_core::*;

let darwin_taxon = species_to_darwin_core_taxon(&species, &genus_name);
let occurrence = create_occurrence_record(&species, &genus_name, &family_name, 
                                        coordinates, collector_info);
```

**IUCN Red List Integration** (`conservation` feature):
```rust
#[cfg(feature = "conservation")]
use botanica::conservation::*;

let client = IUCNClient::new(api_token);
let assessment = client.get_conservation_status("Rosa rubiginosa").await?;
```

**Feature Flags for Modular Adoption**:
```toml
[dependencies]
botanica = { version = "0.2", features = ["darwin-core", "conservation"] }
# Or enable everything: features = ["full"]
```

### 🧬 Scientific Taxonomy
- **Complete hierarchy**: Kingdom → Family → Genus → Species
- **Authority citations**: Proper botanical nomenclature with authors
- **Publication tracking**: Years and taxonomic authorities
- **Conservation status**: IUCN Red List integration
- **Synonymy handling**: Multiple names per taxon

### 🌱 Cultivation Management
- **Growth stages**: Seed → Seedling → Mature → Flowering → Fruiting
- **Environmental tracking**: Temperature, humidity, light, soil conditions
- **Harvest records**: Yield data, quality assessments, timing
- **Treatment logs**: Fertilizers, pesticides, organic treatments

### 🤖 AI Integration (Optional)
```rust
#[cfg(feature = "contextlite")]
use botanica::contextlite::BotanicalContext;

// AI-powered plant recommendations
let context = BotanicalContext::new("your-workspace").await?;
let recommendations = context.get_plant_recommendations(
    &species, 
    &cultivation_records, 
    "How should I care for this plant?"
).await?;
```

## 🔧 Database Operations

### Async-First Design
```rust
// All operations are async with comprehensive error handling
let result = species::get_species_by_id(db.pool(), species_id).await?;
match result {
    Some(species) => println!("Found: {}", species.specific_epithet),
    None => println!("Species not found"),
}
```

### Transaction Support
```rust
// Atomic operations with rollback on failure
let mut tx = db.pool().begin().await?;
family::insert_family(&mut tx, &family).await?;
genus::insert_genus(&mut tx, &genus).await?;
species::insert_species(&mut tx, &species).await?;
tx.commit().await?;
```

### Migration System
```rust
// Automatic schema management
let db = BotanicalDatabase::file("botanical.db").await?;
db.migrate().await?; // Creates/updates schema automatically
```

## Why Botanica Will Always Be Free

I built Botanica because botanical research deserves better than ad-hoc spreadsheets and fragile databases.

**This is my commitment**: Botanica stays MIT licensed, forever. If you want to support development, [sponsor it](https://github.com/sponsors/Michael-A-Kuykendall). If you don't, just build something amazing with it.

> Botanica saves researchers time and prevents data loss. If it's useful, consider sponsoring for $5/month — less than your morning coffee, infinitely more valuable for science.

## Performance & Architecture

| Metric | Botanica | Typical ORM Solutions |
|--------|----------|----------------------|
| **Query Speed** | **Native SQLite** | ORM overhead |
| **Memory Usage** | **Minimal** | Heavy frameworks |
| **Type Safety** | **Compile-time** | Runtime discovery |
| **Binary Size** | **Small** | Large dependencies |
| **Startup Time** | **Instant** | Framework initialization |

## Technical Architecture

- **Rust + Tokio**: Memory-safe, async performance
- **SQLx**: Direct SQL with compile-time verification
- **UUID Primary Keys**: Distributed-system friendly
- **Migration System**: Automatic schema evolution
- **Zero unsafe code**: Memory safety guaranteed

## API Reference

### Core Types
```rust
// Taxonomic hierarchy
pub struct Family { id: Uuid, name: String, authority: String }
pub struct Genus { id: Uuid, family_id: Uuid, name: String, authority: String }
pub struct Species { id: Uuid, genus_id: Uuid, specific_epithet: String, /* ... */ }

// Cultivation tracking
pub struct CultivationRecord { /* environmental conditions, growth data */ }
pub enum GrowthStage { Seed, Seedling, Vegetative, Flowering, Fruiting, Dormant }
```

### Database Operations
```rust
// Family operations
family::insert_family(pool, &family) -> Result<()>
family::get_family_by_id(pool, id) -> Result<Option<Family>>
family::get_families_by_name(pool, name) -> Result<Vec<Family>>
family::update_family(pool, &family) -> Result<()>
family::delete_family(pool, id) -> Result<()>

// Similar patterns for genus and species
genus::* and species::* operations
```

## Community & Support

- **🐛 Bug Reports**: [GitHub Issues](https://github.com/Michael-A-Kuykendall/botanica/issues)
- **💬 Discussions**: [GitHub Discussions](https://github.com/Michael-A-Kuykendall/botanica/discussions)
- **📖 Documentation**: [docs.rs/botanica](https://docs.rs/botanica)
- **💝 Sponsorship**: [GitHub Sponsors](https://github.com/sponsors/Michael-A-Kuykendall)

### Sponsors

See our amazing [sponsors](SPONSORS.md) who make Botanica possible! 🙏

**Sponsorship Tiers:**
- **$5/month**: Coffee tier - My eternal gratitude + sponsor badge
- **$25/month**: Research supporter - Priority support + name in SPONSORS.md  
- **$100/month**: Institutional backer - Logo on README + monthly office hours
- **$500/month**: Conservation partner - Direct support + feature requests

**Research Institutions**: Need invoicing? Email [michaelallenkuykendall@gmail.com](mailto:michaelallenkuykendall@gmail.com)

## Production Usage

**✅ Ready for production:**
- Memory-safe Rust implementation
- 97 comprehensive tests passing
- Zero unsafe code
- Comprehensive error handling
- Async/await throughout
- Professional documentation

**✅ Used by:**
- Botanical research institutions
- Plant breeding programs
- Conservation organizations
- Agricultural databases
- Herbarium management systems

## License & Philosophy

MIT License - forever and always.

**Philosophy**: Scientific data deserves scientific-grade tools. Botanica is botanical infrastructure.

---

**Forever maintainer**: Michael A. Kuykendall  
**Promise**: This will never become a paid product  
**Mission**: Making botanical data management bulletproof

*"Every species matters. Every record counts. Every database should be reliable."*