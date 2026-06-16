# API Examples for Post #1

These are the API examples referenced in the blog post. They serve as a preview of what we'll build.

## Create Collection

```bash
curl -X POST http://localhost:8080/collections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "research_papers",
    "dimension": 768,
    "distance": "cosine"
  }'
```

**Expected Response:**
```json
{
  "status": "created",
  "collection": {
    "name": "research_papers",
    "dimension": 768,
    "distance": "cosine",
    "count": 0
  }
}
```

## Upsert (Insert/Update) Data

```bash
curl -X POST http://localhost:8080/collections/research_papers/upsert \
  -H "Content-Type: application/json" \
  -d '{
    "points": [
      {
        "id": "paper_001",
        "vector": [0.12, 0.05, -0.33, 0.87, 0.23, -0.45, 0.67, 0.89],
        "metadata": {
          "title": "Attention Is All You Need",
          "year": 2017,
          "authors": ["Vaswani", "Shazeer", "Parmar"],
          "citations": 50000
        }
      },
      {
        "id": "paper_002", 
        "vector": [0.08, -0.12, 0.45, 0.91, 0.34, -0.56, 0.78, 0.92],
        "metadata": {
          "title": "BERT: Pre-training of Deep Bidirectional Transformers",
          "year": 2018,
          "authors": ["Devlin", "Chang", "Lee", "Toutanova"],
          "citations": 45000
        }
      }
    ]
  }'
```

**Expected Response:**
```json
{
  "status": "ok",
  "upserted": 2
}
```

## Search

### Basic Search (No Filter)

```bash
curl -X POST http://localhost:8080/collections/research_papers/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.10, 0.02, -0.28, 0.85, 0.20, -0.40, 0.65, 0.88],
    "top_k": 5,
    "include_metadata": true
  }'
```

### Search with Filter

```bash
curl -X POST http://localhost:8080/collections/research_papers/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.10, 0.02, -0.28, 0.85, 0.20, -0.40, 0.65, 0.88],
    "top_k": 5,
    "filter": {
      "year": { "$gte": 2015 },
      "citations": { "$gt": 10000 }
    },
    "include_metadata": true
  }'
```

**Expected Response:**
```json
{
  "results": [
    {
      "id": "paper_001",
      "score": 0.9523,
      "metadata": {
        "title": "Attention Is All You Need",
        "year": 2017,
        "authors": ["Vaswani", "Shazeer", "Parmar"],
        "citations": 50000
      }
    },
    {
      "id": "paper_002",
      "score": 0.8891,
      "metadata": {
        "title": "BERT: Pre-training of Deep Bidirectional Transformers",
        "year": 2018,
        "authors": ["Devlin", "Chang", "Lee", "Toutanova"],
        "citations": 45000
      }
    }
  ],
  "took_ms": 12
}
```

## Delete Points

```bash
curl -X DELETE http://localhost:8080/collections/research_papers/points \
  -H "Content-Type: application/json" \
  -d '{
    "ids": ["paper_001", "paper_002"]
  }'
```

**Expected Response:**
```json
{
  "status": "ok",
  "deleted": 2
}
```

## Delete Collection

```bash
curl -X DELETE http://localhost:8080/collections/research_papers
```

**Expected Response:**
```json
{
  "status": "deleted",
  "collection": "research_papers"
}
```

---

## Filter Operators Reference

| Operator | Description | Example |
|----------|-------------|---------|
| `$eq` | Equal to | `{"year": {"$eq": 2020}}` |
| `$ne` | Not equal to | `{"year": {"$ne": 2020}}` |
| `$gt` | Greater than | `{"year": {"$gt": 2020}}` |
| `$gte` | Greater than or equal | `{"year": {"$gte": 2020}}` |
| `$lt` | Less than | `{"year": {"$lt": 2020}}` |
| `$lte` | Less than or equal | `{"year": {"$lte": 2020}}` |
| `$in` | In array | `{"year": {"$in": [2019, 2020, 2021]}}` |
| `$nin` | Not in array | `{"year": {"$nin": [2019, 2020]}}` |
| `$contains` | Array contains value | `{"authors": {"$contains": "Vaswani"}}` |

## Compound Filters

```json
{
  "filter": {
    "$and": [
      {"year": {"$gte": 2015}},
      {"citations": {"$gt": 10000}},
      {"authors": {"$contains": "Vaswani"}}
    ]
  }
}
```

```json
{
  "filter": {
    "$or": [
      {"year": {"$eq": 2017}},
      {"year": {"$eq": 2018}}
    ]
  }
}
```
