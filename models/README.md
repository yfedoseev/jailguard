# Pre-trained Models

This directory contains pre-trained JailGuard prompt injection detection models in multiple formats.

## Available Formats

| Format | File | Size | Use Case |
|--------|------|------|----------|
| **JSON** | `jailguard_injection_detector.json` | 1.6 MB | Direct loading in Python, JavaScript, Rust |
| **SafeTensors** | `jailguard_injection_detector.safetensors` | 795 B | Hugging Face Hub integration |
| **ONNX Metadata** | `jailguard_injection_detector.onnx.metadata.json` | 1.4 KB | Convert to ONNX for mobile/web |

## Performance Metrics

Verified on held-out test set (1,875 samples):

| Metric | Value |
|--------|-------|
| **Accuracy** | 99.62% |
| **Precision** | 99.97% |
| **Recall** | 98.12% |
| **Specificity** | 99.99% |
| **F1 Score** | 99.04% |

## Model Architecture

```
Input: 384-dim embedding (MiniLM-L6-v2)
  ↓
Dense Layer: 384 → 256 (ReLU, Dropout 0.2)
  ↓
Dense Layer: 256 → 128 (ReLU, Dropout 0.2)
  ↓
Output Layer: 128 → 1 (Sigmoid)
  ↓
Output: Injection probability [0.0, 1.0]
```

## Usage Examples

### Python
```python
from loaders.jailguard_loader import JailGuardModelJSON

model = JailGuardModelJSON("models/jailguard_injection_detector.json")
confidence = model.predict(embedding)  # Returns 0.0-1.0
is_injection = confidence > 0.5
```

### JavaScript
```javascript
const { JailGuardModelJSON } = require('./loaders/jailguard_loader.js');

const model = new JailGuardModelJSON("models/jailguard_injection_detector.json");
const confidence = model.predict(embedding);
const isInjection = confidence > 0.5;
```

### Rust (Native)
```rust
use jailguard::training::NeuralBinaryNetwork;

let model = NeuralBinaryNetwork::load("models/jailguard_injection_detector.json")?;
let confidence = model.forward_eval(&embedding);
let is_injection = confidence > 0.5;
```

## Training Your Own Model

To train a new model:

```bash
cargo run --example train_neural_binary --release
```

Or with the full pipeline:

```bash
cargo run --example evaluate_on_test_set --release
```

## ONNX Conversion

To convert to ONNX format for cross-platform deployment:

```bash
python scripts/convert_to_onnx.py models/jailguard_injection_detector.json
```

Then use with:
- Python: ONNX Runtime
- iOS: CoreML (via ONNX → CoreML conversion)
- Android: TensorFlow Lite / NNAPI
- Web: ONNX.js

## Hugging Face Hub

To upload to Hugging Face Hub:

```bash
huggingface-cli upload jailguard models/jailguard_injection_detector.safetensors
```
