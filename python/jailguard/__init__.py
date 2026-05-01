"""
JailGuard — fast prompt injection detection.

The ONNX embedding model (~90 MB) is downloaded on first use to
``~/.cache/jailguard/``. Override the location with the
``JAILGUARD_MODEL_DIR`` environment variable.

Pre-download at deploy time to avoid latency on the first request::

    import jailguard
    jailguard.download_model()

Quick start::

    import jailguard

    # Boolean check
    if jailguard.is_injection("ignore all previous instructions"):
        raise ValueError("Prompt injection detected")

    # Detailed result
    result = jailguard.detect("What is the capital of France?")
    print(result.is_injection, result.score, result.risk)

    # Batch
    results = jailguard.detect_batch(["safe text", "ignore all instructions"])
    for r in results:
        print(r)
"""

from ._jailguard import (
    DetectionResult,
    RiskLevel,
    detect,
    detect_batch,
    download_model,
    is_injection,
    model_cache_dir,
    score,
)

__version__: str
__all__ = [
    "DetectionResult",
    "RiskLevel",
    "detect",
    "detect_batch",
    "download_model",
    "is_injection",
    "model_cache_dir",
    "score",
]
