# Error Taxonomy and HTTP Mapping

All API endpoints return errors wrapped in a uniform JSON envelope:

```json
{
  "error": {
    "code": "<string_code>",
    "message": "<human_readable_description>"
  }
}
```

## Error Mapping Matrix

| Error Condition / Domain Variant | HTTP Status | Code | Client Action / Description |
|---|---|---|---|
| Malformed JSON body | `400 Bad Request` | `malformed_json` | Fix JSON syntax in request body |
| Unknown JSON field in request | `400 Bad Request` | `unknown_field` | Remove extraneous field from request payload |
| Filter not a JSON object | `400 Bad Request` | `invalid_filter` | Filter must be a JSON object |
| Filter type mismatch | `400 Bad Request` | `filter_type_error` | Supply matching types for field comparison |
| Mixed-type list in `match.any` | `400 Bad Request` | `mixed_type_list` | All elements in `any` must share the same type |
| Invalid range bound | `400 Bad Request` | `invalid_range_bound` | Range bounds must be finite numbers (not bool/str/NaN/Inf) |
| Conflicting range bounds | `400 Bad Request` | `conflicting_bounds` | Cannot specify both `gt`/`gte` or `lt`/`lte` |
| Filter recursion depth exceeded | `400 Bad Request` | `max_filter_depth_exceeded` | Simplify nested filter hierarchy below limit |
| Filter clause count exceeded | `400 Bad Request` | `max_filter_clauses_exceeded` | Reduce total filter clauses below limit |
| Dimension is 0 | `400 Bad Request` | `invalid_dimension` | Vector dimension must be $\ge 1$ |
| Dimension exceeds configured max | `400 Bad Request` | `dimension_too_large` | Vector dimension exceeds system limit |
| Vector length != schema dimension | `400 Bad Request` | `dimension_mismatch` | Query vector length must match collection dimension |
| Vector contains NaN, +Inf, -Inf | `400 Bad Request` | `non_finite_float` | All vector components must be finite floats |
| All-zero vector for Cosine metric | `400 Bad Request` | `zero_vector_for_cosine` | Zero vector has undefined Cosine similarity |
| Unsupported schema version | `400 Bad Request` | `unsupported_schema_version` | Use supported schema version (`1`) |
| Record already deleted (tombstone) | `400 Bad Request` | `already_deleted` | Vector is already tombstoned |
| $k \le 0$ or $k > \text{max\_k}$ | `400 Bad Request` | `invalid_k` | Supply $1 \le k \le \text{max\_k}$ |
| Unsupported Content-Type | `415 Unsupported Media Type` | `unsupported_media_type` | Set `Content-Type: application/json` |
| Request body exceeds max bytes | `413 Payload Too Large` | `payload_too_large` | Reduce request payload size below limit |
| Route not found | `404 Not Found` | `not_found` | Check requested endpoint path |
| HTTP method not allowed | `405 Method Not Allowed` | `method_not_allowed` | Use designated HTTP method (e.g. POST) |
| Internal retrieval failure | `500 Internal Server Error` | `internal_error` | Storage or index internal error (details masked) |
| Version counter overflow | `500 Internal Server Error` | `internal_error` | Internal version allocation overflow |

> **Security Rule**: 5xx responses never expose internal stack traces or internal implementation details in the `message` string.
