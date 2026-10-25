# Distance and Similarity Metrics

This document defines the mathematical formulation, range, properties, and usage recommendations for vector distance metrics supported in the index.

---

## 1. Squared Euclidean Distance (L2)

### Formula
$$D_{L2}(a, b) = \sum_{i=0}^{d-1} (a_i - b_i)^2$$

### Range
$$[0, +\infty)$$

### Mathematical Properties
- **Non-negativity**: $D_{L2}(a, b) \ge 0$ for all $a, b \in \mathbb{R}^d$.
- **Identity of indiscernibles**: $D_{L2}(a, a) = 0$, and $D_{L2}(a, b) = 0 \iff a = b$.
- **Symmetry**: $D_{L2}(a, b) = D_{L2}(b, a)$.
- **Triangle Inequality**: While the square root metric ($\|a - b\|_2$) satisfies the triangle inequality, the squared distance preserves relative ranking while avoiding computationally expensive square roots.

### When to Use
- When magnitude/scale of vector components carries significant information (e.g., spatial coordinates, raw feature vectors).
- Standard image classification features where embedding vectors are not unit-normalized.

---

## 2. Cosine Distance

### Formula
$$\text{Cosine Similarity}(a, b) = \frac{\sum_{i=0}^{d-1} a_i b_i}{\sqrt{\sum_{i=0}^{d-1} a_i^2} \sqrt{\sum_{i=0}^{d-1} b_i^2}}$$

$$D_{\text{Cosine}}(a, b) = 1.0 - \text{Cosine Similarity}(a, b)$$

### Range
$$[0, 2.0]$$
- $D_{\text{Cosine}} = 0.0$: Perfectly aligned vectors (angle $\theta = 0$).
- $D_{\text{Cosine}} = 1.0$: Orthogonal vectors ($\theta = \pi / 2$).
- $D_{\text{Cosine}} = 2.0$: Directly opposing vectors ($\theta = \pi$).

### Mathematical Properties
- **Non-negativity**: $D_{\text{Cosine}}(a, b) \ge 0$.
- **Identity of indiscernibles**: $D_{\text{Cosine}}(a, a) = 0$ for any non-zero vector $a$.
- **Symmetry**: $D_{\text{Cosine}}(a, b) = D_{\text{Cosine}}(b, a)$.
- **Undefined at zero**: Undefined if either $a$ or $b$ is the all-zero vector $\|a\| = 0$. In this case, an explicit error `MetricError::ZeroVector` is returned.

### When to Use
- When the direction of the vector represents semantics and the magnitude is arbitrary (e.g. text embeddings from BERT, OpenAI text-embedding-ada-002, Sentence-Transformers).
- Normalized document frequencies or semantic embeddings where length varies due to text token count.

---

## 3. Inner Product Distance

### Formula
$$D_{\text{IP}}(a, b) = -\sum_{i=0}^{d-1} a_i b_i = -\langle a, b \rangle$$

### Range
$$(-\infty, +\infty)$$
*(Can be negative when the dot product is positive).*

### Mathematical Properties
- **Symmetry**: $D_{\text{IP}}(a, b) = D_{\text{IP}}(b, a)$.
- **No non-negativity**: Distance is negative whenever $\langle a, b \rangle > 0$.
- **No identity of indiscernibles**: $D_{\text{IP}}(a, a) = -\|a\|^2 \neq 0$ unless $a = 0$.
- **Preserves maximum inner product search (MIPS)**: Minimizing $D_{\text{IP}}$ directly maximizes the inner product $\langle a, b \rangle$.

### When to Use
- Recommendation systems and matrix factorization models (e.g., user-item dot products, learned dual-encoder retrieval).
- Unit-normalized vectors where dot product is algebraically equivalent to cosine similarity ($D_{\text{IP}} = D_{\text{Cosine}} - 1$).
