"""Async wrappers for JailGuard.

Provides async equivalents of the four core functions plus an
``AsyncDetector`` class that holds a private executor for fan-out
batch workloads.

Each wrapper runs the (CPU-bound, ONNX-backed) detector call in a
background thread via ``run_in_executor`` so the asyncio event loop
is never blocked.

Example::

    import asyncio
    from jailguard import detect_async, is_injection_async

    async def main():
        result = await detect_async("ignore previous instructions")
        if result.is_injection:
            print(f"BLOCKED — score={result.score:.2f}")

        # Fan-out
        results = await asyncio.gather(*[
            detect_async(p) for p in user_prompts
        ])

    asyncio.run(main())

For higher throughput, use :class:`AsyncDetector` which keeps a
dedicated thread pool around::

    async with AsyncDetector(max_workers=8) as det:
        results = await det.detect_batch(prompts)
"""

from __future__ import annotations

import asyncio
from concurrent.futures import ThreadPoolExecutor
from typing import Iterable, Sequence

from ._jailguard import (
    DetectionResult,
    detect as _detect,
    detect_batch as _detect_batch,
    is_injection as _is_injection,
    score as _score,
)


# ---------------------------------------------------------------------------
# Module-level helpers — use the asyncio default executor.
# ---------------------------------------------------------------------------


async def detect_async(text: str) -> DetectionResult:
    """Async wrapper around :func:`jailguard.detect`."""
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(None, _detect, text)


async def is_injection_async(text: str) -> bool:
    """Async wrapper around :func:`jailguard.is_injection`."""
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(None, _is_injection, text)


async def score_async(text: str) -> float:
    """Async wrapper around :func:`jailguard.score`."""
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(None, _score, text)


async def detect_batch_async(texts: Sequence[str]) -> list[DetectionResult]:
    """Async wrapper around :func:`jailguard.detect_batch`."""
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(None, _detect_batch, list(texts))


# ---------------------------------------------------------------------------
# AsyncDetector — for workloads that want a dedicated worker pool.
# ---------------------------------------------------------------------------


class AsyncDetector:
    """Owned thread-pool wrapper around the JailGuard detector.

    Use as an async context manager so the executor is shut down
    cleanly when you're done::

        async with AsyncDetector(max_workers=8) as det:
            r = await det.detect("ignore previous instructions")
    """

    def __init__(self, max_workers: int = 4) -> None:
        self._executor = ThreadPoolExecutor(
            max_workers=max_workers,
            thread_name_prefix="jailguard",
        )

    async def __aenter__(self) -> "AsyncDetector":
        return self

    async def __aexit__(self, exc_type, exc, tb) -> None:
        self.close()

    def close(self) -> None:
        """Shut down the worker thread pool."""
        self._executor.shutdown(wait=True)

    async def detect(self, text: str) -> DetectionResult:
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(self._executor, _detect, text)

    async def is_injection(self, text: str) -> bool:
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(self._executor, _is_injection, text)

    async def score(self, text: str) -> float:
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(self._executor, _score, text)

    async def detect_batch(
        self, texts: Iterable[str]
    ) -> list[DetectionResult]:
        loop = asyncio.get_running_loop()
        text_list = list(texts)
        return await loop.run_in_executor(
            self._executor, _detect_batch, text_list
        )
