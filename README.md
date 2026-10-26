<p align="center">
  <img src="docs/assets/logo.svg" alt="OpenAdServer" width="400"/>
</p>

<h1 align="center">OpenAdServer (Rust)</h1>

<p align="center">
  <strong>High-Performance Computational Advertising System with Native ML-Powered CTR Prediction</strong><br>
  <em>Sub-millisecond latency, zero-copy safety, and production-grade ad serving in pure Rust</em>
</p>

<p align="center">
  <a href="#-features">Features</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-architecture">Architecture</a> •
  <a href="docs/ARCHITECTURE_PIPELINE.md">Pipeline Docs</a> •
  <a href="#-benchmarks">Benchmarks</a> •
  <a href="#-api-reference">API Reference</a> •
  <a href="#-project-structure">Project Structure</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/rust-1.80+-orange.svg" alt="Rust 1.80+"/>
  <img src="https://img.shields.io/badge/axum-0.8-blue.svg" alt="Axum 0.8"/>
  <img src="https://img.shields.io/badge/tokio-async-green.svg" alt="Tokio Async"/>
  <img src="https://img.shields.io/badge/license-Apache%202.0-green.svg" alt="License"/>
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg" alt="PRs Welcome"/>
</p>

---

## ⚡ Why Rust OpenAdServer?

OpenAdServer has been rewritten from Python/FastAPI into **pure Rust** using **Axum**, **Tokio**, and **Sqlx**, delivering **sub-millisecond latency (<1ms)** and massive throughput capable of tens of thousands of QPS per node with minimal memory footprint (<30MB).

### Comparison

| Feature | OpenAdServer (Rust) | OpenAdServer (Python) | Revive Adserver | Google Ad Manager |
|---------|:-------------------:|:---------------------:|:---------------:|:-----------------:|
| **Core Language** | **Rust** (Async Tokio) | Python 3.11 (FastAPI) | PHP 8 | Proprietary |
| **P99 Latency** | **< 1.5ms** | ~10-15ms | > 50ms | N/A |
| **ML Inference** | **Native DeepFM & LR** (sub-ms) | PyTorch / Python | ❌ None | Blackbox |
| **Memory Footprint**| **~25 MB** | ~250 MB | ~150 MB | Cloud |
| **Self-Hosted** | ✅ 100% Control | ✅ | ✅ | ❌ |
| **Revenue Share** | **0% (Free)** | 0% | 0% | 20-30% 💰 |
| **Async Ecosystem** | Axum 0.8 + Sqlx + Redis | FastAPI + SQLAlchemy | Apache/Nginx | Proprietary |

---

## ✨ Features

### 🚀 Ultra Low-Latency Ad Serving
- **Async Tokio & Axum 0.8** — Blazing fast non-blocking HTTP pipeline.
- **Sub-Millisecond Pipeline** — End-to-end Retrieval, Filtering, ML Inference, and Ranking in under 1ms.
- **Smart Targeting Engine** — Multi-dimensional matching: OS, Device, Geo (Country/City), Age, Gender, Interests, and App Categories with include/exclude rules.
- **Budget Pacing & Frequency Capping** — Atomic Redis-backed daily & hourly frequency caps and smooth budget pacing.

### 🤖 Native Machine Learning Engine
- **Pure Rust DeepFM & Logistic Regression** — Zero external C++/Python dependencies. Forward pass runs in <0.05ms directly in process memory!
- **Sparse Feature Hashing & Dense Embeddings** — Fast, collision-resistant feature hashing for user context and ad candidates.
- **Second-Order FM Interactions** — Vectorized dot-product interaction algebra for high-order feature combinations.
- **Ensemble & Statistical Predictors** — Built-in fallbacks with statistical moving averages and configurable exploration.

### 💰 Monetization & Auctions
- **Comprehensive eCPM Bidding** — Unified ranking for CPM, CPC, CPA, and oCPM bid types.
- **Second Price Auction (GSP)** — Compute optimal clearing prices for advertiser fair-play and publisher yield maximization.
- **Diversity & Exploration Re-rankers** — Maximal Marginal Relevance (MMR) for creative diversity and $\epsilon$-greedy exploration to bootstrap new ad campaigns.

### 📊 Observability & Reliability
- **Prometheus Metrics** — Native `/metrics` exporter for QPS, latency percentiles, error rates, and impression/click counts.
- **Graceful Degradation** — Multi-level caching (Redis + In-memory fallback) with resilient DB fallbacks.

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Rust OpenAdServer Flow                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   📱 Client Request                                                         │
│          │                                                                  │
│          ▼                                                                  │
│   ┌──────────────┐     ┌──────────────┐     ┌──────────────┐                │
│   │  Axum Router │────▶│ Targeting    │────▶│ Budget & Freq│                │
│   │ (HTTP / JSON)│     │ Retrieval    │     │ Filters      │                │
│   └──────────────┘     └──────────────┘     └──────────────┘                │
│                                                     │                       │
│                                                     ▼                       │
│   ┌──────────────┐     ┌──────────────┐     ┌──────────────┐                │
│   │ Response     │◀────│ MMR & Epsilon│◀────│ Native ML    │                │
│   │ JSON Format  │     │ Re-ranking   │     │ (DeepFM / LR)│                │
│   └──────────────┘     └──────────────┘     └──────────────┘                │
│          │                                                                  │
│          ▼                                                                  │
│   ┌──────────────┐     ┌──────────────┐                                     │
│   │ PostgreSQL   │     │ Redis Cache  │                                     │
│   │ (Campaigns)  │     │ (Stats/Caps) │                                     │
│   └──────────────┘     └──────────────┘                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 Quick Start

### 1. Prerequisites
- **Rust 1.80+** (Install via `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **Docker & Docker Compose** (Optional for local PostgreSQL and Redis)

### 2. Run with Docker Compose (Fastest)

```bash
# Clone the repository
git clone https://github.com/pysean/openadserver.git
cd openadserver

# Launch PostgreSQL, Redis, and OpenAdServer
docker compose up -d

# Verify server health
curl http://localhost:8000/health
```

### 3. Local Development

```bash
# Start local PostgreSQL and Redis containers
docker compose up -d postgres redis

# Check code compilation
make check

# Run all unit & integration tests
make test

# Start the server locally
make dev
```

---

## 📡 API Usage

### Ad Request (`POST /api/v1/ad/request`)

```bash
curl -X POST http://localhost:8000/api/v1/ad/request \
  -H "Content-Type: application/json" \
  -d '{
    "slot_id": "slot_banner_01",
    "user_id": "user_98765",
    "num_ads": 1,
    "device": {
      "os": "android",
      "os_version": "14.0",
      "model": "Pixel 8",
      "brand": "Google"
    },
    "geo": {
      "country": "ID",
      "city": "Jakarta"
    },
    "user_features": {
      "age": 28,
      "gender": "male",
      "interests": ["technology", "gaming"]
    }
  }'
```

**Response:**
```json
{
  "request_id": "req_01940b52-d6c1-7f8e-a2b1-6a4e32c842b1",
  "ads": [
    {
      "ad_id": "ad_1_101",
      "campaign_id": 1,
      "creative_id": 101,
      "creative": {
        "title": "Super Gadget Deal",
        "landing_url": "https://example.com/gadgets",
        "creative_type": "banner"
      },
      "tracking": {
        "impression_url": "http://localhost:8000/api/v1/event/track?type=impression&req=req_0194...&ad=1",
        "click_url": "http://localhost:8000/api/v1/event/track?type=click&req=req_0194...&ad=1",
        "conversion_url": "http://localhost:8000/api/v1/event/track?type=conversion&req=req_0194...&ad=1"
      }
    }
  ],
  "count": 1
}
```

### Event Tracking (`POST /api/v1/event/track` & `GET /api/v1/event/track`)

```bash
# POST event tracking
curl -X POST http://localhost:8000/api/v1/event/track \
  -H "Content-Type: application/json" \
  -d '{
    "request_id": "req_01940b52-d6c1-7f8e-a2b1-6a4e32c842b1",
    "ad_id": "ad_1_101",
    "event_type": "click",
    "user_id": "user_98765"
  }'

# Pixel GET tracking (compatible with standard web beacons)
curl "http://localhost:8000/api/v1/event/track?type=impression&req=req_0194&ad=1_101"
```

### IAB VAST 3.0 Video Ad Serving (`GET /api/v1/ad/vast`)

Standard XML endpoint for video players (Video.js, IMA SDK, ExoPlayer):

```bash
curl "http://localhost:8000/api/v1/ad/vast?slot_id=video_slot_01&os=android&country=ID"
```

**Response (`application/xml`):**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<VAST version="3.0">
  <Ad id="ad_1_101">
    <InLine>
      <AdSystem version="1.0">OpenAdServer</AdSystem>
      <AdTitle><![CDATA[Featured Video Ad]]></AdTitle>
      <Impression><![CDATA[http://localhost:8000/api/v1/event/track?type=impression&req=...&ad=1]]></Impression>
      <Creatives>
        <Creative id="cr_101" sequence="1">
          <Linear>
            <Duration>00:00:15</Duration>
            <TrackingEvents>
              <Tracking event="start"><![CDATA[http://localhost:8000/api/v1/event/track?type=video_start...]]></Tracking>
              <Tracking event="complete"><![CDATA[http://localhost:8000/api/v1/event/track?type=video_complete...]]></Tracking>
            </TrackingEvents>
            <VideoClicks>
              <ClickThrough><![CDATA[https://example.com/promo]]></ClickThrough>
            </VideoClicks>
            <MediaFiles>
              <MediaFile delivery="progressive" type="video/mp4" width="1920" height="1080">
                <![CDATA[https://cdn.example.com/ad_video.mp4]]>
              </MediaFile>
            </MediaFiles>
          </Linear>
        </Creative>
      </Creatives>
    </InLine>
  </Ad>
</VAST>
```

### OpenRTB 2.5 Real-Time Bidding (`POST /api/v1/openrtb2/bid`)

Standard DSP/SSP real-time bidding interface with floor pricing:

```bash
curl -X POST http://localhost:8000/api/v1/openrtb2/bid \
  -H "Content-Type: application/json" \
  -d '{
    "id": "rtb_req_1001",
    "imp": [{
      "id": "imp_1",
      "banner": { "w": 300, "h": 250 },
      "bidfloor": 2.50
    }],
    "device": { "os": "android", "geo": { "country": "ID" } }
  }'
```

Returns `HTTP 200 OK` with standard `OpenRtbBidResponse` (or `HTTP 204 No Content` if below bid floor).

### Campaign Management CRUD API

Manage advertisers, campaigns, targeting rules, and creatives dynamically:

```bash
# 1. Create an Advertiser
curl -X POST http://localhost:8000/api/v1/advertiser \
  -H "Content-Type: application/json" \
  -d '{"name": "Acme Brand", "contact_email": "ads@acme.com", "balance": 5000.0}'

# 2. Create a Campaign with Targeting Rules
curl -X POST http://localhost:8000/api/v1/campaign \
  -H "Content-Type: application/json" \
  -d '{
    "advertiser_id": 1,
    "name": "Summer Tech Promo",
    "bid_type": 1,
    "bid_amount": 5.0,
    "budget_daily": 200.0,
    "targeting_rules": [
      { "rule_type": "geo", "rule_value": { "countries": ["ID", "SG"] }, "is_include": true },
      { "rule_type": "os", "rule_value": { "values": ["android", "ios"] }, "is_include": true }
    ]
  }'

# 3. Create a Creative
curl -X POST http://localhost:8000/api/v1/creative \
  -H "Content-Type: application/json" \
  -d '{
    "campaign_id": 1,
    "title": "Summer Gadget Sale",
    "landing_url": "https://acme.com/summer",
    "image_url": "https://cdn.acme.com/banner300x250.jpg",
    "creative_type": 1,
    "width": 300,
    "height": 250
  }'

# 4. List / Get / Update / Delete
curl http://localhost:8000/api/v1/campaign/1
curl -X DELETE http://localhost:8000/api/v1/campaign/1
```

*(Note: Creating or updating campaigns automatically clears Redis active ad caches, so changes take effect in real time!)*

### Metrics & Observability

```bash
curl http://localhost:8000/metrics
```

---

## 📁 Project Structure

```
openadserver/
├── Cargo.toml                    # Rust workspace dependencies and features
├── configs/                      # YAML configuration files (default.yaml, dev.yaml)
├── deployment/
│   └── docker/
│       └── Dockerfile            # Optimized multi-stage Rust build container
├── docker-compose.yml            # Orchestration for PostgreSQL, Redis, Grafana, Prometheus
├── Makefile                      # Make targets for development and testing
├── src/
│   ├── main.rs                   # CLI entrypoint (serve, init-db, mock-data)
│   ├── lib.rs                    # Axum router mounting and AppState definition
│   ├── ad_server/                # HTTP layer
│   │   ├── middleware/           # Prometheus metrics and request timing
│   │   ├── routers/              # Axum handlers for /health, /ad, /event
│   │   └── services/             # Orchestration services (AdService, EventService)
│   ├── ml_engine/                # High-speed CTR Machine Learning in pure Rust
│   │   ├── models/               # DeepFM and Logistic Regression inference
│   │   └── serving/              # Online batch predictor and feature hashing
│   ├── rec_engine/               # Core recommendation & auction pipeline
│   │   ├── retrieval/            # Multi-dimensional targeting rules & caching
│   │   ├── filter/               # Budget, frequency cap, and quality filters
│   │   ├── ranking/              # eCPM calculator, second price auction, predictors
│   │   ├── reranking/            # MMR creative diversity & exploration re-rankers
│   │   └── engine.rs             # Pipeline orchestrator
│   ├── models/                   # SQLx database entities and domain enums
│   ├── schemas/                  # Serde request, response, and internal DTOs
│   └── common/                   # Database pool, Redis cache client, config, logger
└── tests/                        # Comprehensive integration test suite
    ├── test_api.rs               # End-to-end Axum HTTP endpoint tests
    ├── test_bidding.rs           # eCPM calculation and second price auction tests
    ├── test_filters.rs           # Budget, frequency, and quality filter tests
    ├── test_ml_models.rs         # DeepFM, LR, and online predictor tests
    ├── test_reranker.rs          # Diversity MMR and epsilon exploration tests
    └── test_retrieval.rs         # Targeting retrieval rule matching tests
```

---

## 🧪 Testing

OpenAdServer includes unit and integration tests covering the entire pipeline:

```bash
# Run all tests
cargo test

# Run a specific test suite
cargo test --test test_api
cargo test --test test_ml_models
cargo test --test test_bidding
```

---

## 📄 License

Apache License 2.0 — See [LICENSE](LICENSE) for details.
