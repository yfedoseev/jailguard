# Collection Pipeline Deployment Guide

## Overview

This guide covers deploying JailGuard's data collection pipeline to gather real-world prompt injection and jailbreak attempts from 5 sources, targeting 100-500 new samples per week.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Collection Pipeline                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Source Collectors (Rate-Limited API Calls)                    │
│  ├─ Reddit r/jailbreak (60 req/min)                            │
│  ├─ GitHub Adversarial (60-5000 req/hour)                      │
│  ├─ Stack Overflow (300 req/day)                               │
│  ├─ arXiv Papers (3 req/sec)                                   │
│  └─ Manual Submissions (Unlimited)                             │
│                                                                 │
│  Processing Pipeline                                            │
│  ├─ Validation (Quality checking)                              │
│  ├─ Deduplication (10/10 tests passing)                        │
│  └─ Labeling (7-way attack classification)                     │
│                                                                 │
│  Output                                                         │
│  └─ Validated Samples → Training Data Repository               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Expected Data Volume

```
Source              Requests/Limit    Expected/Day    Weekly      Monthly
────────────────────────────────────────────────────────────────────────
Reddit              60/min            200 samples     1,400       6,000
GitHub              5000/hour         150 samples     1,050       4,500
StackOverflow       300/day           50 samples        350       1,500
arXiv               3/sec             100 samples       700       3,000
Manual              Unlimited         50 samples        350       1,500
────────────────────────────────────────────────────────────────────────
TOTAL                                 550 samples     3,850      16,500
```

## Prerequisites

### Environment Setup

```bash
# 1. Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 2. Clone JailGuard repository
git clone https://github.com/your-org/jailguard.git
cd jailguard

# 3. Install dependencies
cargo build --release
```

### API Credentials

You'll need credentials for the data sources:

#### Reddit API
```bash
# 1. Go to https://www.reddit.com/prefs/apps
# 2. Create a new application (type: script)
# 3. Get credentials:
#    - Client ID: under app name
#    - Client Secret: under app name

export REDDIT_CLIENT_ID="your_client_id"
export REDDIT_CLIENT_SECRET="your_client_secret"
```

#### GitHub API (Optional but Recommended)
```bash
# 1. Go to https://github.com/settings/tokens
# 2. Generate new token with "public_repo" scope
# 3. Copy the token

export GITHUB_TOKEN="your_github_token"
# With token: 5000 requests/hour
# Without token: 60 requests/hour
```

#### Stack Overflow API (Optional)
```bash
# 1. Register at https://stackapps.com/apps/register
# 2. Create an application
# 3. Get your API key

export STACKOVERFLOW_API_KEY="your_api_key"
```

#### arXiv API
- No credentials needed (public API)
- Rate limit: 3 requests/second

#### Manual Community Submissions
```bash
# Configure webhook for community submissions (optional)
export JAILGUARD_WEBHOOK_URL="https://your-server/webhook/submissions"
```

## Deployment Steps

### Step 1: Review Deployment Configuration

Run the deployment configuration example:

```bash
cargo run --example deploy_collection_pipeline --release
```

This will show:
- Configuration status
- API credentials status
- Rate limit settings
- Expected daily sample volume
- Deployment checklist

### Step 2: Export API Credentials

```bash
# Reddit
export REDDIT_CLIENT_ID="your_client_id"
export REDDIT_CLIENT_SECRET="your_client_secret"

# GitHub (optional but recommended)
export GITHUB_TOKEN="your_github_token"

# Stack Overflow (optional)
export STACKOVERFLOW_API_KEY="your_api_key"

# Manual webhook (optional)
export JAILGUARD_WEBHOOK_URL="https://your-server/webhook"
```

### Step 3: Create Output Directories

```bash
mkdir -p data/collected_samples
mkdir -p logs
```

### Step 4: Start Collection Pipeline

#### Option A: Development (Manual Control)

```bash
# Run with verbose logging
RUST_LOG=debug cargo run --example deploy_collection_pipeline --release
```

#### Option B: Production (Continuous)

Create a systemd service file:

```ini
# /etc/systemd/system/jailguard-collection.service
[Unit]
Description=JailGuard Collection Pipeline
After=network.target

[Service]
Type=simple
User=jailguard
WorkingDirectory=/opt/jailguard
Environment="RUST_LOG=info"
Environment="REDDIT_CLIENT_ID=your_client_id"
Environment="REDDIT_CLIENT_SECRET=your_client_secret"
Environment="GITHUB_TOKEN=your_github_token"
ExecStart=/opt/jailguard/target/release/jailguard-collection-daemon
Restart=always
RestartSec=60

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable jailguard-collection
sudo systemctl start jailguard-collection
```

#### Option C: Docker (Container)

```dockerfile
FROM rust:latest
WORKDIR /app
COPY . .

ENV REDDIT_CLIENT_ID=${REDDIT_CLIENT_ID}
ENV REDDIT_CLIENT_SECRET=${REDDIT_CLIENT_SECRET}
ENV GITHUB_TOKEN=${GITHUB_TOKEN}
ENV RUST_LOG=info

RUN cargo build --release --example deploy_collection_pipeline

ENTRYPOINT ["./target/release/examples/deploy_collection_pipeline"]
```

Run:
```bash
docker build -t jailguard-collection:latest .
docker run -d \
  -e REDDIT_CLIENT_ID="your_client_id" \
  -e REDDIT_CLIENT_SECRET="your_client_secret" \
  -e GITHUB_TOKEN="your_github_token" \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/logs:/app/logs \
  jailguard-collection:latest
```

### Step 5: Monitor Collection

#### Check Logs

```bash
# Real-time monitoring
tail -f logs/collection_pipeline.log

# Statistics
grep "samples collected" logs/collection_pipeline.log | tail -20

# Errors
grep "error" logs/collection_pipeline.log
```

#### Check Collected Data

```bash
# List collected samples by source
ls -lh data/collected_samples/

# Count samples
find data/collected_samples -type f | wc -l

# View sample contents
head -n 20 data/collected_samples/reddit_samples.json
```

#### Monitor API Rate Limits

```bash
# Check rate limit status
grep "rate.*limit" logs/collection_pipeline.log

# Expected per day:
# Reddit: 60 req/min = ~86,400 req/day (plenty of quota)
# GitHub: 5000 req/hour = 120,000 req/day (plenty)
# Stack Overflow: 300 req/day (full quota used)
# arXiv: 3 req/sec = 259,200 req/day (plenty)
```

### Step 6: Process Collected Data

#### Deduplication

The collection pipeline automatically deduplicates samples:

```bash
# Run deduplication test
cargo test --lib collection::deduplication --release

# Deduplication is configured with:
# - Algorithm: Longest Common Subsequence (LCS)
# - Threshold: 0.92 (92% similarity = duplicate)
# - Expected: 20-30% of raw samples are duplicates
```

#### Attack Type Labeling

Collected samples are automatically labeled as one of 7 attack types:

```
1. Role-play injection
2. Instruction override
3. Context manipulation
4. Output manipulation
5. Encoding/obfuscation
6. Jailbreak patterns
7. Benign (no attack)
```

#### View Processed Data

```bash
# List processed/labeled samples
ls -lh data/labeled_samples/

# Check labeling distribution
python3 - <<'EOF'
import json
from collections import Counter

with open('data/labeled_samples/labeled.json') as f:
    samples = json.load(f)

attack_types = [s.get('attack_type') for s in samples]
print(Counter(attack_types))
EOF
```

## Monitoring & Maintenance

### Daily Tasks

```bash
# 1. Check collection log for errors
grep "error\|ERROR\|panic" logs/collection_pipeline.log

# 2. Verify samples were collected
find data/collected_samples -type f -mtime -1 | wc -l
# Should be > 50 (expecting ~550/day)

# 3. Monitor API rate limits
grep "rate.*limit\|quota" logs/collection_pipeline.log

# 4. Check disk usage
du -sh data/
# Each sample ~200 bytes, so 16,500/month ≈ 3-4 MB/month
```

### Weekly Tasks

```bash
# 1. Generate collection report
cargo run --example collection_stats_report --release

# 2. Review attack type distribution
python3 scripts/analyze_attack_types.py

# 3. Check data quality
cargo test --lib collection --release

# 4. Archive old samples
tar -czf data/archived/samples_$(date +%Y%m%d).tar.gz data/collected_samples/
```

### Monthly Tasks

```bash
# 1. Comprehensive analysis
cargo run --example monthly_collection_analysis --release

# 2. Update model with new samples
# (See Training Guide: TRAINING.md)

# 3. Review and update detection model
cargo run --example fine_tune_with_collected_data --release

# 4. Backup collected data
rsync -avz data/collected_samples/ backup@server:/backups/jailguard/
```

## Health Checks

### Collection Status

```bash
#!/bin/bash
# Check if collection is running

PROCESS_COUNT=$(pgrep -f "jailguard.*collection" | wc -l)

if [ $PROCESS_COUNT -eq 0 ]; then
    echo "⚠️  Collection process not running"
    exit 1
fi

# Check last collection timestamp
LAST_COLLECTED=$(stat -c %Y data/collected_samples/reddit_samples.json 2>/dev/null)
NOW=$(date +%s)
AGE=$((NOW - LAST_COLLECTED))

if [ $AGE -gt 86400 ]; then  # 24 hours
    echo "⚠️  No collections in last 24 hours"
    exit 1
fi

echo "✅ Collection pipeline healthy"
exit 0
```

Run health check:
```bash
# Via cron
0 * * * * /opt/jailguard/health_check.sh

# Or manually
./health_check.sh && echo "Healthy" || echo "Issues detected"
```

### Data Quality Metrics

```rust
// Check data quality
let validator = SampleValidator::new(ValidationConfig::default());

for sample in collected_samples {
    match validator.validate(&sample.text) {
        Ok(result) => println!("✅ Valid: {}", result.quality_score),
        Err(e) => println!("❌ Invalid: {:?}", e),
    }
}
```

## Troubleshooting

### Issue: API Rate Limit Exceeded

**Symptom**: Errors like "429 Too Many Requests"

**Solution**:
```rust
// The rate limiter is automatically configured:
// - Reddit: 100ms delay between requests
// - GitHub: 50ms delay (authenticated)
// - Stack Overflow: 1000ms delay
// - arXiv: 333ms delay (3 req/sec)

// If still hitting limits, wait 1 hour and retry
```

### Issue: No Samples Collected

**Symptom**: `data/collected_samples/` is empty

**Solutions**:

1. **Check credentials**
   ```bash
   echo $REDDIT_CLIENT_ID
   echo $GITHUB_TOKEN
   ```

2. **Check logs**
   ```bash
   tail -f logs/collection_pipeline.log | grep "error\|failed\|unauthorized"
   ```

3. **Verify network connectivity**
   ```bash
   # Test Reddit API
   curl -u "$REDDIT_CLIENT_ID:$REDDIT_CLIENT_SECRET" \
     -d "grant_type=client_credentials" \
     https://www.reddit.com/api/v1/access_token

   # Test GitHub API
   curl -H "Authorization: token $GITHUB_TOKEN" \
     https://api.github.com/user
   ```

4. **Test collectors in isolation**
   ```bash
   # Manual test
   cargo test --lib reddit_collector --release -- --nocapture
   cargo test --lib github_collector --release -- --nocapture
   ```

### Issue: High Duplication Rate (>50%)

**Symptom**: Deduplication removing most samples

**Solution**:
1. Increase LCS threshold (reduce sensitivity)
2. Collect from more diverse sources
3. Run longer to accumulate samples from different time periods

```rust
// Adjust deduplication config
let config = DeduplicationConfig {
    similarity_threshold: 0.95,  // Was 0.92, now more lenient
    algorithm: DeduplicationAlgorithm::LCS,
};
```

### Issue: High Memory Usage

**Symptom**: Process consuming >1GB RAM

**Solution**:
1. Process samples in batches
2. Clear old samples periodically
3. Reduce collection frequency

```bash
# Archive and delete old samples
find data/collected_samples -type f -mtime +30 -delete
```

## Expected Results

### Week 1 Baseline
- **Samples collected**: ~550 (varying by source availability)
- **Unique samples**: ~420 (20-30% duplicates)
- **Collection uptime**: >99%
- **Processing latency**: <1 minute

### Month 1 Targets
- **Total samples**: 16,500
- **Unique samples**: 11,500-13,000
- **Attack type distribution**: Balanced across 7 types
- **Data quality**: >90% pass validation
- **Storage used**: 3-4 MB

### Continuous Operation
- **Daily collection**: 500-550 samples
- **Monthly growth**: ~16,500 new unique samples
- **Quarterly improvement**: Improved model accuracy from training data
- **Annual data volume**: ~200,000 samples

## Integration with Model Training

### Step 1: Collect Real Data (This Phase)
- Gather 200-1000 real samples from production sources
- Deduplicate and label
- Validate quality

### Step 2: Augment Training Data
```bash
# Combine collected data with synthetic data
cargo run --example combine_datasets --release \
  --synthetic data/synthetic_samples.json \
  --real data/collected_samples/*.json \
  --output data/combined_training_set.json
```

### Step 3: Retrain Model
```bash
# Fine-tune on combined dataset
cargo run --example fine_tune_stage4 --release \
  --dataset data/combined_training_set.json
```

### Step 4: Evaluate Improvement
```bash
# Compare accuracy before/after
cargo run --example evaluate_improvement --release
```

## Production Checklist

- [ ] All API credentials configured
- [ ] Output directories created (`data/`, `logs/`)
- [ ] Collection process started (systemd/Docker)
- [ ] Monitoring enabled (logs, metrics)
- [ ] Health checks configured (cron)
- [ ] Backup strategy in place
- [ ] Team notified of collection
- [ ] First 24-hour collection verified (>50 samples)
- [ ] Data quality validated
- [ ] Weekly review scheduled

## References

- **Integration Guide**: See [INTEGRATION_GUIDE.md](./INTEGRATION_GUIDE.md)
- **Deployment Guide**: See [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md)
- **Performance Tuning**: See [PERFORMANCE_TUNING.md](./PERFORMANCE_TUNING.md)
- **Training Guide**: See [TRAINING.md](./TRAINING.md)

## Support

For issues or questions about collection pipeline:
1. Check logs: `logs/collection_pipeline.log`
2. Review troubleshooting section above
3. Run diagnostic example: `cargo run --example deploy_collection_pipeline`
4. File issue on GitHub with logs and configuration

---

**Expected Result**: Production collection pipeline running and gathering 500-550 real-world samples per day for continuous model improvement.
