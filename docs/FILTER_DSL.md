# Filter DSL Specification

This document specifies the syntax, semantics, canonicalization, limits, and error handling for the HNSW Filter DSL.

## 1. Syntax & Operators

A filter expression is a JSON object with boolean clause groups: `must`, `should`, and `must_not`. An empty filter `{}` is valid and matches all records.

### 1.1 `must` Group
All clauses within `must` must evaluate to `true`.
```json
{
  "must": [
    {
      "key": "category",
      "match": {
        "value": "electronics"
      }
    },
    {
      "key": "in_stock",
      "match": {
        "value": true
      }
    }
  ]
}
```

### 1.2 `should` Group
At least one clause within `should` must evaluate to `true`. An empty `should` group matches everything (`true`).
```json
{
  "should": [
    {
      "key": "color",
      "match": {
        "value": "red"
      }
    },
    {
      "key": "color",
      "match": {
        "value": "blue"
      }
    }
  ]
}
```

### 1.3 `must_not` Group
None of the clauses within `must_not` may evaluate to `true` (negation of each clause).
```json
{
  "must_not": [
    {
      "key": "status",
      "match": {
        "value": "discontinued"
      }
    }
  ]
}
```

### 1.4 `match.value`
Matches exact equality against a single scalar value. For `StringSet` payloads, matches if the set contains the target string.
```json
{
  "key": "author",
  "match": {
    "value": "Knuth"
  }
}
```

### 1.5 `match.any`
Matches if the record's payload value equals any element in the array. For `StringSet` payloads, matches if the set intersects with the array. Elements in the array must be homogenous in type.
```json
{
  "key": "tags",
  "match": {
    "any": [
      "databases",
      "algorithms",
      "systems"
    ]
  }
}
```

### 1.6 `range`
Specifies numeric bounds with `gt`, `gte`, `lt`, and `lte`.
- `gt` and `gte` cannot be specified together on the same condition.
- `lt` and `lte` cannot be specified together on the same condition.
- Range bounds must be finite numbers (cannot be boolean, string, NaN, or Inf).
```json
{
  "key": "price",
  "range": {
    "gte": 19.99,
    "lt": 100.0
  }
}
```

### 1.7 Nested Groups
Filters can be nested arbitrarily up to `max_filter_depth`.
```json
{
  "must": [
    {
      "key": "region",
      "match": {
        "value": "us-east"
      }
    },
    {
      "should": [
        {
          "key": "tier",
          "match": {
            "value": "gold"
          }
        },
        {
          "key": "tier",
          "match": {
            "value": "platinum"
          }
        }
      ]
    }
  ]
}
```

## 2. Evaluation Rules

### 2.1 Missing-Key Rule
If a condition references a key that does not exist in the record's payload:
- A `match` condition evaluates to `false`.
- A `range` condition evaluates to `false`.
- Consequently, a `must_not` condition wrapping a missing key evaluates to `true` (since the inner predicate did not match).

### 2.2 Numerical Comparisons (Int vs Float)
Numeric comparisons are value-based and cross-type:
- An integer `1` and float `1.0` evaluate as equal (`1 == 1.0`).
- Range comparisons convert payload integers and floats into numerical values before checking bounds.

## 3. Canonicalization Rules

Canonical form produces deterministic, byte-identical JSON:
1. **Flatten Single-Clause Groups**: Single-clause nested groups are flattened where logical semantics are preserved.
2. **Stable Clause Order**: Clauses within each boolean group are sorted canonically.
3. **Stable Key Order**: Object keys and dictionary fields are sorted in lexicographical order.
4. **Stable Set/Any Order**: Lists in `match.any` are sorted canonically.

## 4. Limits & Enforcements

Limits are loaded from `Config` (configurable via environment variables):
- `max_filter_depth`: Maximum depth of nested filter expressions (default: `10`, env: `HNSW_MAX_FILTER_DEPTH`). Exceeding returns `max_filter_depth_exceeded` (HTTP 400).
- `max_filter_clauses`: Maximum total clauses across the AST (default: `100`, env: `HNSW_MAX_FILTER_CLAUSES`). Exceeding returns `max_filter_clauses_exceeded` (HTTP 400).
