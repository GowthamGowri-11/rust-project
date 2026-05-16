# ML Integration Guide

## Overview

RustFlow-AI supports ONNX models for traffic prediction and congestion detection.

## Model Requirements

- Format: ONNX
- Input: Traffic features (bandwidth, latency, packet loss, etc.)
- Output: Predictions (congestion probability, recommended actions)

## Loading Models

Place ONNX models in the `models/` directory:

```
models/
├── traffic_predictor.onnx
├── congestion_detector.onnx
└── flow_classifier.onnx
```

Configure in `.env`:
```bash
ML_MODEL_PATH=/models/traffic_predictor.onnx
```

## Feature Engineering

The analytics crate extracts features from raw network data:

- Bandwidth utilization
- Latency percentiles
- Packet loss rate
- Flow count
- Traffic patterns

## Inference

```rust
use ml_engine::{MlEngineService, ModelConfig};

let ml_engine = MlEngineService::new();
ml_engine.load_model(config).await?;

let result = ml_engine.predict_congestion(features).await?;
```

## Training Models

Train models using Python/PyTorch, then export to ONNX:

```python
import torch.onnx

torch.onnx.export(
    model,
    dummy_input,
    "traffic_predictor.onnx",
    input_names=['features'],
    output_names=['predictions']
)
```

## Performance

- Inference latency: <10ms
- Batch processing supported
- Multi-threaded execution

## Future Enhancements

- Graph Neural Networks (GNN) for topology-aware predictions
- Online learning support
- Model versioning and A/B testing
