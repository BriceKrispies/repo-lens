# API Versioning Policy

## Version Scheme

repo-lens uses semantic versioning for its API:

- **v0.x.x**: Initial development, breaking changes allowed
- **v1.x.x**: Stable API, only additive changes
- **v2.x.x+**: Major version bumps for breaking changes

## Compatibility Guarantees

### Within v0
- Additive changes are allowed (new fields, new endpoints)
- Breaking changes require coordination with all clients

### v1 and Beyond
- Strict backward compatibility
- Only additive changes allowed
- Breaking changes require new major version

## Migration Strategy

When breaking changes are needed:
1. Introduce new version alongside old version
2. Update clients to use new version
3. Deprecate old version after transition period
4. Remove old version in next major release

## Version Negotiation

Clients specify the API version in each request:

```json
{
  "version": "v0",
  ...
}
```

The server responds with the same version or an error if unsupported.
