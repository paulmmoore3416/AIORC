# Sovereign Orchestrator - Project Status & Handoff Document

**Project Name:** Sovereign Orchestrator (AIORC)  
**Location:** `/home/paul/Documents/AIORC`  
**Status:** ~80% Complete - Build Phase with Critical Fixes Needed  
**Last Updated:** May 13, 2026

---

## 📋 Executive Summary

**Sovereign Orchestrator** is a high-performance, resource-efficient AI orchestration layer that coordinates multiple locally-hosted small language models (1B-8B parameters) into a single unified logical unit. The project is built in Rust using gRPC for inter-service communication.

### Current State
- ✅ Core architecture designed and partially implemented
- ✅ 90% of source code written
- ⚠️ Compilation errors in gRPC trait implementations (20 errors remaining)
- ⚠️ Build toolchain issues (protobuf compiler)
- ❌ Not yet buildable or testable

---

## ✅ COMPLETED COMPONENTS

### 1. Project Structure & Configuration
- **Status:** ✅ Complete
- **Files:**
  - `Cargo.toml` - Package manifest with all dependencies
  - `build.rs` - Protocol buffer compilation script
  - `config.json` - Default system configuration
  - `.gitignore` - Git configuration

### 2. Protocol Buffer Definitions
- **Status:** ✅ Complete (needs compile test)
- **File:** `proto/orchestrator.proto`
- **Contents:**
  - `ModelWorker` service (inference endpoint)
  - `OrchestratorRouter` service (routing endpoint)
  - `OrchestratorGateway` service (unified entry point)
  - Message types for requests/responses
  - Health checks and service discovery

### 3. Core Module (`src/core/`)
- **Status:** ✅ Complete
- **Components:**
  - `error.rs` - Custom error types and conversions (OrchestratorError)
  - `config.rs` - Configuration structures and file I/O
  - `types.rs` - Core data types (InferenceRequest, RoutingDecision, etc.)
  - `metrics.rs` - Metrics collection and system statistics

### 4. Routing Module (`src/routing/`)
- **Status:** ✅ ~95% Complete (minor fixes needed)
- **Components:**
  - `complexity_scorer.rs` - Analyzes prompt complexity (1-10 scale)
    - Pattern matching for code, math, logic, creative content
    - Returns tier classification: "instant", "analytical", "expert"
  - `semantic_embedder.rs` - Lightweight semantic task classification
    - Vector-based task classification (code, creative, logic, summarization, chat, math, data)
    - Cosine similarity computation
  - `service_registry.rs` - Service discovery and model registration
    - Tracks available models and their health status
    - Load balancing queries
  - `router.rs` - Main routing logic
    - ⚠️ Has type mismatch errors (needs fixes)
    - Implements routing decisions based on complexity + task type

### 5. Memory Management Module (`src/memory/`)
- **Status:** ✅ ~90% Complete (minor fixes needed)
- **Components:**
  - `warm_swap.rs` - LRU-based VRAM management
    - Model weight caching with eviction strategy
    - Swap time estimation
    - Swap event logging for monitoring
  - `semantic_cache.rs` - Local vector database for cached logic chains
    - Hash-based prompt lookup
    - Similarity-based retrieval
    - Cache statistics tracking
  - `memory_manager.rs` - Unified memory management interface

### 6. Inference Module (`src/inference/`)
- **Status:** ✅ ~85% Complete (trait implementation issues)
- **Components:**
  - `inference_engine.rs` - Local inference wrapper
    - Placeholder for llama.cpp integration
    - Token generation simulation
    - Model metadata management
  - `model_sidecar.rs` - gRPC server wrapper for models
    - ⚠️ Has trait implementation errors (Status vs OrchestratorError mismatch)
    - Streaming inference support
    - Health checks

### 7. Gateway Module (`src/gateway/`)
- **Status:** ✅ ~80% Complete
- **Components:**
  - `orchestrator_gateway.rs` - Main gateway orchestrator
    - HTTP server initialization
    - Service registry and router integration
    - Metrics collection
  - `handlers.rs` - HTTP endpoint handlers
    - `/query` - Main inference endpoint
    - `/health` - Health check
    - `/metrics` - System metrics
    - `/models` - Available models list

### 8. Binary Targets
- **Status:** ✅ ~90% Complete
- **Files:**
  - `src/bin/gateway.rs` - Main HTTP gateway server (production entry point)
  - `src/bin/sidecar.rs` - gRPC wrapper for individual models
  - `src/bin/router.rs` - Standalone routing demonstration

### 9. Documentation
- **Status:** ✅ Complete
- **Files:**
  - `README.md` - Comprehensive project documentation
  - `config.json` - Configuration example with 4 model definitions

---

## ❌ CRITICAL ISSUES BLOCKING BUILD

### Issue 1: Compilation Errors (20 Total)
**Location:** Multiple files  
**Type:** Type mismatch and trait incompatibility  

**Error Groups:**

1. **E0053 - Trait Implementation Mismatch** (2 errors)
   - **File:** `src/inference/model_sidecar.rs`
   - **Issue:** Methods return `crate::core::Result<T>` but trait expects `Result<T, Status>`
   - **Methods:** `health_check()`, `get_model_info()`
   - **Root Cause:** Type alias conflict - `crate::core::Result` is defined as `Result<T, OrchestratorError>` but gRPC expects `Result<T, tonic::Status>`
   - **Fix:** Need to use `Result<..., Status>` directly for these trait methods

2. **E0107 - Wrong Number of Generic Arguments** (4 errors)
   - **Files:** `src/routing/router.rs`, `src/memory/semantic_cache.rs`
   - **Issue:** Generic type parameters mismatch
   - **Examples:**
     - `cosine_similarity` missing type annotations
     - Iterator `min_by_key` expecting `Ord` but getting `f32`

3. **E0277 - Trait Bounds Not Satisfied** (3 errors)
   - **Issue:** `f32` doesn't implement `Ord` trait
   - **Files:** `src/routing/router.rs`, `src/memory/warm_swap.rs`

4. **E0308 - Type Mismatch** (2 errors)
   - **File:** `src/memory/semantic_cache.rs`
   - **Issue:** DashMap iteration returns different type than pattern expects

5. **E0382 - Borrow of Moved Value** (3 errors)
   - **File:** `src/routing/router.rs`
   - **Issue:** `fallback.model_id` moved and then borrowed again

6. **E0425 - Undefined Variable** (3 errors)
   - **Files:** Various
   - **Issue:** Missing imports or typos

7. **E0432 - Unresolved Import** (3 errors)
   - **Issue:** `orchestrator` module not properly exported

### Issue 2: Build Tool Missing
**Status:** ⚠️ PARTIALLY RESOLVED
- **Problem:** `protoc` (Protocol Buffers compiler) not installed
- **Current Solution:** Downloaded manually to `/tmp/bin/protoc`
- **Permanent Solution:** Need to either:
  - Install system-wide: `apt-get install protobuf-compiler` (requires sudo)
  - Or add `prost-build` workaround to `build.rs`

---

## 🔴 IMMEDIATE NEXT STEPS (Prioritized)

### Phase 1: Fix Compilation Errors (2-3 hours)

#### Step 1.1: Fix gRPC Trait Methods
**File:** `src/inference/model_sidecar.rs`
**Action:** Change return types for `health_check` and `get_model_info` to `Result<Response<T>, Status>`

```rust
// BEFORE:
async fn health_check(&self, _request: Request<HealthCheckRequest>,) 
    -> Result<Response<HealthCheckResponse>, Status>

// AFTER (need to change to return Status, not OrchestratorError):
async fn health_check(&self, _request: Request<HealthCheckRequest>,) 
    -> Result<Response<HealthCheckResponse>, Status> {
    // Convert OrchestratorError to Status using existing From impl
}
```

#### Step 1.2: Fix f32 Comparisons
**Files:** `src/routing/router.rs`, `src/memory/warm_swap.rs`
**Action:** Replace `min_by_key` on floats with `partial_cmp`-based iteration

```rust
// BEFORE:
models.iter().min_by_key(|m| m.current_load)

// AFTER:
models.iter().min_by(|a, b| 
    a.current_load.partial_cmp(&b.current_load)
    .unwrap_or(std::cmp::Ordering::Equal)
)
```

#### Step 1.3: Fix DashMap Iterator Issues
**File:** `src/memory/semantic_cache.rs`
**Action:** Properly destructure DashMap iterator results

```rust
// Fix the evict_lru method to handle DashMap RefMulti correctly
```

#### Step 1.4: Fix Move/Borrow Issues
**File:** `src/routing/router.rs`
**Action:** Clone strings before move or use references

### Phase 2: Verify Compilation (30 minutes)
```bash
cd /home/paul/Documents/AIORC
PROTOC=/tmp/bin/protoc cargo check
PROTOC=/tmp/bin/protoc cargo build --release
```

### Phase 3: Run Tests (1 hour)
```bash
PROTOC=/tmp/bin/protoc cargo test --lib
PROTOC=/tmp/bin/protoc cargo test --doc
```

### Phase 4: Manual Integration Tests (2-3 hours)
```bash
# Terminal 1: Start gateway
cargo run --release --bin orchestrator_gateway

# Terminal 2: Start sidecar
MODEL_ID=tinyllama cargo run --release --bin model_sidecar

# Terminal 3: Test routing
cargo run --release --bin router

# Terminal 4: Call HTTP API
curl -X POST http://localhost:9090/query \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Hello", "temperature": 0.7, "max_tokens": 256}'
```

---

## 🟡 MEDIUM-PRIORITY TASKS

### Task 1: Replace Placeholder Inference Engine
**File:** `src/inference/inference_engine.rs`
**Current:** Simulation-only implementation
**Action:** Integrate actual llama.cpp bindings
- Add `llama-cpp-2` crate to `Cargo.toml`
- Implement actual model loading
- Implement actual token generation

### Task 2: Implement Proper Memory Monitoring
**File:** `src/core/metrics.rs`
**Current:** Placeholder values (0.0 for VRAM/CPU)
**Action:** 
- Integrate `memory-stats` crate for real memory usage
- Add GPU VRAM monitoring (using nvidia-ml or similar)
- Add CPU usage via `/proc` on Linux

### Task 3: Add Real Semantic Embeddings
**File:** `src/routing/semantic_embedder.rs`
**Current:** Hardcoded task vectors
**Action:**
- Integrate actual embedding model (all-MiniLM-L6-v2)
- Use `ort` crate for ONNX Runtime
- Or use `sentence-transformers` Python bridge

### Task 4: Implement Service Discovery Registry
**File:** `src/routing/service_registry.rs`
**Current:** In-memory only
**Action:**
- Add persistence (JSON file or SQLite)
- Add automatic health check heartbeat
- Add model auto-registration on startup

---

## 🟢 LOWER-PRIORITY TASKS

### Task 1: Add WebSocket Support
**For real-time token streaming**

### Task 2: Add Admin Dashboard
**For monitoring and management**

### Task 3: Add Metrics Export
**Prometheus format for monitoring**

### Task 4: Add Multi-GPU Support
**Load balancing across GPUs**

### Task 5: Add ONNX Runtime Support
**For CPU inference optimization**

---

## 📁 File Structure

```
/home/paul/Documents/AIORC/
├── Cargo.toml                    # ✅ Package manifest
├── build.rs                      # ✅ Proto compilation script
├── config.json                   # ✅ Configuration example
├── README.md                     # ✅ Documentation
│
├── proto/
│   └── orchestrator.proto        # ✅ gRPC service definitions
│
├── src/
│   ├── lib.rs                    # ✅ Library root
│   ├── main.rs                   # ✅ (placeholder)
│   │
│   ├── core/                     # ✅ Core types & config
│   │   ├── mod.rs
│   │   ├── error.rs              # ✅ Error types
│   │   ├── config.rs             # ✅ Configuration
│   │   ├── types.rs              # ✅ Data types
│   │   └── metrics.rs            # ✅ Metrics collection
│   │
│   ├── routing/                  # ⚠️ Routing logic (minor fixes needed)
│   │   ├── mod.rs
│   │   ├── router.rs             # ⚠️ Main router (has errors)
│   │   ├── complexity_scorer.rs  # ✅ Prompt analysis
│   │   ├── semantic_embedder.rs  # ✅ Task classification
│   │   └── service_registry.rs   # ✅ Model registry
│   │
│   ├── memory/                   # ⚠️ Memory management (minor fixes)
│   │   ├── mod.rs
│   │   ├── warm_swap.rs          # ⚠️ VRAM management
│   │   ├── semantic_cache.rs     # ⚠️ Logic cache (iterator issues)
│   │   └── memory_manager.rs     # ✅ Interface
│   │
│   ├── inference/                # ⚠️ Inference (trait errors)
│   │   ├── mod.rs
│   │   ├── inference_engine.rs   # ✅ Simulation version
│   │   └── model_sidecar.rs      # ⚠️ gRPC wrapper (trait errors)
│   │
│   ├── gateway/                  # ✅ HTTP gateway
│   │   ├── mod.rs
│   │   ├── orchestrator_gateway.rs # ✅ Main gateway
│   │   └── handlers.rs           # ✅ HTTP handlers
│   │
│   └── bin/                      # ✅ Binary targets
│       ├── gateway.rs            # ✅ Gateway server
│       ├── sidecar.rs            # ✅ Model wrapper
│       └── router.rs             # ✅ Router demo
│
└── target/                       # Build artifacts (auto-generated)
```

---

## 🔧 Build Instructions

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install protoc (Option A - System package)
sudo apt-get install protobuf-compiler

# OR Option B - Manual (already done)
# Downloaded to /tmp/bin/protoc
```

### Build Commands
```bash
cd /home/paul/Documents/AIORC

# Check compilation
PROTOC=/tmp/bin/protoc cargo check

# Build in debug mode
PROTOC=/tmp/bin/protoc cargo build

# Build optimized release
PROTOC=/tmp/bin/protoc cargo build --release

# Run tests
PROTOC=/tmp/bin/protoc cargo test --lib

# Run specific binary
PROTOC=/tmp/bin/protoc cargo run --release --bin orchestrator_gateway
```

---

## 🧪 Testing Strategy

### Unit Tests
- Located: Test modules in each file marked with `#[cfg(test)]`
- Run: `PROTOC=/tmp/bin/protoc cargo test --lib`
- Status: Minimal tests written, comprehensive coverage needed

### Integration Tests (Manual)
- Start gateway server
- Start model sidecars
- Call HTTP endpoints
- Verify response format and latency

### Benchmark Tests
- Routing latency (<50ms target)
- Memory usage tracking
- Model swap performance (<400ms target)

---

## 📊 Architecture Summary

```
┌─────────────────────────────────────────────────────┐
│        HTTP REST Gateway (Axum)                      │
│  GET /health | GET /metrics | POST /query            │
└────────────────┬────────────────────────────────────┘
                 │
      ┌──────────┼──────────┐
      │          │          │
  ┌───▼───┐  ┌──▼────┐  ┌─▼──────┐
  │Router │  │Memory │  │Metrics │
  │Logic  │  │Mgr    │  │Collect │
  └───┬───┘  └───────┘  └────────┘
      │
  ┌───▼─────────────────────────┐
  │ Service Registry            │
  │ (Model Discovery)           │
  └───┬───────────────────────┬─┘
      │                       │
  ┌───▼────────┐   ┌─────────▼──┐
  │ gRPC       │   │   gRPC     │
  │ Sidecar 1  │   │  Sidecar N │
  │(llama.cpp) │   │(llama.cpp) │
  └────────────┘   └────────────┘
```

---

## 🎯 Key Design Decisions

1. **Rust + gRPC:** High performance, type safety, binary protocol efficiency
2. **Multi-stage routing:** Complexity score → Semantic analysis → Model selection
3. **Warm-swap memory:** LRU cache with NVMe acceleration for model weights
4. **Semantic caching:** Local vector DB for deduplication of similar prompts
5. **Modular architecture:** Each component independently testable and replaceable

---

## 📝 Notes for Next Developer

### Important Implementation Details

1. **Error Handling:** Always convert `OrchestratorError` to `tonic::Status` at service boundaries
2. **Async/Await:** Use `tokio::spawn` for non-blocking tasks
3. **Metrics:** Update `MetricsCollector` after each request for observability
4. **Configuration:** Load from `config.json` at startup, hot-reload not yet implemented
5. **Model Registration:** Use `ServiceRegistry::register()` before routing requests

### Debugging Tips

1. Check compilation with: `PROTOC=/tmp/bin/protoc cargo check`
2. Run routing demo: `PROTOC=/tmp/bin/protoc cargo run --release --bin router`
3. Enable logging: Set `RUST_LOG=debug` environment variable
4. Monitor metrics: `curl http://localhost:9090/metrics`

### Common Pitfalls

1. **Float comparisons:** Always use `partial_cmp()` for f32, not `cmp()`
2. **gRPC trait methods:** Return `Status`, not custom error types
3. **DashMap iteration:** Be careful with reference lifetime
4. **String cloning:** Clone before move in async blocks

---

## 📞 Project Context

**Original Concept:** Autonomous Model-Agnostic Logic Switch (AMALS)  
**Goal:** Prove enterprise-grade AI doesn't require massive models or cloud infrastructure  
**Target:** Consumer-grade hardware (RTX 3060, 12GB VRAM)  
**Performance Target:**
- Routing latency: <50ms
- Model swap: <400ms
- Cache hit rate: >70%
- Throughput: >50 RPS on 1B models

---

## 🚀 Current Handoff Status

**Ready to Hand Off:** ✅ YES (with pending fixes)

**What's Working:**
- Project structure complete
- All source files written
- Configuration system ready
- Test framework in place

**What Needs Fixing:**
- 20 compilation errors (mostly trait/type issues)
- Placeholder inference engine
- Mock metrics implementation
- Limited unit tests

**Estimated Time to Working Build:** 4-6 hours for experienced Rust developer
**Estimated Time to MVP (with real llama.cpp):** 2-3 days

---

**Last Updated:** May 13, 2026  
**Status:** Ready for continuation with compilation fixes prioritized
