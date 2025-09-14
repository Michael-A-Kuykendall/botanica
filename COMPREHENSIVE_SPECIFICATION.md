# Botanica: Comprehensive Specification for Professional Botanical Data Management

## Executive Summary

Botanica is a production-ready Rust crate for professional botanical data management that follows international standards and best practices used by major herbaria, research institutions, and agricultural organizations worldwide.

## 1. Core Data Standards Compliance

### 1.1 Darwin Core Compliance
**Priority: Critical**

Based on Darwin Core, the most widely used open-access standard for biodiversity data, Botanica must implement:

#### Required Darwin Core Terms
- **occurrenceID**: Unique identifier for each specimen/observation
- **eventDate**: When the specimen was collected/observed
- **decimalLongitude/decimalLatitude**: Geographic coordinates
- **scientificName**: Full scientific name with authorship
- **occurrenceStatus**: Present/Absent
- **basisOfRecord**: Type of record (PreservedSpecimen, LivingSpecimen, HumanObservation, etc.)
- **scientificNameID**: Link to nomenclatural authority

#### Additional Essential Terms
- **catalogNumber**: Institution's specimen identifier
- **recordedBy**: Collector name(s)
- **individualCount**: Number of individuals
- **lifeStage**: Growth stage of organism
- **reproductiveCondition**: Reproductive state
- **establishmentMeans**: Native/Introduced/Cultivated
- **preparations**: How specimen is preserved
- **associatedTaxa**: Related species at collection site

### 1.2 Nomenclatural Standards (ICN/IPNI)
**Priority: Critical**

Following International Plant Names Index (IPNI) standards for nomenclatural data:

#### Required Fields
- **Scientific Name Components**:
  - Family (with APG IV classification support)
  - Genus
  - Specific epithet
  - Infraspecific rank (subsp., var., f., cv.)
  - Infraspecific epithet
  
- **Authority Citation**:
  - Author abbreviation (following IPNI standards)
  - Publication year
  - Basionym author (in parentheses if recombined)
  - Combining author
  
- **Nomenclatural Status**:
  - Valid/Invalid
  - Legitimate/Illegitimate
  - Conserved/Rejected
  - Synonym type (homotypic/heterotypic)

### 1.3 Conservation Status (IUCN Red List)
**Priority: High**

Implement IUCN Red List categories: Not Evaluated, Data Deficient, Least Concern, Near Threatened, Vulnerable, Endangered, Critically Endangered, Extinct in the Wild, and Extinct:

#### Required Fields
- **Category**: EX, EW, CR, EN, VU, NT, LC, DD, NE
- **Criteria**: A-E with subcriteria
- **Assessment Date**
- **Population Trend**: Increasing/Stable/Decreasing/Unknown
- **Threats**: Categorized threat types
- **Conservation Actions**: In-place and needed

## 2. Specimen & Collection Management

### 2.1 Herbarium Specimen Workflow
**Priority: Critical**

Based on established herbarium digitization workflows:

#### Accession Process
```rust
struct AccessionWorkflow {
    // Entry Stage
    transaction_id: String,        // Links to acquisition record
    entry_date: DateTime,
    source_type: SourceType,       // Exchange, Gift, Purchase, Field Collection
    legal_status: LegalStatus,     // Permits, agreements
    
    // Processing Stage
    accession_number: String,      // Sequential or barcode
    processing_status: ProcessingStatus,
    determinations: Vec<Determination>,
    preparation_method: PreparationMethod,
    
    // Storage Stage
    filing_system: FilingSystem,   // Geographic, Taxonomic, Collector
    storage_location: StorageLocation,
    digitization_status: DigitizationStatus,
}
```

#### Quality Control Fields
- Image resolution requirements (minimum 300 DPI)
- Data completeness score
- Verification status
- Annotation history

### 2.2 Living Collection Management
**Priority: High**

Modeled after BG-BASE and IrisBG systems used by botanic gardens:

#### Plant Records
```rust
struct LivingPlant {
    // Identity
    accession_number: String,
    plant_id: String,              // Individual plant identifier
    
    // Source
    provenance: Provenance,
    wild_origin: Option<WildOrigin>,
    propagation_history: Vec<PropagationEvent>,
    
    // Location
    garden_location: GardenLocation,
    bed_position: Option<String>,
    gps_coordinates: Option<Coordinates>,
    
    // Cultivation
    planting_date: Date,
    soil_type: Option<String>,
    irrigation_zone: Option<String>,
    
    // Monitoring
    phenology_records: Vec<PhenologyObservation>,
    health_assessments: Vec<HealthAssessment>,
    treatments: Vec<Treatment>,
}
```

## 3. Agricultural & Germplasm Features

### 3.1 Germplasm Management (GRIN-Global Compatible)
**Priority: High**

Following USDA GRIN standards for germplasm resources:

#### Accession Data
```rust
struct GermplasmAccession {
    // Core Identity
    accession_id: String,
    genus_species: String,
    
    // Passport Data
    collecting_number: String,
    collection_date: Date,
    country_of_origin: String,
    collection_site: CollectionSite,
    habitat_description: String,
    
    // Biological Status
    biological_status: BiologicalStatus, // Wild, Landrace, Breeding Line, Cultivar
    mls_status: bool,                   // Multilateral System inclusion
    
    // Availability
    availability: AvailabilityStatus,
    distribution_restrictions: Vec<String>,
    quarantine_status: QuarantineStatus,
}
```

#### Characterization & Evaluation
```rust
struct CharacterizationData {
    // Morphological Traits
    descriptors: HashMap<String, DescriptorValue>,
    
    // Agronomic Performance
    yield_data: Vec<YieldRecord>,
    quality_traits: Vec<QualityTrait>,
    
    // Stress Resistance
    disease_resistance: Vec<DiseaseResistance>,
    abiotic_stress: Vec<AbioticStressTolerance>,
    
    // Molecular Markers
    genotype_data: Vec<GenotypeMarker>,
}
```

### 3.2 Breeding Program Support
**Priority: Medium**

#### Pedigree Tracking
```rust
struct BreedingLine {
    line_designation: String,
    generation: String,
    
    // Parentage
    female_parent: Option<String>,
    male_parent: Option<String>,
    cross_type: CrossType,
    cross_date: Date,
    
    // Selection History
    selection_method: SelectionMethod,
    selection_criteria: Vec<String>,
    
    // Performance Data
    trial_results: Vec<TrialResult>,
    advancement_decision: Option<AdvancementDecision>,
}
```

#### Plant Variety Protection
- PVP certificate number
- Application date
- Grant date
- Expiration date
- Owner information

## 4. Research & Analysis Features

### 4.1 Taxonomic Research Tools
**Priority: High**

#### Type Specimen Management
```rust
struct TypeSpecimen {
    type_status: TypeStatus,        // Holotype, Isotype, Paratype, etc.
    protologue_citation: String,
    type_locality: String,
    original_material: bool,
    
    // Digitization
    high_res_images: Vec<ImageRecord>,
    transcribed_label: String,
    verified_by: Vec<Verification>,
}
```

#### Nomenclatural Changes
- Synonym tracking with relationships
- Taxonomic concept mapping
- Misapplied name records
- Orthographic variant handling

### 4.2 Ecological & Distribution Data
**Priority: Medium**

#### Habitat Information
```rust
struct HabitatData {
    vegetation_type: String,
    elevation_meters: Range<i32>,
    slope_aspect: Option<String>,
    soil_description: Option<String>,
    
    // Environmental Measurements
    climate_data: ClimateVariables,
    
    // Associated Species
    canopy_species: Vec<String>,
    understory_species: Vec<String>,
    ground_cover: Vec<String>,
}
```

#### Distribution Modeling
- Native range polygons
- Occurrence point validation
- Climate envelope data
- Invasive potential assessment

## 5. Data Exchange & Interoperability

### 5.1 Darwin Core Archive Support
**Priority: Critical**

Implement Darwin Core Archive format with core and extension files:

- **Core Types**: Occurrence, Taxon, Event
- **Extensions**: Multimedia, Measurements, ResourceRelationship
- **Metadata**: EML (Ecological Metadata Language)

### 5.2 API Standards
**Priority: High**

#### RESTful Endpoints
```
GET /api/v1/specimens
GET /api/v1/taxa
GET /api/v1/accessions
POST /api/v1/determinations
PUT /api/v1/specimens/{id}
```

#### GBIF IPT Compatibility
- RSS feed generation
- Incremental harvesting support
- Dataset versioning

### 5.3 Import/Export Formats
**Priority: High**

- **ABCD Schema** (European standard)
- **DwC-A** (Darwin Core Archive)
- **DELTA** (Descriptive taxonomy)
- **KEW CSV** (Herbarium standard)
- **BRAHMS** exchange format

## 6. Specialized Collections

### 6.1 Seed Bank Management
**Priority: Medium**

```rust
struct SeedAccession {
    // Collection Data
    collection_number: String,
    seed_weight_grams: f64,
    thousand_seed_weight: Option<f64>,
    
    // Storage
    storage_behavior: StorageBehavior,  // Orthodox, Recalcitrant, Intermediate
    storage_location: String,
    storage_temperature: f32,
    relative_humidity: f32,
    
    // Viability Testing
    initial_viability: f32,
    test_protocol: TestProtocol,
    viability_tests: Vec<ViabilityTest>,
    predicted_longevity: Option<Duration>,
}
```

### 6.2 DNA/Tissue Banking
**Priority: Low**

```rust
struct TissueSample {
    sample_id: String,
    tissue_type: TissueType,
    preservation_method: PreservationMethod,
    storage_location: String,
    
    // Molecular Work
    dna_extractions: Vec<DNAExtraction>,
    sequences: Vec<SequenceRecord>,
    genbank_accessions: Vec<String>,
}
```

## 7. User & Workflow Management

### 7.1 User Roles & Permissions
**Priority: High**

```rust
enum UserRole {
    Administrator,
    Curator,
    Researcher,
    Technician,
    DataEntry,
    Guest,
}

struct Permissions {
    can_create: bool,
    can_edit: bool,
    can_delete: bool,
    can_approve: bool,
    can_export: bool,
    restricted_collections: Vec<String>,
}
```

### 7.2 Audit Trail
**Priority: Critical**

All data modifications must be tracked:
- User ID
- Timestamp
- Action type
- Previous value
- New value
- Justification (for critical changes)

## 8. Integration Features

### 8.1 External Service Integration
**Priority: Medium**

- **IPNI API**: Name validation
- **GBIF API**: Occurrence data
- **IUCN Red List API**: Conservation status
- **GeoNames API**: Locality standardization
- **BOLD Systems**: DNA barcode data

### 8.2 GIS Integration
**Priority: Medium**

- WGS84 coordinate system
- Uncertainty radius
- Georeference protocol
- Coordinate validation
- Polygon support for areas

## 9. Performance Requirements

### 9.1 Scalability Targets
- Support 10 million+ specimen records
- Sub-second query response for common searches
- Batch import of 100,000 records in < 10 minutes
- Handle 1,000 concurrent users

### 9.2 Data Integrity
- ACID compliance for transactions
- Referential integrity enforcement
- Duplicate detection algorithms
- Data validation rules

## 10. Implementation Priorities

### Phase 1: Core Foundation (Months 1-3)
1. Darwin Core compliant data model
2. Basic CRUD operations
3. Scientific name handling with authorities
4. Simple search interface
5. CSV import/export

### Phase 2: Professional Features (Months 4-6)
1. Herbarium workflow management
2. IUCN conservation status
3. Determination history
4. Darwin Core Archive generation
5. REST API

### Phase 3: Advanced Capabilities (Months 7-9)
1. GRIN-compatible germplasm features
2. Living collection management
3. Breeding program support
4. External API integrations
5. GIS features

### Phase 4: Specialized Tools (Months 10-12)
1. Seed bank management
2. Type specimen handling
3. Advanced search with facets
4. Batch processing tools
5. Statistical reporting

## 11. Testing Requirements

### 11.1 Data Validation Tests
- Scientific name format validation
- Authority abbreviation checks
- Coordinate boundary validation
- Date format consistency
- Required field enforcement

### 11.2 Integration Tests
- Darwin Core Archive generation
- API endpoint functionality
- External service mocking
- Import/export round-trips
- Database migration safety

### 11.3 Performance Tests
- Load testing with millions of records
- Concurrent user simulation
- Query optimization verification
- Memory usage profiling
- Backup/restore timing

## 12. Documentation Requirements

### 12.1 User Documentation
- Installation guide
- Data model documentation
- API reference
- Import/export formats
- Best practices guide

### 12.2 Developer Documentation
- Architecture overview
- Contributing guidelines
- Plugin development guide
- Database schema
- Test coverage reports

## Conclusion

This specification provides a comprehensive framework for developing Botanica into a professional-grade botanical data management system. By following established international standards and incorporating features from successful systems like BRAHMS, Specify, and BG-BASE, Botanica can serve the needs of herbaria, botanic gardens, agricultural research institutions, and conservation organizations worldwide.

The modular architecture allows for incremental development while maintaining compatibility with global biodiversity informatics infrastructure. Priority should be given to Darwin Core compliance and core specimen management features, with specialized capabilities added based on user community needs.