# Performance Expectations

## Performance Budgets

repo-lens aims for sub-second response times for common operations:

| Operation | Warm Cache P95 | Notes |
|-----------|----------------|--------|
| Status | 50ms | Repository status check |
| Log (page) | 80ms | Single page of commit log |
| Diff Summary | 120ms | Summary of changes |
| Commit Details | 100ms | Full commit information |
| Branch List | 50ms | List all branches |
| File Diff | 200ms | Content diff for files |

## Measurement Methodology

- **P95**: 95th percentile response time
- **Warm Cache**: After initial repository scan
- **Test Data**: Linux kernel repository (~1M commits)

## Factors Affecting Performance

1. **Repository Size**: Larger repos = slower operations
2. **Cache State**: Cold start vs warm cache
3. **Network**: Remote operations (fetch/push)
4. **Concurrent Load**: Multiple simultaneous requests

## Optimization Priorities

1. **Incremental Updates**: Avoid full repo rescans
2. **Bounded Caches**: Memory usage limits
3. **Streaming**: Large results don't block
4. **Cancellation**: User intent changes cancel work
