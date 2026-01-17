# JailGuard Deployment Guide

## Overview

This guide covers deploying JailGuard in production environments, including Docker, Kubernetes, monitoring, and scaling strategies.

## Local Development Setup

### Prerequisites

- Rust 1.70+ ([Install](https://rustup.rs/))
- Cargo package manager (included with Rust)

### Build from Source

```bash
# Clone repository
git clone https://github.com/yourusername/jailguard.git
cd jailguard

# Build library
cargo build --release

# Run tests
cargo test --release

# Build documentation
cargo doc --open
```

## Docker Deployment

### Single Container Deployment

#### Dockerfile

```dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy from builder
COPY --from=builder /app/target/release/jailguard /usr/local/bin/

# Create app user
RUN useradd -m -u 1000 appuser
USER appuser

EXPOSE 8080

ENTRYPOINT ["jailguard"]
```

#### Building and Running

```bash
# Build Docker image
docker build -t jailguard:latest .

# Run container
docker run -d \
  --name jailguard \
  -p 8080:8080 \
  -v $(pwd)/config:/app/config \
  -e RUST_LOG=info \
  jailguard:latest

# View logs
docker logs -f jailguard

# Stop container
docker stop jailguard
```

### Docker Compose Setup

#### docker-compose.yml

```yaml
version: '3.8'

services:
  jailguard:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: jailguard
    ports:
      - "8080:8080"
    environment:
      RUST_LOG: info
      JAILGUARD_CONFIG: /app/config.toml
    volumes:
      - ./config:/app/config
      - ./logs:/app/logs
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'

  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    ports:
      - "3000:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: admin
    volumes:
      - grafana_data:/var/lib/grafana

volumes:
  prometheus_data:
  grafana_data:
```

#### Running with Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f jailguard

# Stop all services
docker-compose down

# Cleanup volumes
docker-compose down -v
```

## Kubernetes Deployment

### Prerequisites

- kubectl configured for your cluster
- Kubernetes 1.20+

### Kubernetes Manifests

#### namespace.yaml

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: jailguard
```

#### deployment.yaml

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: jailguard
  namespace: jailguard
  labels:
    app: jailguard
spec:
  replicas: 3
  selector:
    matchLabels:
      app: jailguard
  template:
    metadata:
      labels:
        app: jailguard
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000

      containers:
      - name: jailguard
        image: jailguard:latest
        imagePullPolicy: IfNotPresent

        ports:
        - name: http
          containerPort: 8080
          protocol: TCP

        env:
        - name: RUST_LOG
          value: "info"
        - name: JAILGUARD_CONFIG
          value: /etc/jailguard/config.toml

        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "1000m"

        livenessProbe:
          httpGet:
            path: /health
            port: http
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3

        readinessProbe:
          httpGet:
            path: /ready
            port: http
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2

        volumeMounts:
        - name: config
          mountPath: /etc/jailguard
          readOnly: true
        - name: logs
          mountPath: /app/logs

      volumes:
      - name: config
        configMap:
          name: jailguard-config
      - name: logs
        emptyDir: {}

      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - jailguard
              topologyKey: kubernetes.io/hostname
```

#### service.yaml

```yaml
apiVersion: v1
kind: Service
metadata:
  name: jailguard
  namespace: jailguard
  labels:
    app: jailguard
spec:
  type: ClusterIP
  selector:
    app: jailguard
  ports:
  - name: http
    port: 80
    targetPort: http
    protocol: TCP
```

#### config-map.yaml

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: jailguard-config
  namespace: jailguard
data:
  config.toml: |
    [jailguard]
    enable_spotlighting = true
    enable_detection = true
    enable_task_tracking = true
    enable_privilege_context = true
    enable_output_validation = true
    enable_monitoring = true
    block_threshold = 0.7
    strict_mode = false

    [server]
    host = "0.0.0.0"
    port = 8080

    [logging]
    level = "info"
    format = "json"
```

#### horizontal-pod-autoscaler.yaml

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: jailguard-hpa
  namespace: jailguard
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: jailguard
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Kubernetes Deployment Commands

```bash
# Create namespace
kubectl apply -f namespace.yaml

# Deploy JailGuard
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f config-map.yaml
kubectl apply -f horizontal-pod-autoscaler.yaml

# Check deployment status
kubectl get pods -n jailguard
kubectl describe pod <pod-name> -n jailguard

# View logs
kubectl logs -f deployment/jailguard -n jailguard

# Port forward for local testing
kubectl port-forward svc/jailguard 8080:80 -n jailguard

# Scale manually
kubectl scale deployment jailguard --replicas=5 -n jailguard

# Update configuration
kubectl edit configmap jailguard-config -n jailguard
# Then trigger rolling update:
kubectl rollout restart deployment/jailguard -n jailguard
```

## Monitoring & Observability

### Prometheus Configuration

#### prometheus.yml

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    monitor: 'jailguard'

scrape_configs:
  - job_name: 'jailguard'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 5s
```

### Key Metrics to Monitor

```
# Detection metrics
jailguard_detection_total          # Total detections
jailguard_injection_detected       # Injections detected
jailguard_benign_allowed           # Benign requests allowed
jailguard_false_positives          # False positive rate

# Performance metrics
jailguard_detection_latency_ms     # Detection latency (histogram)
jailguard_throughput_rps           # Requests per second
jailguard_memory_usage_bytes       # Memory footprint

# Confidence metrics
jailguard_confidence_distribution  # Confidence score histogram
jailguard_average_confidence       # Average confidence score

# System metrics
jailguard_errors_total             # Error count
jailguard_uptime_seconds           # Service uptime
```

### Grafana Dashboard

Sample dashboard JSON for Grafana:

```json
{
  "dashboard": {
    "title": "JailGuard Monitoring",
    "panels": [
      {
        "title": "Detection Rate",
        "targets": [
          {
            "expr": "rate(jailguard_detection_total[5m])"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Latency (p95)",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, jailguard_detection_latency_ms)"
          }
        ],
        "type": "gauge"
      },
      {
        "title": "False Positive Rate",
        "targets": [
          {
            "expr": "jailguard_false_positives / jailguard_detection_total"
          }
        ],
        "type": "stat"
      },
      {
        "title": "Throughput",
        "targets": [
          {
            "expr": "rate(jailguard_detection_total[1m])"
          }
        ],
        "type": "graph"
      }
    ]
  }
}
```

## Health Checks

### Health Check Endpoint

```rust
// GET /health - Liveness probe
// Returns 200 if service is running
pub fn health_check() -> Result<String> {
    Ok("OK".to_string())
}

// GET /ready - Readiness probe
// Returns 200 if service is ready for traffic
pub fn readiness_check() -> Result<String> {
    // Check dependencies, memory, etc.
    Ok("READY".to_string())
}
```

## Logging Configuration

### Structured Logging

```rust
use tracing::{info, warn, error};

fn process_request(input: &str) {
    info!(
        input_length = input.len(),
        "Processing request"
    );

    let result = jailguard.check_input(input, &context);

    if result.allowed {
        info!(
            session_id = result.session_id,
            confidence = result.detection.as_ref().map(|d| d.confidence),
            "Input allowed"
        );
    } else {
        warn!(
            session_id = result.session_id,
            reason = result.reason,
            "Input blocked"
        );
    }
}
```

### Log Aggregation with ELK Stack

```yaml
# docker-compose.yml addition
elasticsearch:
  image: docker.elastic.co/elasticsearch/elasticsearch:latest
  environment:
    - discovery.type=single-node

kibana:
  image: docker.elastic.co/kibana/kibana:latest
  ports:
    - "5601:5601"

filebeat:
  image: docker.elastic.co/beats/filebeat:latest
  volumes:
    - /var/log/jailguard:/var/log/jailguard:ro
    - ./filebeat.yml:/usr/share/filebeat/filebeat.yml
```

## Security Hardening

### Network Policy

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: jailguard-network-policy
  namespace: jailguard
spec:
  podSelector:
    matchLabels:
      app: jailguard
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          role: frontend
  egress:
  - to:
    - podSelector: {}
```

### Pod Security Policy

```yaml
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: jailguard-psp
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
  - ALL
  volumes:
  - 'configMap'
  - 'emptyDir'
  - 'projected'
  - 'secret'
  - 'downwardAPI'
  - 'persistentVolumeClaim'
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'MustRunAs'
  supplementalGroups:
    rule: 'RunAsAny'
```

## Performance Tuning

See [Performance Tuning Guide](./PERFORMANCE_TUNING.md) for:
- CPU optimization
- Memory optimization
- Latency reduction
- Throughput scaling
- Caching strategies

## Troubleshooting

### High CPU Usage

```bash
# Check resource usage
kubectl top pod -n jailguard

# Profile with perf
cargo install flamegraph
cargo flamegraph --release

# Disable expensive layers temporarily
kubectl set env deployment/jailguard \
  ENABLE_TASK_TRACKING=false \
  ENABLE_MONITORING=false
```

### High Memory Usage

```bash
# Check memory allocation
valgrind --leak-check=full ./jailguard

# Monitor with metrics
kubectl logs -f deployment/jailguard | grep memory
```

### Slow Responses

```bash
# Check latency metrics
kubectl exec -it pod/jailguard -n jailguard -- curl localhost:8080/metrics

# Increase replicas
kubectl scale deployment jailguard --replicas=5 -n jailguard

# Check for bottlenecks
kubectl top nodes
```

## Backup & Recovery

### Configuration Backup

```bash
# Backup ConfigMap
kubectl get configmap jailguard-config -n jailguard -o yaml > backup.yaml

# Restore ConfigMap
kubectl apply -f backup.yaml
```

### Database Snapshots

```bash
# If using persistent storage
kubectl get pvc -n jailguard
kubectl snapshot create jailguard-backup -n jailguard
```

## Upgrade Process

```bash
# Update Docker image
docker build -t jailguard:v2.0 .
docker push your-registry/jailguard:v2.0

# Update Kubernetes deployment
kubectl set image deployment/jailguard \
  jailguard=your-registry/jailguard:v2.0 \
  -n jailguard

# Check rollout status
kubectl rollout status deployment/jailguard -n jailguard

# Rollback if needed
kubectl rollout undo deployment/jailguard -n jailguard
```

## Production Checklist

- [ ] Health checks configured and tested
- [ ] Monitoring and alerting set up
- [ ] Log aggregation configured
- [ ] Resource limits set
- [ ] Security policies applied
- [ ] Load testing completed
- [ ] Backup strategy in place
- [ ] Incident response plan documented
- [ ] Documentation deployed
- [ ] Team trained on deployment

## Support & Resources

- GitHub Issues: Report bugs and feature requests
- Documentation: See [docs/](../docs/) directory
- Performance Tuning: See [PERFORMANCE_TUNING.md](./PERFORMANCE_TUNING.md)
- Integration: See [INTEGRATION_GUIDE.md](./INTEGRATION_GUIDE.md)
