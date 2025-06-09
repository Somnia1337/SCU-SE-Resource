# Ray Tracing

A Rust reimplementation of a simple ray tracer introduced in [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html).

This project is heavily influenced by a similar one: [ray-tracing-in-one-weekend](https://github.com/fralken/ray-tracing-in-one-weekend).

## Features

See [CHANGELOG](CHANGELOG.md) for version details.

- BVH (Bounding Volume Hierarchies) from [Ray Tracing: The Next Week](https://raytracing.github.io/books/RayTracingTheNextWeek.html).
- Parallel computing powered by [`Rayon`](https://docs.rs/rayon/).

## Example result

The results below are of `1920x1080`, sample rate `1000`, max depth `50`.

### Run without feature flags

![result](/images/result.png)

- scene: lined-up scene (with camera focus)
- rendering time: `248.5s`

### Run with `benchmark` feature

![benchmark](/images/benchmark.png)

- scene: final scene (with camera focus)
- rendering time: `199.7s`

### Run with `course` feature

![course](/images/course.png)

- scene: lined-up scene (without camera focus)
- rendering time: `255.8s`

## Performance

With `benchmark` feature (from `v0.2.1`) enabled (which limits most of the randomness), we get a not-so-serious (but useful enough) benchmark system to measure the performance (rendering time) difference between versions.

Configuration of the benchmark:

- resolution `1200x800`, sample rate `100`, max depth `50`
- run with `cargo run --release --features benchmark`
- run the benchmark multiple times and use the best as the result

| Version  | Best Rendering Time (s) | Speed (pix/s) | Rel-Speed | Note                |
| :------: | :---------------------: | :-----------: | --------- | ------------------- |
| `v0.1.0` |          56.4           |     17.0k     | 1         |                     |
| `v0.2.0` |          34.3           |     28.0k     | 1.64      | implemented BVH     |
| `v0.2.1` |            -            |       -       | -         | no perf improvement |
| `v0.2.2` |          22.2           |     43.2k     | 2.54      | limited BVH depth   |
| `v0.3.0` |           9.3           |    103.2k     | 6.06      | \* see below        |
| `v0.4.0` |            -            |       -       | -         | no perf improvement |
| `v0.4.1` |            -            |       -       | -         | no perf improvement |

- "Rel-Speed" is the relative speed compared to `v0.1.0`.
- Note for `v0.3.0`:
  - Switched to stratified pixel sampling.
  - Optimized `AaBb::hit()` by removing branches, enabling better SIMD utilization.
  - Refactored `Material` trait as an enum to reduce runtime overhead.
  - Enabled `lto` and tuned `codegen-units` for improved performance.
