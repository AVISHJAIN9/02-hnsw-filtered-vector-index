import os, time, json, urllib.request
import numpy as np, h5py, hnswlib
import matplotlib; matplotlib.use("Agg")
import matplotlib.pyplot as plt

DS, K = "glove-25-angular", 10
N = int(os.environ.get("N", 200_000))      # subsample size for speed
NQ = 500
here = os.path.dirname(__file__)
path = os.path.join(here, "data", f"{DS}.hdf5")
if not os.path.exists(path):
    urllib.request.urlretrieve(f"https://ann-benchmarks.com/{DS}.hdf5", path)

f = h5py.File(path, "r")
train = np.array(f["train"][:N], dtype="float32")
test = np.array(f["test"][:NQ], dtype="float32")
norm = lambda a: a / np.maximum(np.linalg.norm(a, axis=1, keepdims=True), 1e-12)
train_n, test_n = norm(train), norm(test)
attr = np.random.RandomState(0).randint(0, 100, len(train))  # attr < s  => s% selectivity

def exact_topk(allowed):                      # exact baseline (brute force)
    sub = train_n[allowed]
    out = np.empty((len(test_n), K), dtype=np.int64)
    for s in range(0, len(test_n), 50):
        sims = test_n[s:s+50] @ sub.T
        top = np.argpartition(-sims, K - 1, axis=1)[:, :K]
        out[s:s+50] = allowed[top]
    return out

p = hnswlib.Index(space="cosine", dim=train.shape[1])
p.init_index(max_elements=len(train), ef_construction=200, M=16)
t0 = time.perf_counter(); p.add_items(train); build_s = time.perf_counter() - t0
p.set_num_threads(1)
print(f"built {len(train)} vectors in {build_s:.1f}s")

truth_all = exact_topk(np.arange(len(train)))
sweep = []
for ef in [10, 20, 40, 80, 160, 320]:
    p.set_ef(ef)
    t0 = time.perf_counter(); labels, _ = p.knn_query(test, k=K); dt = time.perf_counter() - t0
    rec = np.mean([len(set(labels[i]) & set(truth_all[i])) / K for i in range(len(test))])
    sweep.append((ef, rec, len(test) / dt)); print(f"ef={ef:4d} recall@{K}={rec:.3f} qps={len(test)/dt:,.0f}")

filtered = []
p.set_ef(100)
for sel in [1, 2, 5, 10, 25, 50, 100]:
    mask = attr < sel
    truth = exact_topk(np.where(mask)[0])
    hits = fails = 0
    for i in range(len(test)):
        try:
            lab, _ = p.knn_query(test[i:i+1], k=K, filter=lambda idx: bool(mask[idx]))
            got = set(lab[0].tolist())
        except RuntimeError:
            got, fails = set(), fails + 1
        hits += len(got & set(truth[i].tolist()))
    rec = hits / (len(test) * K)
    filtered.append((sel, rec, fails)); print(f"selectivity={sel:3d}% recall@{K}={rec:.3f} failed_queries={fails}")

json.dump({"dataset": DS, "n": len(train), "k": K, "unfiltered": sweep, "filtered": filtered},
          open(os.path.join(here, "results.json"), "w"), indent=1)
fig, ax = plt.subplots(1, 2, figsize=(11, 4))
ax[0].plot([s[2] for s in sweep], [s[1] for s in sweep], "o-"); ax[0].set(xscale="log", xlabel="queries/sec (1 thread)", ylabel=f"recall@{K}", title="hnswlib baseline: recall vs speed")
ax[1].plot([s[0] for s in filtered], [s[1] for s in filtered], "o-"); ax[1].set(xlabel="filter selectivity (% of data passing)", ylabel=f"recall@{K}", title="hnswlib built-in filter, ef=100")
plt.tight_layout(); plt.savefig(os.path.join(here, "hnswlib_baseline.png"), dpi=140)
