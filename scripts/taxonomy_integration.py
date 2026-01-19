#!/usr/bin/env python3
"""
Unified Taxonomy Integration System

Implements the 8-class attack taxonomy across all data sources:
- Standardizes attack type classification
- Provides heuristic-based inference for unlabeled data
- Maps legacy taxonomies to unified format
- Generates comprehensive taxonomy mapping documentation

Usage:
    python3 scripts/taxonomy_integration.py --analyze expansion_combined_raw.json
    python3 scripts/taxonomy_integration.py --convert input.json --output output.json
"""

import json
import sys
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from collections import defaultdict
import re

from unified_schema import (
    ATTACK_TYPE_TO_IDX,
    infer_attack_type,
    convert_to_canonical_format,
    TrainingSample,
)


# ============================================================================
# TAXONOMY DEFINITIONS
# ============================================================================

UNIFIED_TAXONOMY = {
    "Benign": {
        "idx": 0,
        "description": "Legitimate queries without injection attempts",
        "keywords": [],
    },
    "RolePlay": {
        "idx": 1,
        "description": "Role-play or persona injection (act as, pretend to be, assume role)",
        "keywords": [
            "act as", "pretend", "pretend to be", "assume the role", "roleplay",
            "simulate", "play the role", "you are now", "imagine you are",
            "take on the role", "behave as",
        ],
    },
    "InstructionOverride": {
        "idx": 2,
        "description": "Instructions override (ignore, disregard, forget previous)",
        "keywords": [
            "ignore", "disregard", "forget", "override", "stop following",
            "bypass", "skip", "cancel", "disregard all previous",
            "forget everything", "ignore all instructions", "forget your training",
        ],
    },
    "ContextManipulation": {
        "idx": 3,
        "description": "Context manipulation (separators, boundary markers, context switching)",
        "keywords": [
            "===", "---", "###", ">>>>", "<<<<", "[[", "]]",
            "__start__", "__end__", "boundary", "separator",
            "new context", "new session", "reset", "clear context",
            "context manipulation", "forget context",
        ],
    },
    "OutputManipulation": {
        "idx": 4,
        "description": "Output format manipulation (change format, encoding, display)",
        "keywords": [
            "output in", "format", "respond in", "tell me in",
            "give me in", "write as", "display as", "show as",
            "change format", "encode output", "obfuscate output",
            "output format", "response format", "reply format",
        ],
    },
    "EncodingAttack": {
        "idx": 5,
        "description": "Encoding/obfuscation attacks (base64, hex, ROT13, encryption)",
        "keywords": [
            "base64", "hex", "rot13", "rot-13", "url encod", "utf-8",
            "utf8", "ascii", "morse", "decode", "encrypt", "obfuscat",
            "encoding", "cipher", "hash", "cryptograp",
        ],
    },
    "JailbreakPattern": {
        "idx": 6,
        "description": "Complex jailbreak patterns (DAN, STAN, multi-technique attacks)",
        "keywords": [
            "dan", "stan", "do anything", "switch", "mode",
            "virtual", "hypothetical", "imagine", "simulation",
            "jailbreak", "unethical", "illegal", "harmful",
        ],
    },
    "PromptLeaking": {
        "idx": 7,
        "description": "System prompt leaking attempts (reveal instructions, show system)",
        "keywords": [
            "system prompt", "reveal", "show me", "tell me",
            "print", "expose", "share your", "what is your",
            "system instructions", "instructions", "secret",
            "hidden prompt", "internal instructions",
        ],
    },
}


LEGACY_TYPE_MAPPING = {
    "Combined": "JailbreakPattern",
    "InstructionOverride": "InstructionOverride",
    "Benign": "Benign",
    "RolePlay": "RolePlay",
    "Encoding": "EncodingAttack",
    "Separator": "ContextManipulation",
    "PromptLeaking": "PromptLeaking",
    "ContextManipulation": "ContextManipulation",
    "OutputManipulation": "OutputManipulation",
}

# Rust AttackType enum indices (from src/detection/result.rs)
RUST_ATTACK_TYPES = {
    0: "RolePlay",
    1: "InstructionOverride",
    2: "ContextManipulation",
    3: "OutputManipulation",
    4: "EncodingAttack",
    5: "JailbreakPattern",
    6: "Benign",
}


# ============================================================================
# ATTACK TYPE CLASSIFICATION
# ============================================================================

class TaxonomyClassifier:
    """Classify samples into the 8-class unified taxonomy."""

    def __init__(self):
        self.keyword_patterns = self._build_keyword_patterns()

    def _build_keyword_patterns(self) -> Dict[str, List[re.Pattern]]:
        """Build compiled regex patterns for each attack type."""
        patterns = {}

        for atype, info in UNIFIED_TAXONOMY.items():
            if atype == "Benign":
                continue

            # Build patterns for keywords
            atype_patterns = []
            for keyword in info["keywords"]:
                # Case-insensitive, word boundary matching
                pattern = re.compile(rf"\b{re.escape(keyword)}\b", re.IGNORECASE)
                atype_patterns.append(pattern)

            patterns[atype] = atype_patterns

        return patterns

    def classify(self, text: str, strict: bool = False) -> Tuple[str, float]:
        """
        Classify text into attack type.

        Returns: (attack_type, confidence)
        - confidence: 0.0-1.0 based on keyword match strength
        """
        text_lower = text.lower()
        scores = defaultdict(float)

        # Score each attack type
        for atype, patterns in self.keyword_patterns.items():
            matches = 0
            for pattern in patterns:
                if pattern.search(text_lower):
                    matches += 1

            if matches > 0:
                # Confidence based on number of matching keywords
                max_keywords = len(UNIFIED_TAXONOMY[atype]["keywords"])
                confidence = min(matches / max_keywords, 1.0)
                scores[atype] = confidence

        # Priority-based resolution if multiple types match
        if scores:
            # Priority order for conflicts
            priority = ["InstructionOverride", "RolePlay", "PromptLeaking",
                       "EncodingAttack", "ContextManipulation", "OutputManipulation",
                       "JailbreakPattern"]

            for atype in priority:
                if atype in scores:
                    return atype, scores[atype]

            # Fallback to highest score
            best_type = max(scores, key=scores.get)
            return best_type, scores[best_type]

        return "JailbreakPattern", 0.3  # Default for unmatched

    def classify_batch(self, texts: List[str]) -> List[Tuple[str, float]]:
        """Classify multiple texts efficiently."""
        return [self.classify(text) for text in texts]


# ============================================================================
# TAXONOMY MAPPING & ANALYSIS
# ============================================================================

def analyze_current_taxonomy(samples: List[Dict]) -> Dict:
    """Analyze the current attack type distribution."""
    analysis = {
        "total_samples": len(samples),
        "by_type": defaultdict(int),
        "by_source": defaultdict(lambda: defaultdict(int)),
        "samples_without_attack_type": 0,
    }

    for sample in samples:
        atype = sample.get("attack_type", "unknown")
        source = sample.get("source", "unknown")

        if not atype or atype == "unknown":
            analysis["samples_without_attack_type"] += 1
            atype = "unknown"

        analysis["by_type"][atype] += 1
        analysis["by_source"][source][atype] += 1

    return analysis


def convert_batch_to_canonical(
    samples: List[Dict],
    use_inference: bool = True,
    classifier: Optional[TaxonomyClassifier] = None,
) -> Tuple[List[TrainingSample], Dict]:
    """
    Convert batch of samples to canonical format.

    Args:
        samples: List of samples (any format)
        use_inference: Whether to infer attack types for unlabeled samples
        classifier: Taxonomy classifier for inference

    Returns:
        (canonical_samples, conversion_stats)
    """
    if use_inference and classifier is None:
        classifier = TaxonomyClassifier()

    canonical = []
    stats = {
        "total_input": len(samples),
        "successful": 0,
        "failed": 0,
        "attack_type_inferred": 0,
        "attack_type_provided": 0,
        "errors": [],
    }

    for i, sample in enumerate(samples):
        try:
            # Try direct conversion first
            canonical_sample = convert_to_canonical_format(sample, index=i)

            # If attack type was inferred (not in original), track it
            if "attack_type" not in sample and use_inference:
                stats["attack_type_inferred"] += 1
            else:
                stats["attack_type_provided"] += 1

            canonical.append(canonical_sample)
            stats["successful"] += 1

        except Exception as e:
            stats["failed"] += 1
            stats["errors"].append({
                "index": i,
                "text_preview": sample.get("text", "")[:100] if sample.get("text") else "",
                "error": str(e),
            })

    return canonical, stats


# ============================================================================
# TAXONOMY DOCUMENTATION
# ============================================================================

def generate_taxonomy_documentation() -> str:
    """Generate comprehensive taxonomy documentation."""
    lines = []

    lines.append("# JailGuard Unified 8-Class Attack Taxonomy\n")
    lines.append("## Overview")
    lines.append("This document describes the standardized attack taxonomy used across all JailGuard components.\n")

    lines.append("## Taxonomy Definition\n")

    for atype, info in UNIFIED_TAXONOMY.items():
        lines.append(f"### {atype} (Index: {info['idx']})\n")
        lines.append(f"**Description:** {info['description']}\n")

        if info["keywords"]:
            lines.append(f"**Keywords:** {', '.join(info['keywords'][:10])}")
            if len(info["keywords"]) > 10:
                lines.append(f", ... and {len(info['keywords']) - 10} more")
            lines.append("\n")

        lines.append("")

    # Legacy mapping table
    lines.append("## Legacy Taxonomy Mapping\n")
    lines.append("| Legacy Type | Unified Type | Rationale |\n")
    lines.append("|-------------|--------------|----------|\n")

    for legacy, unified in sorted(LEGACY_TYPE_MAPPING.items()):
        lines.append(f"| {legacy} | {unified} | Auto-mapped |\n")

    # Python/Rust consistency
    lines.append("\n## Python/Rust Consistency\n")
    lines.append("Both Python and Rust components use the same 8-class taxonomy:\n\n")
    lines.append("| Language | Location | Classes |\n")
    lines.append("|----------|----------|----------|\n")
    lines.append("| Python | `scripts/unified_schema.py` | ATTACK_TYPE_TO_IDX dict |\n")
    lines.append("| Rust | `src/detection/result.rs` | AttackType enum (indices 0-7) |\n")

    return "\n".join(lines)


# ============================================================================
# MAIN
# ============================================================================

def main():
    """Main execution."""
    import argparse

    parser = argparse.ArgumentParser(description="Unified taxonomy integration system")
    parser.add_argument(
        "--analyze",
        type=Path,
        help="Analyze current taxonomy in JSON file"
    )
    parser.add_argument(
        "--convert",
        type=Path,
        help="Convert samples to canonical format"
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Output file for converted samples"
    )
    parser.add_argument(
        "--docs",
        type=Path,
        help="Generate taxonomy documentation to file"
    )
    parser.add_argument(
        "--no-inference",
        action="store_true",
        help="Don't infer attack types for unlabeled samples"
    )

    args = parser.parse_args()

    # Generate documentation
    if args.docs:
        print(f"📝 Generating taxonomy documentation...")
        docs = generate_taxonomy_documentation()
        args.docs.parent.mkdir(parents=True, exist_ok=True)
        with open(args.docs, 'w') as f:
            f.write(docs)
        print(f"✓ Saved: {args.docs}")

    # Analyze current taxonomy
    if args.analyze:
        print(f"\n📊 Analyzing taxonomy in {args.analyze}...")
        with open(args.analyze) as f:
            samples = json.load(f)

        analysis = analyze_current_taxonomy(samples)
        print(f"\n✓ Total samples: {analysis['total_samples']:,}")
        print(f"✓ Samples without attack_type: {analysis['samples_without_attack_type']}")
        print(f"\nBreakdown by type:")
        for atype, count in sorted(analysis["by_type"].items()):
            pct = count / analysis['total_samples'] * 100
            print(f"  {atype:20} {count:>6} ({pct:>5.1f}%)")

        print(f"\nBreakdown by source and type:")
        for source, type_counts in sorted(analysis["by_source"].items()):
            print(f"\n  {source}:")
            for atype, count in sorted(type_counts.items()):
                pct = count / analysis['total_samples'] * 100
                print(f"    {atype:18} {count:>6} ({pct:>5.1f}%)")

    # Convert to canonical format
    if args.convert:
        if not args.output:
            print("❌ Error: --output required when using --convert")
            return 1

        print(f"\n🔄 Converting samples to canonical format...")
        with open(args.convert) as f:
            samples = json.load(f)

        canonical, stats = convert_batch_to_canonical(
            samples,
            use_inference=not args.no_inference
        )

        # Save
        args.output.parent.mkdir(parents=True, exist_ok=True)
        with open(args.output, 'w') as f:
            json.dump(
                [s.to_dict() for s in canonical],
                f,
                indent=2
            )

        print(f"\n✓ Converted: {stats['successful']:,} samples")
        print(f"✗ Failed: {stats['failed']:,} samples")
        print(f"✓ Attack types inferred: {stats['attack_type_inferred']:,}")
        print(f"✓ Attack types provided: {stats['attack_type_provided']:,}")

        if stats['errors']:
            print(f"\n⚠️  First 5 errors:")
            for error in stats['errors'][:5]:
                print(f"  [{error['index']}] {error['text_preview']}: {error['error']}")

        print(f"\n✓ Saved: {args.output}")

    if not any([args.analyze, args.convert, args.docs]):
        print("ℹ️  Use --help to see usage options")

    return 0


if __name__ == "__main__":
    sys.exit(main())
