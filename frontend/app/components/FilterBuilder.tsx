'use client'

import React from 'react'
import { Filter, Cpu, Target, Gauge } from 'lucide-react'

export interface FilterState {
  category: string
  minRating: number
  status: string
  k: number
  efSearch: number
  metric: 'euclidean' | 'cosine' | 'dot_product'
  mode: 'auto' | 'approximate' | 'exact'
}

interface FilterBuilderProps {
  filters: FilterState
  onChange: (filters: FilterState) => void
}

export default function FilterBuilder({ filters, onChange }: FilterBuilderProps) {
  const categories = [
    { label: 'All Domains (Unfiltered)', value: '' },
    { label: 'Cybersecurity Threat Telemetry', value: 'cybersecurity' },
    { label: 'NLP & LLM Architectures', value: 'nlp_research' },
    { label: 'High-Frequency FinTech', value: 'financial_tech' },
    { label: 'E-Commerce & Server Hardware', value: 'e_commerce' },
    { label: 'Biomedical & Genomics', value: 'biomedical' },
  ]

  const update = (partial: Partial<FilterState>) => {
    onChange({ ...filters, ...partial })
  }

  return (
    <div className="glass-panel p-5 rounded-2xl space-y-4">
      <div className="flex items-center justify-between pb-3 border-b border-slate-800">
        <div className="flex items-center gap-2">
          <Filter className="w-4 h-4 text-cyan-400" />
          <h3 className="font-semibold text-white text-sm">Metadata Filter Predicate & Index Tuning</h3>
        </div>
        <span className="text-[11px] font-mono text-slate-400">Roaring Bitmap AST</span>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Category Filter */}
        <div>
          <label className="block text-xs font-mono text-slate-400 mb-1.5">
            Category Field Filter (`category == $val`)
          </label>
          <select
            value={filters.category}
            onChange={(e) => update({ category: e.target.value })}
            className="w-full text-xs font-mono bg-slate-900 border border-slate-800 rounded-xl px-3 py-2 text-slate-200 focus:border-cyan-500 focus:outline-none"
          >
            {categories.map((c) => (
              <option key={c.value} value={c.value}>
                {c.label}
              </option>
            ))}
          </select>
        </div>

        {/* Min Score Rating Slider */}
        <div>
          <div className="flex justify-between items-center text-xs font-mono text-slate-400 mb-1.5">
            <span>Minimum Rating (`score_rating &gt;=`)</span>
            <span className="text-cyan-400 font-bold">{filters.minRating.toFixed(1)}</span>
          </div>
          <input
            type="range"
            min="0"
            max="10"
            step="0.5"
            value={filters.minRating}
            onChange={(e) => update({ minRating: parseFloat(e.target.value) })}
            className="w-full accent-cyan-400 cursor-pointer bg-slate-800 rounded-lg h-2"
          />
          <div className="flex justify-between text-[10px] font-mono text-slate-500 mt-1">
            <span>0.0 (Pass-through)</span>
            <span>9.5+ (High-selectivity)</span>
          </div>
        </div>

        {/* Distance Metric */}
        <div>
          <label className="block text-xs font-mono text-slate-400 mb-1.5">
            Distance Metric
          </label>
          <select
            value={filters.metric}
            onChange={(e) => update({ metric: e.target.value as any })}
            className="w-full text-xs font-mono bg-slate-900 border border-slate-800 rounded-xl px-3 py-2 text-slate-200 focus:border-cyan-500 focus:outline-none"
          >
            <option value="cosine">Cosine Distance (1 - u · v)</option>
            <option value="euclidean">Euclidean (L2 Squared)</option>
            <option value="dot_product">Dot Product Inner Product</option>
          </select>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 pt-2 border-t border-slate-800/60">
        {/* Top-K Slider */}
        <div>
          <div className="flex justify-between items-center text-xs font-mono text-slate-400 mb-1.5">
            <span className="flex items-center gap-1.5">
              <Target className="w-3.5 h-3.5 text-cyan-400" />
              Top-K Nearest Neighbors
            </span>
            <span className="text-cyan-400 font-bold">{filters.k}</span>
          </div>
          <input
            type="range"
            min="1"
            max="20"
            step="1"
            value={filters.k}
            onChange={(e) => update({ k: parseInt(e.target.value, 10) })}
            className="w-full accent-cyan-400 cursor-pointer bg-slate-800 rounded-lg h-2"
          />
        </div>

        {/* ef_search beam width */}
        <div>
          <div className="flex justify-between items-center text-xs font-mono text-slate-400 mb-1.5">
            <span className="flex items-center gap-1.5">
              <Gauge className="w-3.5 h-3.5 text-purple-400" />
              Exploration Beam (`ef_search`)
            </span>
            <span className="text-purple-400 font-bold">{filters.efSearch}</span>
          </div>
          <input
            type="range"
            min="10"
            max="256"
            step="8"
            value={filters.efSearch}
            onChange={(e) => update({ efSearch: parseInt(e.target.value, 10) })}
            className="w-full accent-purple-400 cursor-pointer bg-slate-800 rounded-lg h-2"
          />
        </div>

        {/* Execution Mode */}
        <div>
          <label className="flex items-center gap-1.5 text-xs font-mono text-slate-400 mb-1.5">
            <Cpu className="w-3.5 h-3.5 text-emerald-400" />
            Query Mode
          </label>
          <div className="grid grid-cols-3 gap-1 bg-slate-900 p-1 rounded-xl border border-slate-800">
            {(['auto', 'approximate', 'exact'] as const).map((m) => (
              <button
                key={m}
                onClick={() => update({ mode: m })}
                className={`text-[11px] font-mono py-1 rounded-lg capitalize transition-all ${
                  filters.mode === m
                    ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 font-semibold'
                    : 'text-slate-400 hover:text-slate-200'
                }`}
              >
                {m}
              </button>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
