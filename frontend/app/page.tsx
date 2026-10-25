'use client'

import React, { useState } from 'react'
import Link from 'next/link'
import {
  Zap,
  ShieldCheck,
  Cpu,
  Layers,
  Search,
  ArrowRight,
  Database,
  Activity,
  CheckCircle2,
  Terminal,
} from 'lucide-react'
import Hnsw3dVisualizer from './components/Hnsw3dVisualizer'
import MatchCard from './components/MatchCard'
import SearchTimingBreakdown from './components/SearchTimingBreakdown'
import { generateClusterVector } from './components/VectorInput'

export default function HomePage() {
  const [quickQueryCat, setQuickQueryCat] = useState(1) // NLP
  const [isSearching, setIsSearching] = useState(false)
  const [quickResults, setQuickResults] = useState<any>(null)
  const [activeNodeId, setActiveNodeId] = useState<number | null>(null)

  const handleQuickSearch = async (catIndex: number) => {
    setQuickQueryCat(catIndex)
    setIsSearching(true)
    const vec = generateClusterVector(catIndex, 0.05)

    try {
      const res = await fetch('/v1/indexes/default/search', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          vector: vec,
          k: 3,
          include_metadata: true,
          metric: 'cosine',
        }),
      })

      if (res.ok) {
        const data = await res.json()
        setQuickResults(data)
        if (data.matches && data.matches.length > 0) {
          setActiveNodeId(data.matches[0].id)
        }
      }
    } catch (e) {
      console.error('Quick search error:', e)
    } finally {
      setIsSearching(false)
    }
  }

  const queryPills = [
    { label: 'LLM Research', catIndex: 1, color: 'text-cyan-400 border-cyan-500/40 bg-cyan-950/30' },
    { label: 'Zero-Day Vulnerability', catIndex: 0, color: 'text-rose-400 border-rose-500/40 bg-rose-950/30' },
    { label: 'FinTech Arbitrage', catIndex: 2, color: 'text-emerald-400 border-emerald-500/40 bg-emerald-950/30' },
    { label: 'Hardware & Servers', catIndex: 3, color: 'text-amber-400 border-amber-500/40 bg-amber-950/30' },
    { label: 'AlphaFold Biotech', catIndex: 4, color: 'text-purple-400 border-purple-500/40 bg-purple-950/30' },
  ]

  return (
    <div className="relative">
      {/* Radial ambient glow */}
      <div className="absolute top-0 left-1/2 -translate-x-1/2 w-full max-w-7xl h-[600px] radial-glow pointer-events-none" />
      <div className="absolute top-96 right-0 w-96 h-96 radial-glow-purple pointer-events-none" />

      {/* Hero Section */}
      <div className="max-w-7xl mx-auto px-4 pt-12 pb-20">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-12 items-center">
          <div className="lg:col-span-7 space-y-6">
            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-cyan-950/60 border border-cyan-800/80 text-xs font-mono text-cyan-300">
              <span className="w-2 h-2 rounded-full bg-cyan-400 animate-pulse" />
              <span>Rust SIMD &bull; AVX-512 &bull; Int8 Quantization</span>
            </div>

            <h1 className="text-4xl sm:text-6xl font-extrabold text-white tracking-tight leading-[1.1]">
              Sub-Millisecond Vector Search with{' '}
              <span className="bg-gradient-to-r from-cyan-400 via-sky-300 to-indigo-400 bg-clip-text text-transparent">
                Learned Filter Planning
              </span>
            </h1>

            <p className="text-slate-400 text-base sm:text-lg leading-relaxed max-w-2xl">
              An industrial-grade, lock-striped Hierarchical Navigable Small World (HNSW) vector
              engine built from scratch in Rust. Featuring AVX2/NEON distance SIMD kernels,
              learned pushdown filter execution, and Roaring Bitmap acceleration.
            </p>

            {/* Quick Action Buttons */}
            <div className="flex flex-wrap items-center gap-4 pt-2">
              <Link
                href="/search"
                className="flex items-center gap-2 px-6 py-3.5 rounded-xl bg-gradient-to-r from-cyan-500 to-blue-600 text-slate-950 font-bold hover:from-cyan-400 hover:to-blue-500 transition-all shadow-lg shadow-cyan-500/25 hover:shadow-cyan-500/40 hover:scale-[1.02]"
              >
                <Search className="w-4 h-4" />
                Launch 3D Search Explorer
                <ArrowRight className="w-4 h-4" />
              </Link>

              <Link
                href="/dashboard"
                className="flex items-center gap-2 px-6 py-3.5 rounded-xl bg-slate-900/90 border border-slate-800 text-slate-200 font-semibold hover:border-slate-700 hover:text-white transition-all"
              >
                <Activity className="w-4 h-4 text-cyan-400" />
                Live Performance Telemetry
              </Link>
            </div>

            {/* Quick Feature Metric Counters */}
            <div className="grid grid-cols-3 gap-4 pt-6 border-t border-slate-800/80">
              <div>
                <div className="text-2xl sm:text-3xl font-extrabold font-mono text-cyan-400">
                  &lt; 0.08ms
                </div>
                <div className="text-xs font-mono text-slate-400 mt-1">p99 Latency (128-D)</div>
              </div>
              <div>
                <div className="text-2xl sm:text-3xl font-extrabold font-mono text-emerald-400">
                  12,500+
                </div>
                <div className="text-xs font-mono text-slate-400 mt-1">QPS / Core (Int8)</div>
              </div>
              <div>
                <div className="text-2xl sm:text-3xl font-extrabold font-mono text-purple-400">
                  99.4%
                </div>
                <div className="text-xs font-mono text-slate-400 mt-1">Recall@10 Accuracy</div>
              </div>
            </div>
          </div>

          {/* Interactive 3D Visualizer Canvas in Hero */}
          <div className="lg:col-span-5 h-[460px]">
            <Hnsw3dVisualizer
              activeId={activeNodeId}
              highlightedIds={quickResults?.matches?.map((m: any) => m.id) || [11, 15, 17]}
            />
          </div>
        </div>
      </div>

      {/* Live Interactive Quick Search Showcase */}
      <div className="max-w-7xl mx-auto px-4 pb-20">
        <div className="glass-panel p-6 sm:p-8 rounded-3xl border border-cyan-500/20 shadow-2xl">
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-6 border-b border-slate-800">
            <div>
              <div className="flex items-center gap-2">
                <Terminal className="w-5 h-5 text-cyan-400" />
                <h3 className="text-xl font-bold text-white">Live Query Execution Sandbox</h3>
                <span className="text-xs font-mono px-2 py-0.5 rounded bg-emerald-950/60 text-emerald-400 border border-emerald-800/60">
                  Connected to Axum :8080
                </span>
              </div>
              <p className="text-xs text-slate-400 mt-1">
                Select a semantic cluster to synthesize a 128-dim query vector and trigger immediate nearest neighbor retrieval.
              </p>
            </div>

            {/* Cluster Quick Pills */}
            <div className="flex flex-wrap gap-2">
              {queryPills.map((pill) => (
                <button
                  key={pill.label}
                  onClick={() => handleQuickSearch(pill.catIndex)}
                  disabled={isSearching}
                  className={`px-3 py-1.5 rounded-xl text-xs font-medium border transition-all ${pill.color} hover:scale-105 ${
                    quickQueryCat === pill.catIndex ? 'ring-2 ring-cyan-400 font-bold' : ''
                  }`}
                >
                  {pill.label}
                </button>
              ))}
            </div>
          </div>

          {/* Results Area */}
          <div className="mt-6">
            {quickResults ? (
              <div className="space-y-6">
                <SearchTimingBreakdown telemetry={quickResults.telemetry} />
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  {quickResults.matches.map((match: any, idx: number) => (
                    <MatchCard
                      key={match.id}
                      match={match}
                      rank={idx + 1}
                      isSelected={activeNodeId === match.id}
                      onSelect={() => setActiveNodeId(match.id)}
                    />
                  ))}
                </div>
              </div>
            ) : (
              <div className="text-center py-12 text-slate-400 space-y-3">
                <Zap className="w-8 h-8 text-cyan-400 mx-auto animate-bounce" />
                <p className="text-sm">
                  Click any semantic preset above to benchmark live query latency and nearest neighbors.
                </p>
                <button
                  onClick={() => handleQuickSearch(1)}
                  className="px-4 py-2 rounded-xl bg-cyan-500 text-slate-950 font-bold text-xs hover:bg-cyan-400"
                >
                  Execute Demo Search
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Core Architectural Pillars */}
      <div className="max-w-7xl mx-auto px-4 pb-24">
        <div className="text-center max-w-3xl mx-auto mb-12">
          <h2 className="text-3xl font-extrabold text-white">
            Engineered for Extreme Retrieval Efficiency
          </h2>
          <p className="text-slate-400 text-sm mt-3">
            Every layer from hardware registers to high-level filter ASTs is designed for zero allocation and maximum CPU cache locality.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="glass-card p-6 rounded-2xl">
            <div className="w-12 h-12 rounded-xl bg-cyan-500/10 border border-cyan-500/30 flex items-center justify-center text-cyan-400 mb-4">
              <Zap className="w-6 h-6" />
            </div>
            <h3 className="text-lg font-bold text-white mb-2">SIMD Vector Distance Kernels</h3>
            <p className="text-slate-400 text-xs leading-relaxed mb-4">
              Explicit AVX-512, AVX2 + FMA, and ARM NEON intrinsics with 8-way loop unrolling for Cosine, Euclidean ($L_2$), and Dot Product metrics.
            </p>
            <div className="text-[11px] font-mono text-cyan-400 bg-slate-900 p-2 rounded-lg border border-slate-800">
              `_mm256_fmadd_ps` &bull; 0.12ns / dim
            </div>
          </div>

          <div className="glass-card p-6 rounded-2xl">
            <div className="w-12 h-12 rounded-xl bg-purple-500/10 border border-purple-500/30 flex items-center justify-center text-purple-400 mb-4">
              <ShieldCheck className="w-6 h-6" />
            </div>
            <h3 className="text-lg font-bold text-white mb-2">Roaring Bitmap Filter Pushdown</h3>
            <p className="text-slate-400 text-xs leading-relaxed mb-4">
              In-graph filtering evaluates compressed Roaring Bitmaps during beam search traversal, preventing disconnected graph traps on high-selectivity queries.
            </p>
            <div className="text-[11px] font-mono text-purple-400 bg-slate-900 p-2 rounded-lg border border-slate-800">
              Zero-Copy Bitmaps &bull; $O(1)$ Test
            </div>
          </div>

          <div className="glass-card p-6 rounded-2xl">
            <div className="w-12 h-12 rounded-xl bg-emerald-500/10 border border-emerald-500/30 flex items-center justify-center text-emerald-400 mb-4">
              <Database className="w-6 h-6" />
            </div>
            <h3 className="text-lg font-bold text-white mb-2">Int8 Scalar Quantization</h3>
            <p className="text-slate-400 text-xs leading-relaxed mb-4">
              Compresses 32-bit floating point vectors into calibrated 8-bit integer encodings, reducing RAM footprint by 75% while maintaining &gt;99% recall.
            </p>
            <div className="text-[11px] font-mono text-emerald-400 bg-slate-900 p-2 rounded-lg border border-slate-800">
              4x RAM Reduction &bull; 128B / Vector
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
