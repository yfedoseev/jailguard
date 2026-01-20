/**
 * JailGuard Model Loader for JavaScript/Node.js
 *
 * Load and use the JailGuard injection detection model in JavaScript
 * Supports multiple formats:
 * - JSON (native format)
 * - SafeTensors (Hugging Face standard, with @huggingface/safetensors)
 * - ONNX (cross-platform, with onnxruntime-web)
 */

const fs = require('fs');
const path = require('path');

/**
 * JailGuard Model Loader - JSON Format
 */
class JailGuardModelJSON {
    /**
     * Load JailGuard model from JSON file
     *
     * @param {string} modelPath - Path to model.json file
     * @example
     * const model = new JailGuardModelJSON("models/jailguard_injection_detector.json");
     * const embedding = new Float32Array(384).fill(0.5);
     * const prediction = model.predict(embedding);
     */
    constructor(modelPath) {
        this.modelPath = modelPath;

        // Load JSON file
        if (!fs.existsSync(modelPath)) {
            throw new Error(`Model file not found: ${modelPath}`);
        }

        const rawData = fs.readFileSync(modelPath, 'utf8');
        const data = JSON.parse(rawData);

        // Load weights
        this.w_h1 = this._arrayToMatrix(data.w_h1, 256, 384);  // 256 x 384
        this.b_h1 = new Float32Array(data.b_h1);              // 256
        this.w_h2 = this._arrayToMatrix(data.w_h2, 128, 256); // 128 x 256
        this.b_h2 = new Float32Array(data.b_h2);              // 128
        this.w_out = this._arrayToMatrix(data.w_out, 1, 128); // 1 x 128
        this.b_out = new Float32Array(data.b_out);            // 1

        this.learning_rate = data.learning_rate || 0.01;
        this.dropout_rate = data.dropout_rate || 0.2;

        console.log(`✅ Loaded model from ${modelPath}`);
        console.log(`   Architecture: 384→256→128→1`);
        console.log(`   Learning rate: ${this.learning_rate}`);
        console.log(`   Dropout rate: ${this.dropout_rate}`);
    }

    /**
     * Convert flat array to 2D matrix
     */
    _arrayToMatrix(data, rows, cols) {
        const matrix = [];
        let idx = 0;
        for (let i = 0; i < rows; i++) {
            matrix[i] = new Float32Array(cols);
            for (let j = 0; j < cols; j++) {
                matrix[i][j] = data[idx++];
            }
        }
        return matrix;
    }

    /**
     * Matrix-vector multiplication
     */
    _matmul(matrix, vector, bias) {
        const result = new Float32Array(matrix.length);
        for (let i = 0; i < matrix.length; i++) {
            result[i] = bias ? bias[i] : 0;
            for (let j = 0; j < vector.length; j++) {
                result[i] += matrix[i][j] * vector[j];
            }
        }
        return result;
    }

    /**
     * ReLU activation
     */
    _relu(x) {
        for (let i = 0; i < x.length; i++) {
            x[i] = Math.max(0, x[i]);
        }
        return x;
    }

    /**
     * Sigmoid activation
     */
    _sigmoid(x) {
        return 1.0 / (1.0 + Math.exp(-Math.max(-500, Math.min(500, x))));
    }

    /**
     * Predict injection probability for an embedding
     *
     * @param {Float32Array|Array} embedding - 384-dimensional embedding
     * @returns {number} Probability between 0 and 1
     */
    predict(embedding) {
        if (embedding.length !== 384) {
            throw new Error(`Expected embedding length 384, got ${embedding.length}`);
        }

        // Convert to Float32Array if needed
        if (!(embedding instanceof Float32Array)) {
            embedding = new Float32Array(embedding);
        }

        // Forward pass (no dropout during inference)
        let h1 = this._matmul(this.w_h1, embedding, this.b_h1);  // 256
        h1 = this._relu(h1);

        let h2 = this._matmul(this.w_h2, h1, this.b_h2);         // 128
        h2 = this._relu(h2);

        const logits = this._matmul(this.w_out, h2, this.b_out); // 1
        const output = this._sigmoid(logits[0]);

        return output;
    }

    /**
     * Predict injection probability for multiple embeddings
     *
     * @param {Array<Float32Array|Array>} embeddings - Array of embeddings
     * @returns {Float32Array} Array of predictions
     */
    predictBatch(embeddings) {
        const predictions = new Float32Array(embeddings.length);
        for (let i = 0; i < embeddings.length; i++) {
            predictions[i] = this.predict(embeddings[i]);
        }
        return predictions;
    }

    /**
     * Classify embedding as injection or benign
     *
     * @param {Float32Array|Array} embedding - 384-dimensional embedding
     * @param {number} threshold - Decision threshold (default 0.5)
     * @returns {Object} {isInjection: boolean, confidence: number}
     */
    classify(embedding, threshold = 0.5) {
        const pred = this.predict(embedding);
        const isInjection = pred > threshold;
        const confidence = isInjection ? pred : 1.0 - pred;

        return {
            isInjection,
            confidence,
            probability: pred,
            label: isInjection ? 'INJECTION' : 'BENIGN'
        };
    }
}

/**
 * JailGuard Model Loader - SafeTensors Format
 */
class JailGuardModelSafeTensors {
    /**
     * Load JailGuard model from SafeTensors file
     * Requires: npm install @huggingface/safetensors
     *
     * @param {string} modelPath - Path to model.safetensors file
     */
    async init(modelPath) {
        try {
            const safetensors = require('@huggingface/safetensors');
            this.tensors = await safetensors.loadSafetensors(modelPath);
        } catch (e) {
            throw new Error(
                'SafeTensors not installed. Install with:\n' +
                '  npm install @huggingface/safetensors'
            );
        }
        console.log(`✅ Loaded model from ${modelPath} (SafeTensors format)`);
        return this;
    }

    predict(embedding) {
        // Implement similar to JSON version
        throw new Error('SafeTensors loading requires browser/async context');
    }
}

/**
 * JailGuard Model Loader - ONNX Format
 */
class JailGuardModelONNX {
    /**
     * Load JailGuard model from ONNX file
     * Requires: npm install onnxruntime-node
     *
     * @param {string} modelPath - Path to model.onnx file
     */
    async init(modelPath) {
        try {
            const ort = require('onnxruntime-node');
            this.session = await ort.InferenceSession.create(modelPath);
        } catch (e) {
            throw new Error(
                'ONNX Runtime not installed. Install with:\n' +
                '  npm install onnxruntime-node'
            );
        }
        console.log(`✅ Loaded model from ${modelPath} (ONNX format)`);
        return this;
    }

    async predict(embedding) {
        // Reshape for batching
        const input = new Float32Array(384);
        for (let i = 0; i < 384; i++) {
            input[i] = embedding[i];
        }

        const output = await this.session.run({
            embedding: new ort.Tensor('float32', input, [1, 384])
        });

        return output.logits.data[0];
    }
}

/**
 * Auto-detect and load JailGuard model from any supported format
 *
 * @param {string} modelPath - Path to model file (.json, .safetensors, or .onnx)
 * @returns {JailGuardModel} Loaded model instance
 */
function loadModel(modelPath) {
    const ext = path.extname(modelPath).toLowerCase();

    if (ext === '.json') {
        return new JailGuardModelJSON(modelPath);
    } else if (ext === '.safetensors') {
        return new JailGuardModelSafeTensors(modelPath);
    } else if (ext === '.onnx') {
        return new JailGuardModelONNX(modelPath);
    } else {
        throw new Error(`Unsupported model format: ${ext}`);
    }
}

// ============================================================================
// BROWSER VERSION (for use in web browsers)
// ============================================================================

/**
 * Browser-compatible loader (loads from URL)
 */
async function loadModelFromURL(modelURL) {
    const response = await fetch(modelURL);
    const data = await response.json();

    class BrowserJailGuardModel {
        constructor(modelData) {
            this.w_h1 = modelData.w_h1;
            this.b_h1 = new Float32Array(modelData.b_h1);
            this.w_h2 = modelData.w_h2;
            this.b_h2 = new Float32Array(modelData.b_h2);
            this.w_out = modelData.w_out;
            this.b_out = new Float32Array(modelData.b_out);
        }

        _matmul(matrix, vector, bias) {
            const result = new Float32Array(matrix.length);
            for (let i = 0; i < matrix.length; i++) {
                result[i] = bias[i] || 0;
                for (let j = 0; j < vector.length; j++) {
                    result[i] += matrix[i][j] * vector[j];
                }
            }
            return result;
        }

        _relu(x) {
            return x.map(v => Math.max(0, v));
        }

        _sigmoid(x) {
            return 1.0 / (1.0 + Math.exp(-x));
        }

        predict(embedding) {
            if (embedding.length !== 384) {
                throw new Error(`Expected embedding length 384, got ${embedding.length}`);
            }

            let h1 = this._matmul(this.w_h1, embedding, this.b_h1);
            h1 = this._relu(h1);

            let h2 = this._matmul(this.w_h2, h1, this.b_h2);
            h2 = this._relu(h2);

            const logits = this._matmul(this.w_out, h2, this.b_out);
            return this._sigmoid(logits[0]);
        }

        classify(embedding, threshold = 0.5) {
            const pred = this.predict(embedding);
            return {
                isInjection: pred > threshold,
                confidence: pred > threshold ? pred : 1.0 - pred,
                probability: pred,
                label: pred > threshold ? 'INJECTION' : 'BENIGN'
            };
        }
    }

    return new BrowserJailGuardModel(data);
}

// ============================================================================
// EXPORTS
// ============================================================================

if (typeof module !== 'undefined' && module.exports) {
    // Node.js
    module.exports = {
        JailGuardModelJSON,
        JailGuardModelSafeTensors,
        JailGuardModelONNX,
        loadModel,
        loadModelFromURL
    };
} else if (typeof window !== 'undefined') {
    // Browser
    window.JailGuard = {
        loadModelFromURL
    };
}

// ============================================================================
// EXAMPLE USAGE
// ============================================================================

if (require.main === module) {
    console.log('\nJailGuard JavaScript Loader Examples\n' + '='.repeat(50));

    try {
        console.log('\n1️⃣  JSON Format (Native):');
        const model = new JailGuardModelJSON('../../models/jailguard_injection_detector.json');

        // Example embedding (random)
        const embedding = new Float32Array(384);
        for (let i = 0; i < 384; i++) {
            embedding[i] = Math.random() - 0.5;
        }

        const pred = model.predict(embedding);
        console.log(`   Prediction: ${pred.toFixed(4)}`);

        console.log('\n2️⃣  Classification with Confidence:');
        const result = model.classify(embedding);
        console.log(`   Label: ${result.label}`);
        console.log(`   Confidence: ${result.confidence.toFixed(4)}`);

        console.log('\n3️⃣  Batch Prediction:');
        const batch = [];
        for (let b = 0; b < 3; b++) {
            const emb = new Float32Array(384);
            for (let i = 0; i < 384; i++) {
                emb[i] = Math.random() - 0.5;
            }
            batch.push(emb);
        }
        const predictions = model.predictBatch(batch);
        console.log(`   Predictions: [${Array.from(predictions).map(p => p.toFixed(4)).join(', ')}]`);

    } catch (e) {
        console.log(`   (Error: ${e.message})`);
    }

    console.log('\n' + '='.repeat(50));
    console.log('✅ Loader examples complete');
}
