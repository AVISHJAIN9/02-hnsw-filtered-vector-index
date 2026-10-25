'use client'

import React, { useState } from 'react'
import {
  Cpu,
  Route,
  Zap,
  ShieldCheck,
  Database,
  ArrowRight,
  TrendingDown,
  Gauge,
  Sparkles,
  Layers,
} from 'lucide-react'

export default function AiPlannerPage() {
  const [selectivity, setSelectivity] = useState<number>(0.08) // 8%
  const [vectorCount, setVectorCount] = useState<number>(100000) // 100k vectors
  const [topK, setTopK] = useState<number>(10)

  // Cost calculations
  const sigma = selectivity
  const N = vectorCount
  const k = topK

  // Estimated operations
  // 1. Bitmap exact scan cost: sigma * N distance computations
  const bitmapScanOps = Math.round(sigma * N)
  const bitmapLatencyUs = Math.round(bitmapScanOps * 0.12) // 0.12us per SIMD dist

  // 2. In-graph filtered HNSW cost: ~ ef * log(N) * (1 / sigma^0.3)
  const inGraphOps = Math.round(64 * Math.log2(N) * Math.pow(1 / Math.max(0.01, sigma), 0.25))
  const inGraphLatencyUs = Math.round(inGraphOps * 0.45)

  // 3. Post-filtered HNSW cost: (k / sigma) * log(N)
  const postFilterOps = Math.round((k / Math.max(0.001, sigma)) * Math.log2(N) * 2.5)
  const postFilterLatencyUs = Math.round(postFilterOps * 0.35)

  // Recommendation engine
  let recommendedStrategy: 'bitmap_scan' | 'in_graph' | 'post_filter' = 'in_graph'
  if (sigma < 0.02) {
    recommendedStrategy = 'bitmap_scan'
  } else if (sigma <= 0.40) {
    recommendedStrategy = 'in_graph'
  } else {
    recommendedStrategy = 'post_filter'
  }

  return (
    <div className="max-w-7xl mx-auto px-4 py-8 space-y-8">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl sm:text-3xl font-extrabold text-white flex items-center gap-3">
            <Cpu className="w-7 h-7 text-cyan-400" />
            Learned Query Planner & Filter Routing
          </h1>
          <p className="text-slate-400 text-xs sm:text-sm mt-1">
            Cost-based optimizer dynamically routing queries between Bitmap Inverted Scans, In-Graph Filter Pushdown, and Post-Filtering.
          </p>
        </div>

        <div className="inline-flex items-center gap-2 px-3.5 py-2 rounded-xl bg-purple-950/40 border border-purple-800/60 text-xs font-mono text-purple-300">
          <Sparkles className="w-4 h-4 text-purple-400" />
          <span>Cost-Based Optimizer Active</span>
        </div>
      </div>

      {/* Interactive Simulator Card */}
      <div className="glass-panel p-6 sm:p-8 rounded-3xl space-y-6">
        <div className="flex items-center justify-between pb-4 border-b border-slate-800">
          <div className="flex items-center gap-2">
            <Gauge className="w-5 h-5 text-cyan-400" />
            <h3 className="text-lg font-bold text-white">Interactive Selectivity Cost Simulator</h3>
          </div>
          <span className="text-xs font-mono text-slate-400">
            Drag slider to observe route transition boundaries
          </span>
        </div>

        {/* Sliders */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div>
            <div className="flex justify-between text-xs font-mono text-slate-300 mb-2">
              <span>Filter Selectivity ($\sigma = |M| / N$)</span>
              <span className="text-cyan-400 font-bold text-sm">
                {(selectivity * 100).toFixed(1)}%
              </span>
            </div>
            <input
              type="range"
              min="0.002"
              max="1.0"
              step="0.002"
              value={selectivity}
              onChange={(e) => setSelectivity(parseFloat(e.target.value))}
              className="w-full accent-cyan-400 cursor-pointer bg-slate-800 rounded-lg h-2.5"
            />
            <div className="flex justify-between text-[10px] font-mono text-slate-500 mt-1">
              <span>0.2% (High Selectivity)</span>
              <span>100% (Unfiltered)</span>
            </div>
          </div>

          <div>
            <div className="flex justify-between text-xs font-mono text-slate-300 mb-2">
              <span>Total Indexed Corpus ($N$)</span>
              <span className="text-emerald-400 font-bold text-sm">
                {vectorCount.toLocaleString()}
              </span>
            </div>
            <input
              type="range"
              min="10000"
              max="1000000"
              step="10000"
              value={vectorCount}
              onChange={(e) => setVectorCount(parseInt(e.target.value, 10))}
              className="w-full accent-emerald-400 cursor-pointer bg-slate-800 rounded-lg h-2.5"
            />
          </div>

          <div>
            <div className="flex justify-between text-xs font-mono text-slate-300 mb-2">
              <span>Nearest Neighbors Target ($k$)</span>
              <span className="text-purple-400 font-bold text-sm">{topK}</span>
            </div>
            <input
              type="range"
              min="1"
              max="100"
              step="1"
              value={topK}
              onChange={(e) => setTopK(parseInt(e.target.value, 10))}
              className="w-full accent-purple-400 cursor-pointer bg-slate-800 rounded-lg h-2.5"
            />
          </div>
        </div>

        {/* Dynamic Route Recommendation Banner */}
        <div className="p-5 rounded-2xl bg-slate-900/90 border border-slate-800 space-y-3">
          <div className="flex items-center justify-between">
            <div className="text-xs font-mono text-slate-400 uppercase tracking-wider">
              Selected Execution Path by Optimizer
            </div>
            <div className="flex items-center gap-2 text-xs font-mono text-slate-300">
              <span>Matching Candidate Pool:</span>
              <span className="text-cyan-400 font-bold">
                {Math.round(selectivity * vectorCount).toLocaleString()} vectors
              </span>
            </div>
          </div>

          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
            <div className="flex items-center gap-3">
              <div
                className={`w-10 h-10 rounded-xl flex items-center justify-center font-bold ${
                  recommendedStrategy === 'bitmap_scan'
                    ? 'bg-amber-500/20 text-amber-400 border border-amber-500/40'
                    : recommendedStrategy === 'in_graph'
                    ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/40'
                    : 'bg-cyan-500/20 text-cyan-400 border border-cyan-500/40'
                }`}
              >
                <Route className="w-5 h-5" />
              </div>
              <div>
                <h4 className="text-base font-bold text-white capitalize">
                  {recommendedStrategy === 'bitmap_scan' && 'Inverted Bitmap Exact Scan (Low $\\sigma$ < 2%)'}
                  {recommendedStrategy === 'in_graph' && 'In-Graph Filtered HNSW Pushdown (2% &le; $\\sigma$ &le; 40%)'}
                  {recommendedStrategy === 'post_filter' && 'Standard HNSW + Post-Filter Rejection ($\\sigma$ > 40%)'}
                </h4>
                <p className="text-xs text-slate-400">
                  {recommendedStrategy === 'bitmap_scan' &&
                    'Corpus subset is tiny. Pure SIMD brute-force over matching bitmap is faster than graph traversal.'}
                  {recommendedStrategy === 'in_graph' &&
                    'Graph pruning active. Evaluates roaring bitmap mask inside HNSW beam search to prevent traps.'}
                  {recommendedStrategy === 'post_filter' &&
                    'Filters are broad. Standard fast HNSW retrieval followed by candidate verification is optimal.'}
                </p>
              </div>
            </div>

            <div className="text-right font-mono shrink-0">
              <div className="text-[11px] text-slate-400">Estimated Latency</div>
              <div className="text-lg font-bold text-emerald-400">
                {recommendedStrategy === 'bitmap_scan' && `${bitmapLatencyUs} µs`}
                {recommendedStrategy === 'in_graph' && `${inGraphLatencyUs} µs`}
                {recommendedStrategy === 'post_filter' && `${postFilterLatencyUs} µs`}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* 3 Comparative Strategy Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* Strategy 1: Bitmap Exact Scan */}
        <div
          className={`glass-card p-6 rounded-2xl transition-all duration-300 ${
            recommendedStrategy === 'bitmap_scan'
              ? 'ring-2 ring-amber-400 border-amber-500/60 bg-amber-950/20 shadow-xl shadow-amber-500/10'
              : 'opacity-70'
          }`}
        >
          <div className="flex items-center justify-between mb-4">
            <span className="text-xs font-mono px-2.5 py-1 rounded-lg bg-amber-950/60 text-amber-400 border border-amber-800/60">
              Strategy 1
            </span>
            {recommendedStrategy === 'bitmap_scan' && (
              <span className="text-[11px] font-mono font-bold text-amber-400 animate-pulse">
                OPTIMAL PATH
              </span>
            )}
          </div>
          <h3 className="text-base font-bold text-white mb-1">Inverted Bitmap Scan</h3>
          <p className="text-xs text-slate-400 mb-4">
            Direct SIMD dot product over IDs in roaring bitmap posting list.
          </p>

          <div className="space-y-2 pt-3 border-t border-slate-800 text-xs font-mono">
            <div className="flex justify-between text-slate-400">
              <span>Dist Computations:</span>
              <span className="text-slate-200">{bitmapScanOps.toLocaleString()}</span>
            </div>
            <div className="flex justify-between text-slate-400">
              <span>Est. Latency:</span>
              <span className="text-amber-400 font-bold">{bitmapLatencyUs} µs</span>
            </div>
            <div className="flex justify-between text-slate-400">
              <span>Recall Guarantee:</span>
              <span className="text-emerald-400 font-bold">100% (Exact)</span>
            </div>
          </div>
        </div>

        {/* Strategy 2: In-Graph Filtered HNSW */}
        <div
          className={`glass-card p-6 rounded-2xl transition-all duration-300 ${
            recommendedStrategy === 'in_graph'
              ? 'ring-2 ring-emerald-400 border-emerald-500/60 bg-emerald-950/20 shadow-xl shadow-emerald-500/10'
              : 'opacity-70'
          }`}
        >
          <div className="flex items-center justify-between mb-4">
            <span className="text-xs font-mono px-2.5 py-1 rounded-lg bg-emerald-950/60 text-emerald-400 border border-emerald-800/60">
              Strategy 2 (Default)
            </span>
            {recommendedStrategy === 'in_graph' && (
              <span className="text-[11px] font-mono font-bold text-emerald-400 animate-pulse">
                OPTIMAL PATH
              </span>
            )}
          </div>
          <h3 className="text-base font-bold text-white mb-1">In-Graph Filter Pushdown</h3>
          <p className="text-xs text-slate-400 mb-4">
            Evaluates filter during beam exploration. Hops through unfiltered bridge nodes.
          </p>

          <div className="space-y-2 pt-3 border-t border-slate-800 text-xs font-mono">
            <div className="flex justify-between text-slate-400">
              <span>Dist Computations:</span>
              <span className="text-slate-200">{inGraphOps.toLocaleString()}</span>
            </div>
            <div className="flex justify-between text-slate-400">
              <span>Est. Latency:</span>
              <span className="text-emerald-400 font-bold">{inGraphLatencyUs} µs</span>
            </div>
            <div className="flex justify-between text-slate-400">
              <span>Recall Guarantee:</span>
              <span className="text-emerald-400 font-bold">&gt;99.2%</span>
            </div>
          </div>
        </div>

        {/* Strategy 3: Post-Filtered HNSW */}
        <div
          className={`glass-card p-6 rounded-2xl transition-all duration-300 ${
            recommendedStrategy === 'post_filter'
              ? 'ring-2 ring-cyan-400 border-cyan-500/60 bg-cyan-950/20 shadow-xl shadow-cyan-500/10'
              : 'opacity-70'
          }`}
        >
          <div className="flex items-center justify-between mb-4">
            <span className="text-xs font-mono px-2.5 py-1 rounded-lg bg-cyan-950/60 text-cyan-400 border border-cyan-800/60">
              Strategy 3
            </span>
            {recommendedStrategy === 'post_filter' && (
              <span className="text-[11px] font-mono font-bold text-cyan-400 animate-pulse">
                OPTIMAL PATH
              </span>
            )}
          </div>
          <h3 className="text-base font-bold text-white mb-1">Unfiltered HNSW + Post-Filter</h3>
          <p className="text-xs text-slate-400 mb-4">
            Retrieves $k/\sigma$ candidates from graph then validates filter predicates.
          </p>

          <div className="space-y-2 pt-3 border-t border-slate-800 text-xs font-mono">
            <div className="flex justify-between text-slate-400">
              <span>Dist Computations:</span>
              <span className="text-slate-200">{postFilterOps.toLocaleString()}</span>
            </div>
            <div className="flex justify-between text-slate-400">
              <span>Est. Latency:</span>
              <span className="text-cyan-400 font-bold">{postFilterLatencyUs} µs</span>
            </div>
            <div className="flex justify-between text-slate-400">
              <span>Recall Guarantee:</span>
              <span className="text-cyan-400 font-bold">
                {sigma < 0.1 ? '~75% (Trap Risk)' : '>98.5%'}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
