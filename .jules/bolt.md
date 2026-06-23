## 2024-06-21 - Optimize grid rendering loop via safe slice iteration
**Learning:** In tight rendering loops over 1D arrays simulating 2D grids, replacing point-by-point `.get(idx)` bounds checking with whole-row `.get(start..end)` slice extraction + `.iter().chain()` reduces iteration overhead significantly (measured ~46% speedup).
**Action:** Prefer safe slice iterators over individual index lookups inside tight rendering and matrix traversal loops in Rust to let the compiler elide bounds checks.
