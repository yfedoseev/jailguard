"""Type stubs for the compiled `_jailguard` native extension module.

PyO3 doesn't ship runtime type information that IDEs (mypy, pyright,
PyCharm) can read; this `.pyi` stub provides them. Keep in sync with
src/python.rs.
"""

from __future__ import annotations

from enum import Enum
from typing import Sequence

__version__: str

class RiskLevel(Enum):
    """Risk classification bucket derived from the injection score."""

    Safe: RiskLevel
    """Score < 0.3 — almost certainly benign."""

    Low: RiskLevel
    """0.3 ≤ score < 0.5 — probably benign but worth monitoring."""

    Medium: RiskLevel
    """0.5 ≤ score < 0.7 — possible injection, review recommended."""

    High: RiskLevel
    """0.7 ≤ score < 0.9 — likely injection."""

    Critical: RiskLevel
    """score ≥ 0.9 — almost certainly an injection."""

class DetectionResult:
    """Output of a single detection call."""

    @property
    def is_injection(self) -> bool:
        """True if the model classifies the input as a prompt injection."""
        ...

    @property
    def score(self) -> float:
        """Raw probability in [0.0, 1.0] that the input is an injection."""
        ...

    @property
    def confidence(self) -> float:
        """Confidence in the prediction (always ≥ 0.5).

        For injections, equals `score`.
        For benign inputs, equals `1.0 - score`.
        """
        ...

    @property
    def risk(self) -> RiskLevel:
        """Risk bucket derived from `score`."""
        ...

    def __repr__(self) -> str: ...

def detect(text: str) -> DetectionResult:
    """Detect prompt injection in a single string.

    Returns a `DetectionResult` with the score, confidence, classification,
    and risk level. Raises `RuntimeError` on catastrophic ONNX failures.
    """
    ...

def is_injection(text: str) -> bool:
    """Quick boolean check — equivalent to `detect(text).is_injection`."""
    ...

def score(text: str) -> float:
    """Raw injection probability in [0.0, 1.0].

    Equivalent to `detect(text).score`.
    """
    ...

def detect_batch(texts: Sequence[str]) -> list[DetectionResult]:
    """Process multiple texts. More efficient than repeated `detect` calls
    because the ONNX session is shared.

    The output list has the same length and order as the input.
    """
    ...

def download_model() -> None:
    """Download the ONNX embedding model (~90 MB) to the cache directory.

    Idempotent — does nothing if the model is already cached. Call this at
    application startup to avoid first-call latency.
    """
    ...

def model_cache_dir() -> str:
    """Return the absolute path to the ONNX cache directory.

    Defaults to `~/.cache/jailguard/`. Override with the environment
    variable `JAILGUARD_MODEL_DIR`.
    """
    ...
