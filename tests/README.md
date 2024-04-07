Tests should in in order, not parallel
```bash
cargo test -- --test-threads=1
```