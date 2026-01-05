# Mermaid Diagrams for Post #4

All Mermaid diagrams for "Structs, Enums, and Error Handling" blog post.

---

## Diagram 1: From Primitives to Structures

Use in: Section 1 (Introduction: Modeling the World)

```mermaid
flowchart LR
    subgraph Primitives["Raw Primitives"]
        P1["f32"]
        P2["String"]
        P3["usize"]
    end
    
    subgraph Structures["Structured Types"]
        S1["Vector<br/>├─ id: String<br/>├─ data: Vec&lt;f32&gt;<br/>└─ dimension: usize"]
        S2["DistanceMetric<br/>├─ Cosine<br/>├─ Euclidean<br/>└─ Dot"]
        S3["Result&lt;T, E&gt;<br/>├─ Ok(value)<br/>└─ Err(error)"]
    end
    
    Primitives -->|"organize into"| Structures
    
    style Primitives fill:#ffcdd2,stroke:#c62828
    style Structures fill:#c8e6c9,stroke:#388e3c
```

---

## Diagram 2: Struct Memory Layout

Use in: Section 2.1 (The Named Struct)

```mermaid
flowchart LR
    subgraph Stack["Stack"]
        V["Vector struct<br/>├─ id: String<br/>│   ├─ ptr<br/>│   ├─ len: 7<br/>│   └─ cap: 7<br/>├─ data: Vec&lt;f32&gt;<br/>│   ├─ ptr<br/>│   ├─ len: 768<br/>│   └─ cap: 768<br/>└─ dimension: usize<br/>    = 768"]
    end
    
    subgraph Heap["Heap"]
        H1["'vec_001'<br/>7 bytes"]
        H2["[0.1, 0.2, ... 768 floats]<br/>3072 bytes"]
    end
    
    V -->|"id.ptr"| H1
    V -->|"data.ptr"| H2
    
    style Stack fill:#fff9c4,stroke:#f9a825
    style Heap fill:#bbdefb,stroke:#1976d2
```

---

## Diagram 3: Self Reference Types

Use in: Section 2.2 (Adding Behavior)

```mermaid
flowchart TD
    subgraph Methods["Method Self Parameter Types"]
        direction TB
        M1["&self<br/><i>Immutable borrow</i><br/>Read-only access"]
        M2["&mut self<br/><i>Mutable borrow</i><br/>Can modify fields"]
        M3["self<br/><i>Takes ownership</i><br/>Consumes the value"]
    end
    
    subgraph Examples["Examples"]
        E1["fn magnitude(&self) → f32"]
        E2["fn normalize(&mut self)"]
        E3["fn into_data(self) → Vec&lt;f32&gt;"]
    end
    
    M1 --> E1
    M2 --> E2
    M3 --> E3
    
    style M1 fill:#c8e6c9,stroke:#388e3c
    style M2 fill:#fff9c4,stroke:#f9a825
    style M3 fill:#ffcdd2,stroke:#c62828
```

---

## Diagram 4: Enum Variants

Use in: Section 3.1 (Defining Distance Metrics)

```mermaid
flowchart TB
    subgraph Enum["enum DistanceMetric"]
        direction TB
        V1["Cosine<br/><i>no data</i>"]
        V2["Euclidean<br/><i>no data</i>"]
        V3["Minkowski(f32)<br/><i>holds p parameter</i>"]
        V4["Weighted(Vec&lt;f32&gt;)<br/><i>holds weight vector</i>"]
    end
    
    Variable["metric: DistanceMetric"] --> Enum
    
    Enum -->|"can be"| V1
    Enum -->|"can be"| V2
    Enum -->|"can be"| V3
    Enum -->|"can be"| V4
    
    style Variable fill:#e1bee7,stroke:#7b1fa2
    style V1 fill:#c8e6c9,stroke:#388e3c
    style V2 fill:#c8e6c9,stroke:#388e3c
    style V3 fill:#fff9c4,stroke:#f9a825
    style V4 fill:#bbdefb,stroke:#1976d2
```

---

## Diagram 5: Tagged Union Memory

Use in: Section 3.2 (Systems Note about tagged unions)

```mermaid
flowchart LR
    subgraph Layout["Enum Memory Layout"]
        direction TB
        Tag["Tag: 1 byte<br/><i>which variant?</i>"]
        Data["Data: N bytes<br/><i>size of largest variant</i>"]
    end
    
    subgraph Variants["Variant Storage"]
        direction TB
        V1["Cosine → Tag:0 + 0 bytes"]
        V2["Minkowski(2.0) → Tag:2 + 4 bytes"]
        V3["Weighted(vec![...]) → Tag:3 + 24 bytes"]
    end
    
    Layout --> Variants
    
    style Tag fill:#ffcdd2,stroke:#c62828
    style Data fill:#bbdefb,stroke:#1976d2
```

---

## Diagram 6: Option vs Null

Use in: Section 4 (The Option Enum: Killing the Null Pointer)

```mermaid
flowchart TD
    subgraph Null["C/Java: Nullable Pointer"]
        N1["Vector* ptr"]
        N2["ptr = address<br/>OR<br/>ptr = NULL"]
        N3["❌ Runtime crash<br/>if you forget to check"]
    end
    
    subgraph Option["Rust: Option&lt;T&gt;"]
        O1["Option&lt;Vector&gt;"]
        O2["Some(vector)<br/>OR<br/>None"]
        O3["✅ Compile error<br/>if you forget to match"]
    end
    
    Null -->|"danger"| N3
    Option -->|"safety"| O3
    
    style Null fill:#ffcdd2,stroke:#c62828
    style Option fill:#c8e6c9,stroke:#388e3c
    style N3 fill:#ffcdd2,stroke:#c62828
    style O3 fill:#c8e6c9,stroke:#388e3c
```

---

## Diagram 7: Result Flow

Use in: Section 5.1 (Handling Errors)

```mermaid
flowchart TD
    Op["File::open('database.wal')"] --> Result["Result&lt;File, io::Error&gt;"]
    
    Result -->|"Success"| Ok["Ok(file)"]
    Result -->|"Failure"| Err["Err(error)"]
    
    Ok --> Use["Use the file"]
    Err --> Handle["Handle error<br/>• Log it<br/>• Retry<br/>• Return to caller"]
    
    style Ok fill:#c8e6c9,stroke:#388e3c
    style Err fill:#ffcdd2,stroke:#c62828
```

---

## Diagram 8: The ? Operator

Use in: Section 5.2 (The ? Operator)

```mermaid
flowchart TD
    subgraph Before["Without ?"]
        B1["match File::open(path) {"]
        B2["  Ok(f) => f,"]
        B3["  Err(e) => return Err(e),"]
        B4["}"]
    end
    
    subgraph After["With ?"]
        A1["File::open(path)?"]
    end
    
    Before -->|"equivalent to"| After
    
    subgraph Flow["What ? Does"]
        F1["Call File::open()"] --> F2{"Result?"}
        F2 -->|"Ok(file)"| F3["Unwrap → continue"]
        F2 -->|"Err(e)"| F4["Return Err(e) immediately"]
    end
    
    style After fill:#c8e6c9,stroke:#388e3c
    style F3 fill:#c8e6c9,stroke:#388e3c
    style F4 fill:#ffcdd2,stroke:#c62828
```

---

## Diagram 9: VectorDB Type Hierarchy

Use in: Section 6 (Application: Designing vectordb Types)

```mermaid
flowchart TB
    subgraph Core["Core Types"]
        Vector["Vector<br/>├─ data: Vec&lt;f32&gt;<br/>└─ metadata: HashMap"]
        Metric["DistanceMetric<br/>├─ Cosine<br/>├─ Euclidean<br/>└─ Dot"]
    end
    
    subgraph Search["Search Types"]
        Request["SearchRequest<br/>├─ vector<br/>├─ top_k<br/>├─ metric<br/>└─ filter"]
        Result["SearchResult<br/>├─ id<br/>└─ score"]
    end
    
    subgraph Errors["Error Types"]
        VdbErr["VectorDbError<br/>├─ EmptyVector<br/>├─ DimensionMismatch<br/>├─ NotFound<br/>├─ IoError<br/>└─ ..."]
    end
    
    Request -->|"uses"| Vector
    Request -->|"uses"| Metric
    Request -->|"returns"| Result
    Request -->|"may fail with"| VdbErr
    
    style Core fill:#bbdefb,stroke:#1976d2
    style Search fill:#c8e6c9,stroke:#388e3c
    style Errors fill:#ffcdd2,stroke:#c62828
```

---

## Diagram 10: Type Design Decision Tree

Use in: Section 7 (Summary)

```mermaid
flowchart TD
    Start["What are you modeling?"] --> Q1{"A 'thing' with<br/>properties?"}
    
    Q1 -->|"Yes"| Struct["Use a Struct"]
    Q1 -->|"No"| Q2{"One of several<br/>possibilities?"}
    
    Struct --> Q3{"Needs behavior?"}
    Q3 -->|"Yes"| Impl["Add impl block"]
    Q3 -->|"No"| Done1["Done ✓"]
    Impl --> Done1
    
    Q2 -->|"Yes"| Enum["Use an Enum"]
    Q2 -->|"No"| Q4{"Might not exist?"}
    
    Enum --> Q5{"Variants carry data?"}
    Q5 -->|"Yes"| EnumData["Add fields to variants"]
    Q5 -->|"No"| Done2["Done ✓"]
    EnumData --> Done2
    
    Q4 -->|"Absence is normal"| Option["Use Option&lt;T&gt;"]
    Q4 -->|"Absence is error"| Result["Use Result&lt;T, E&gt;"]
    
    style Struct fill:#bbdefb,stroke:#1976d2
    style Enum fill:#e1bee7,stroke:#7b1fa2
    style Option fill:#fff9c4,stroke:#f9a825
    style Result fill:#c8e6c9,stroke:#388e3c
```
