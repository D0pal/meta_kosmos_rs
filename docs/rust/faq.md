- Cargo build hangs with " Blocking waiting for file lock on the registry index" after building parity from source
```
rm -rf ~/.cargo/registry/index/*
rm -rf ~/.cargo/.package-cache
```