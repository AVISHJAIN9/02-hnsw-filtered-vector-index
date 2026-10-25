# Reference Map

This document maps specific features to corresponding reference code repositories located in `docs/reference/repos/`.

> **Note**: `docs/reference/` contains third-party reference implementations and academic research code. It is **READ-ONLY**. Do not edit files under `docs/reference/`, do not copy code directly from them, and do not count reference files toward project implementation progress. Only inspect them when consulting reference implementations or algorithms.

All paths listed below have been verified to exist on the filesystem.

---

### Feature 17: HNSW build
*Graph construction with configurable M/Mmax/efConstruction*
- `docs/reference/repos/hnswlib/hnswlib` — Canonical C++ HNSW graph construction algorithms (`hnswalg.h`, `addPoint`, heuristic neighbor selection)
- `docs/reference/repos/lucene-hnsw` — Java Lucene HNSW graph builders (`HnswGraphBuilder.java`, `OnHeapHnswGraph.java`, `MergingHnswGraphBuilder.java`)
- `docs/reference/repos/acorn/faiss` — ACORN / Faiss graph construction routines (`IndexACORN.cpp`, `IndexACORN.h`)

### Feature 18: HNSW search
*Approximate nearest-neighbor traversal with efSearch*
- `docs/reference/repos/hnswlib/hnswlib` — Canonical nearest neighbor search algorithms (`searchKnn`, `searchBaseLayer` in `hnswalg.h`)
- `docs/reference/repos/lucene-hnsw` — Lucene graph searchers (`HnswGraphSearcher.java`, `AbstractHnswGraphSearcher.java`)

### Feature 19: Filtered traversal
*Apply metadata predicates during candidate exploration*
- `docs/reference/repos/acorn/faiss` — ACORN filtered beam search exploring nodes while evaluating predicates (`IndexACORN.cpp`)
- `docs/reference/repos/lucene-hnsw` — Lucene filtered graph traversal (`FilteredHnswGraphSearcher.java`)
- `docs/reference/repos/hnswlib/hnswlib` — Predicate functor support during traversal (`BaseFilterFunctor` in `hnswalg.h`)

### Feature 20: Pre/post filtering modes
*Support and benchmark filter-before/after candidate selection*
- `docs/reference/repos/acorn/faiss` — Comparative baseline implementations of pre-filtering and post-filtering vs in-graph traversal
- `docs/reference/repos/filtered-ann-benchmarks/filtered_ann_benchmarks` — Benchmark harness comparing pre-filtering, post-filtering, and hybrid filtering
- `docs/reference/repos/lucene-hnsw` — Filtered search implementations (`FilteredHnswGraphSearcher.java`)

### Feature 32: Benchmark harness
*Recall@k, latency, QPS, memory, build time and filter selectivity*
- `docs/reference/repos/filtered-ann-benchmarks/filtered_ann_benchmarks` — Evaluation scripts measuring filtered recall, throughput, and selectivity metrics
- `docs/reference/repos/ann-benchmarks/ann_benchmarks` — Industry-standard ANN benchmark suite (Pareto frontier, QPS, Recall@k)
- `docs/reference/repos/big-ann-benchmarks/benchmark` — NeurIPS Big ANN benchmark execution pipeline

### Feature 36: Property tests
*Validate graph invariants, filters and ranking behavior*
- `docs/reference/repos/proptest/proptest` — Rust property-based testing framework, strategy generators, and test shrinking

### Feature 37: Fuzzing
*Fuzz vectors, filters, serialized indexes and API payloads*
- `docs/reference/repos/cargo-fuzz/src` — Rust `cargo-fuzz` / libFuzzer implementation and test harness
- `docs/reference/repos/afl.rs/cargo-afl` — Rust AFL harness integration
- `docs/reference/repos/arbitrary/src` — Structured input generation for fuzzing arbitrary data structures

### Feature 38: Benchmark datasets
*Curate public/synthetic datasets across dimensions and distributions*
- `docs/reference/repos/filtered-ann-benchmarks/filtered_ann_benchmarks` — Synthetic and real-world filtered dataset generators and loaders
- `docs/reference/repos/big-ann-benchmarks/dataset_preparation` — Dataset download, preparation, and formatting tools
- `docs/reference/repos/ann-benchmarks/ann_benchmarks` — Dataset definitions and HDF5 dataset loaders

### Feature 39: Filter selectivity corpus
*Generate/collect metadata distributions from 0.001% to 100% selectivity*
- `docs/reference/repos/filtered-ann-benchmarks/filtered_ann_benchmarks` — Selectivity distribution configurations and metadata query generators

### Feature 40: Recall@k
*Measure filtered and unfiltered recall against exact ground truth*
- `docs/reference/repos/filtered-ann-benchmarks/filtered_ann_benchmarks` — Exact ground truth calculation and recall evaluation metrics
- `docs/reference/repos/ann-benchmarks/ann_benchmarks` — Recall@k metric calculations and plotting
- `docs/reference/repos/big-ann-benchmarks/eval` — Evaluation and scoring scripts for approximate nearest neighbor benchmarks

### Feature 63: ANN search
*Approximate nearest-neighbor search using HNSW*
- `docs/reference/repos/hnswlib/hnswlib` — Standard approximate nearest neighbor search implementation (`hnswalg.h`)
- `docs/reference/repos/lucene-hnsw` — Core HNSW searcher (`HnswGraphSearcher.java`)

### Feature 64: Filtered ANN
*ANN search constrained by metadata predicates*
- `docs/reference/repos/acorn/faiss` — Filtered approximate nearest neighbor graph exploration (`IndexACORN.cpp`)
- `docs/reference/repos/lucene-hnsw` — Filtered HNSW search (`FilteredHnswGraphSearcher.java`)
- `docs/reference/repos/filtered-ann-benchmarks/filtered_ann_benchmarks` — Filtered ANN evaluation harness

### Feature 65: Exact baseline
*Brute-force/exact search for ground truth and small collections*
- `docs/reference/repos/hnswlib/hnswlib` — Brute-force linear scan baseline (`bruteforce.h`)
- `docs/reference/repos/ann-benchmarks/ann_benchmarks` — Exact linear scan ground truth generation
- `docs/reference/repos/filtered-ann-benchmarks/filtered_ann_benchmarks` — Filtered exact baseline solver

### Feature 71: Filter pushdown
*Use metadata conditions during ANN traversal when safe*
- `docs/reference/repos/acorn/faiss` — Dynamic predicate checking during neighbor exploration
- `docs/reference/repos/lucene-hnsw` — Filter predicate pushdown into graph search (`FilteredHnswGraphSearcher.java`)

### Feature 72: Hybrid execution
*Combine graph traversal and metadata candidate filtering*
- `docs/reference/repos/acorn/faiss` — ACORN-1 and ACORN-gamma hybrid execution models combining graph traversal and inverted list filtering
- `docs/reference/repos/lucene-hnsw` — Seeded and filtered searcher combination (`SeededHnswGraphSearcher.java`, `FilteredHnswGraphSearcher.java`)
- `docs/reference/repos/filtered-ann-benchmarks/filtered_ann_benchmarks` — Hybrid strategy evaluation across varying selectivity bands
