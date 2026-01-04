# Architecture Diagrams (Mermaid)

## System Architecture (Full)

```mermaid
flowchart TB
    subgraph Client["🌐 Client"]
        HTTP["HTTP Request - curl / SDK / Browser"]
    end

    subgraph Transport["LAYER 1: TRANSPORT LAYER<br/>The Doorman"]
        direction LR
        Axum["Axum<br/>Web Framework"] --> JSON["JSON Parser<br/>serde_json"] --> Auth["Authentication<br/>Optional"]
    end

    subgraph Engine["LAYER 2: CORE ENGINE<br/>The Brain"]
        direction TB
        QP["Query Planner - Optimizer"]
        VI["Vector Index - HNSW Graph"]
        MI["Metadata Index - Tantivy"]
        QP --> VI
        QP --> MI
    end

    subgraph Storage["LAYER 3: STORAGE LAYER<br/>The Vault"]
        direction LR
        WAL["Write-Ahead Log<br/>Append-Only"] --> Segments["Segments<br/>Memory-Mapped Files"]
    end

    subgraph Disk["💾 Disk"]
        Files["Binary Files on Disk"]
    end

    HTTP --> Transport
    Auth --> QP
    VI --> WAL
    MI --> WAL
    Segments --> Files

    style Transport fill:#e1f5fe
    style Engine fill:#fff3e0
    style Storage fill:#e8f5e9
```

## Simplified Request Flow

```mermaid
sequenceDiagram
    participant C as Client
    participant T as Transport Layer
    participant E as Core Engine
    participant S as Storage Layer

    C->>T: POST /search { vector: [...] }
    T->>T: Parse JSON, Validate
    T->>E: Search Request
    
    E->>E: Query Planning
    
    alt Vector-First Strategy
        E->>S: Search HNSW Index
        S-->>E: Top 100 candidates
        E->>S: Filter by metadata
        S-->>E: Top K results
    else Filter-First Strategy
        E->>S: Query metadata index
        S-->>E: Matching IDs
        E->>S: Search vectors (subset)
        S-->>E: Top K results
    end
    
    E-->>T: Results with scores
    T-->>C: JSON Response
```

## Data Flow: Insert Operation

```mermaid
flowchart LR
    subgraph Input
        A["Upsert Request<br/>{id, vector, metadata}"]
    end

    subgraph Processing
        B["1. Write to WAL"]
        C["2. Update Vector Index"]
        D["3. Update Metadata Index"]
        E["4. Acknowledge Client"]
    end

    subgraph Background
        F["Periodic Compaction"]
        G["WAL → Segments"]
    end

    A --> B
    B --> C
    C --> D
    D --> E
    B -.-> F
    F --> G

    style B fill:#ffcdd2
    style G fill:#c8e6c9
```

## Component Dependencies

```mermaid
graph TD
    subgraph "Rust Crates We'll Use"
        Tokio["tokio<br/>Async Runtime"]
        Axum["axum<br/>Web Framework"]
        Serde["serde<br/>Serialization"]
        Tantivy["tantivy<br/>Search Engine"]
        Memmap["memmap2<br/>Memory Mapping"]
    end

    subgraph "What We'll Build"
        HNSW["HNSW Index<br/>Vector Search"]
        WAL["WAL<br/>Durability"]
        Engine["Query Engine<br/>Orchestration"]
        API["REST API<br/>Interface"]
    end

    Tokio --> Axum
    Axum --> API
    Serde --> API
    Serde --> WAL
    Memmap --> WAL
    Memmap --> HNSW
    Tantivy --> Engine
    HNSW --> Engine
    WAL --> Engine
    Engine --> API
```

## Embedding Space Visualization

```mermaid
quadrantChart
    title Word Embeddings in 2D Space
    x-axis Low Gender --> High Gender
    y-axis Low Royalty --> High Royalty
    quadrant-1 Queens & Princesses
    quadrant-2 Kings & Princes
    quadrant-3 Common Men
    quadrant-4 Common Women
    King: [0.25, 0.85]
    Queen: [0.75, 0.85]
    Man: [0.25, 0.15]
    Woman: [0.75, 0.15]
    Prince: [0.30, 0.70]
    Princess: [0.70, 0.70]
```

## SQL vs Vector Search Comparison

```mermaid
flowchart LR
    subgraph Query["User Query: 'feline behavior'"]
        Q["🔍"]
    end

    subgraph SQL["Traditional SQL"]
        S1["SELECT * WHERE<br/>text LIKE '%feline%'"]
        S2["❌ 'Cat Psychology'"]
        S3["❌ 'Kitten Development'"]
        S4["✅ 'Feline Studies'"]
    end

    subgraph Vector["Vector Database"]
        V1["Convert query<br/>to embedding"]
        V2["✅ 'Cat Psychology'<br/>score: 0.92"]
        V3["✅ 'Kitten Development'<br/>score: 0.89"]
        V4["✅ 'Feline Studies'<br/>score: 0.95"]
    end

    Q --> S1
    Q --> V1
    S1 --> S2
    S1 --> S3
    S1 --> S4
    V1 --> V2
    V1 --> V3
    V1 --> V4

    style S2 fill:#ffcdd2
    style S3 fill:#ffcdd2
    style S4 fill:#c8e6c9
    style V2 fill:#c8e6c9
    style V3 fill:#c8e6c9
    style V4 fill:#c8e6c9
```

