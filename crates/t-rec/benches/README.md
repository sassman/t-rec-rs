# Benchmarks

## Decors Benchmark

Compares native Rust image operations vs ImageMagick for corner and shadow effects.

Using the cargo alias (runs ImageMagick-only benchmarks):

```sh
cargo b:run
```

With the native imgops feature enabled (compares both implementations):

```sh
cargo bench -p t-rec --features x-native-imgops --bench decors_benchmark -- --sample-size 10 --warm-up-time 1
```
