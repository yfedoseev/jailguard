"""
JailGuard Model Loader for Python

This module provides utilities to load and use the JailGuard injection detection model
in Python. Supports multiple formats:
- JSON (native format)
- SafeTensors (Hugging Face standard)
- ONNX (cross-platform)
"""

import json
import numpy as np
from pathlib import Path
from typing import Union, Tuple, Dict, Any
import warnings


class JailGuardModelJSON:
    """Load and use JailGuard model from JSON format"""

    def __init__(self, model_path: Union[str, Path]):
        """
        Load JailGuard model from JSON file

        Args:
            model_path: Path to model.json file

        Example:
            >>> model = JailGuardModelJSON("models/jailguard_injection_detector.json")
            >>> embedding = np.random.randn(384)
            >>> prediction = model.predict(embedding)
        """
        self.model_path = Path(model_path)

        if not self.model_path.exists():
            raise FileNotFoundError(f"Model file not found: {model_path}")

        with open(self.model_path, 'r') as f:
            data = json.load(f)

        # Load weights
        self.w_h1 = np.array(data['w_h1'], dtype=np.float32)  # 256 x 384
        self.b_h1 = np.array(data['b_h1'], dtype=np.float32)  # 256
        self.w_h2 = np.array(data['w_h2'], dtype=np.float32)  # 128 x 256
        self.b_h2 = np.array(data['b_h2'], dtype=np.float32)  # 128
        self.w_out = np.array(data['w_out'], dtype=np.float32)  # 1 x 128
        self.b_out = np.array(data['b_out'], dtype=np.float32)  # 1

        self.learning_rate = data.get('learning_rate', 0.01)
        self.dropout_rate = data.get('dropout_rate', 0.2)

        print(f"✅ Loaded model from {model_path}")
        print(f"   Architecture: 384→256→128→1")
        print(f"   Learning rate: {self.learning_rate}")
        print(f"   Dropout rate: {self.dropout_rate}")

    def _relu(self, x: np.ndarray) -> np.ndarray:
        """ReLU activation"""
        return np.maximum(0.0, x)

    def _sigmoid(self, x: np.ndarray) -> np.ndarray:
        """Sigmoid activation"""
        return 1.0 / (1.0 + np.exp(-np.clip(x, -500, 500)))

    def predict(self, embedding: np.ndarray) -> float:
        """
        Predict injection probability for an embedding

        Args:
            embedding: 384-dimensional embedding vector

        Returns:
            Float between 0 and 1 (injection probability)
        """
        if embedding.shape != (384,):
            raise ValueError(f"Expected embedding shape (384,), got {embedding.shape}")

        # Forward pass (no dropout during inference)
        h1 = self._relu(np.dot(self.w_h1, embedding) + self.b_h1)  # 256
        h2 = self._relu(np.dot(self.w_h2, h1) + self.b_h2)  # 128
        logits = np.dot(self.w_out, h2) + self.b_out  # 1
        output = self._sigmoid(logits[0])

        return float(output)

    def predict_batch(self, embeddings: np.ndarray) -> np.ndarray:
        """
        Predict injection probability for multiple embeddings

        Args:
            embeddings: Array of shape (N, 384)

        Returns:
            Array of shape (N,) with probabilities
        """
        if embeddings.shape[1] != 384:
            raise ValueError(f"Expected embeddings shape (N, 384), got {embeddings.shape}")

        predictions = np.zeros(embeddings.shape[0])
        for i, emb in enumerate(embeddings):
            predictions[i] = self.predict(emb)

        return predictions

    def classify(self, embedding: np.ndarray, threshold: float = 0.5) -> Tuple[bool, float]:
        """
        Classify embedding as injection or benign

        Args:
            embedding: 384-dimensional embedding
            threshold: Decision threshold (default 0.5)

        Returns:
            Tuple of (is_injection, confidence)
        """
        pred = self.predict(embedding)
        is_injection = pred > threshold
        confidence = pred if is_injection else 1.0 - pred

        return is_injection, confidence


class JailGuardModelSafeTensors:
    """Load and use JailGuard model from SafeTensors format"""

    def __init__(self, model_path: Union[str, Path]):
        """
        Load JailGuard model from SafeTensors file

        Args:
            model_path: Path to model.safetensors file

        Example:
            >>> try:
            ...     from safetensors.numpy import load_file
            ...     model = JailGuardModelSafeTensors("models/jailguard.safetensors")
            ... except ImportError:
            ...     print("Install safetensors: pip install safetensors")
        """
        try:
            from safetensors.numpy import load_file
        except ImportError:
            raise ImportError(
                "safetensors not installed. Install with:\n"
                "  pip install safetensors"
            )

        self.model_path = Path(model_path)

        if not self.model_path.exists():
            raise FileNotFoundError(f"Model file not found: {model_path}")

        # Load tensors
        tensors = load_file(self.model_path)

        self.w_h1 = tensors['w_h1'].reshape(256, 384).astype(np.float32)
        self.b_h1 = tensors['b_h1'].astype(np.float32)
        self.w_h2 = tensors['w_h2'].reshape(128, 256).astype(np.float32)
        self.b_h2 = tensors['b_h2'].astype(np.float32)
        self.w_out = tensors['w_out'].reshape(1, 128).astype(np.float32)
        self.b_out = tensors['b_out'].astype(np.float32)

        print(f"✅ Loaded model from {model_path} (SafeTensors format)")
        print(f"   Architecture: 384→256→128→1")

    def _relu(self, x: np.ndarray) -> np.ndarray:
        """ReLU activation"""
        return np.maximum(0.0, x)

    def _sigmoid(self, x: np.ndarray) -> np.ndarray:
        """Sigmoid activation"""
        return 1.0 / (1.0 + np.exp(-np.clip(x, -500, 500)))

    def predict(self, embedding: np.ndarray) -> float:
        """Predict injection probability"""
        if embedding.shape != (384,):
            raise ValueError(f"Expected embedding shape (384,), got {embedding.shape}")

        h1 = self._relu(np.dot(self.w_h1, embedding) + self.b_h1)
        h2 = self._relu(np.dot(self.w_h2, h1) + self.b_h2)
        logits = np.dot(self.w_out, h2) + self.b_out
        output = self._sigmoid(logits[0])

        return float(output)


class JailGuardModelONNX:
    """Load and use JailGuard model from ONNX format"""

    def __init__(self, model_path: Union[str, Path]):
        """
        Load JailGuard model from ONNX file

        Args:
            model_path: Path to model.onnx file

        Example:
            >>> try:
            ...     import onnxruntime as rt
            ...     model = JailGuardModelONNX("models/jailguard.onnx")
            ... except ImportError:
            ...     print("Install onnxruntime: pip install onnxruntime")
        """
        try:
            import onnxruntime as rt
        except ImportError:
            raise ImportError(
                "onnxruntime not installed. Install with:\n"
                "  pip install onnxruntime"
            )

        self.model_path = Path(model_path)

        if not self.model_path.exists():
            raise FileNotFoundError(f"Model file not found: {model_path}")

        self.session = rt.InferenceSession(str(model_path), providers=['CPUExecutionProvider'])
        self.input_name = self.session.get_inputs()[0].name
        self.output_name = self.session.get_outputs()[0].name

        print(f"✅ Loaded model from {model_path} (ONNX format)")
        print(f"   Architecture: 384→256→128→1")

    def predict(self, embedding: np.ndarray) -> float:
        """Predict injection probability"""
        if embedding.shape != (384,):
            raise ValueError(f"Expected embedding shape (384,), got {embedding.shape}")

        # ONNX expects batched input
        embedding_batched = embedding.reshape(1, 384).astype(np.float32)
        outputs = self.session.run([self.output_name], {self.input_name: embedding_batched})

        return float(outputs[0][0][0])


def load_model(model_path: Union[str, Path]) -> Union[JailGuardModelJSON, JailGuardModelSafeTensors, JailGuardModelONNX]:
    """
    Auto-detect and load JailGuard model from any supported format

    Supported formats:
    - .json: Native Rust JSON export
    - .safetensors: Hugging Face standard format
    - .onnx: Cross-platform format

    Example:
        >>> model = load_model("models/jailguard_injection_detector.json")
        >>> prediction = model.predict(embedding)
    """
    model_path = Path(model_path)
    suffix = model_path.suffix.lower()

    if suffix == '.json':
        return JailGuardModelJSON(model_path)
    elif suffix == '.safetensors':
        return JailGuardModelSafeTensors(model_path)
    elif suffix == '.onnx':
        return JailGuardModelONNX(model_path)
    else:
        raise ValueError(f"Unsupported model format: {suffix}")


# Example usage
if __name__ == "__main__":
    print("\nJailGuard Python Loader Examples\n" + "="*50)

    # Example 1: JSON format
    print("\n1️⃣  JSON Format (Native):")
    try:
        model_json = JailGuardModelJSON("../../models/jailguard_injection_detector.json")
        test_embedding = np.random.randn(384).astype(np.float32)
        pred = model_json.predict(test_embedding)
        print(f"   Prediction: {pred:.4f}")
    except FileNotFoundError:
        print("   (Model file not found - train first)")

    # Example 2: Batch prediction
    print("\n2️⃣  Batch Prediction:")
    try:
        batch = np.random.randn(5, 384).astype(np.float32)
        preds = model_json.predict_batch(batch)
        print(f"   Batch predictions: {preds}")
    except NameError:
        print("   (Model not loaded)")

    # Example 3: Classification with confidence
    print("\n3️⃣  Classification with Confidence:")
    try:
        test_embedding = np.random.randn(384).astype(np.float32)
        is_injection, confidence = model_json.classify(test_embedding)
        label = "INJECTION" if is_injection else "BENIGN"
        print(f"   Classification: {label} (confidence: {confidence:.4f})")
    except NameError:
        print("   (Model not loaded)")

    print("\n" + "="*50)
    print("✅ Loader examples complete")
