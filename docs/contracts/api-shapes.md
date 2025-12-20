# API Shapes and Serialization

## Overview

The repo-lens API uses JSON for all communication between clients and the backend engine. All DTOs are defined in the `rl_api` crate with deterministic serialization.

## Request Format

```json
{
  "version": "v0",
  "id": "request-id",
  "payload": {
    "type": "Status",
    "repo_path": "/path/to/repo"
  }
}
```

## Response Format

```json
{
  "id": "request-id",
  "result": {
    "Status": {
      "branch": "main",
      "head": "abc123...",
      "workdir": { ... },
      "index": { ... }
    }
  }
}
```

## Error Format

```json
{
  "id": "request-id",
  "result": {
    "error": {
      "code": "repo_not_found",
      "message": "Repository not found",
      "remediation": "Check the repository path"
    }
  }
}
```

## Streaming Responses

For streaming endpoints, responses are sent as multiple chunks:

```json
{
  "id": "request-id",
  "result": {
    "DiffContent": {
      "sequence": 0,
      "is_final": false,
      "data": { ... }
    }
  }
}
```
