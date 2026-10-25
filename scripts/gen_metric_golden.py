#!/usr/bin/env python3
import json
import os
import numpy as np

def generate_golden():
    np.random.seed(42)
    pairs = []
    
    # 20 vector pairs across different dimensions
    dims = [4, 4, 8, 8, 16, 16, 32, 32, 64, 64, 128, 128, 4, 8, 16, 32, 64, 128, 5, 10]
    
    for i, d in enumerate(dims):
        a = np.random.uniform(-1.0, 1.0, size=d).astype(np.float32)
        b = np.random.uniform(-1.0, 1.0, size=d).astype(np.float32)
        
        # Normalize to unit length (standard embedding form)
        norm_a_init = np.linalg.norm(a)
        norm_b_init = np.linalg.norm(b)
        if norm_a_init == 0:
            a[0] = 1.0
            norm_a_init = 1.0
        if norm_b_init == 0:
            b[0] = 1.0
            norm_b_init = 1.0
            
        a = (a / norm_a_init).astype(np.float32)
        b = (b / norm_b_init).astype(np.float32)
            
        # L2 (squared Euclidean): sum((a - b)^2)
        l2 = float(np.sum((a - b) ** 2))
        
        # Cosine distance: 1 - dot(a, b) / (norm(a) * norm(b))
        norm_a = float(np.linalg.norm(a))
        norm_b = float(np.linalg.norm(b))
        sim = float(np.dot(a, b) / (norm_a * norm_b))
        cosine = float(1.0 - sim)
        
        # Inner Product distance: negative dot product
        ip = float(-np.dot(a, b))
        
        pairs.append({
            "id": i + 1,
            "dimension": d,
            "a": [float(x) for x in a],
            "b": [float(x) for x in b],
            "expected": {
                "l2": l2,
                "cosine": cosine,
                "inner_product": ip
            }
        })
        
    out_dir = os.path.join(os.path.dirname(__file__), "..", "backend", "crates", "simd", "tests", "golden")
    os.makedirs(out_dir, exist_ok=True)
    out_path = os.path.join(out_dir, "metrics.json")
    
    with open(out_path, "w") as f:
        json.dump(pairs, f, indent=2)
    print(f"Wrote {len(pairs)} golden metric test cases to {out_path}")

if __name__ == "__main__":
    generate_golden()
