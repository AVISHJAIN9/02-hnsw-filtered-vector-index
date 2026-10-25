'use client'

import React, { useState } from 'react'
import { Sparkles, RefreshCw, Sliders, FileJson } from 'lucide-react'

export interface VectorPreset {
  name: string
  category: string
  description: string
  vector: number[]
}

// Deterministic semantic cluster generator for 128-dim vectors
export function generateClusterVector(categoryIndex: number, noise: number = 0.1): number[] {
  const vec: number[] = []
  for (let d = 0; d < 128; d++) {
    const base = ((d * 17 + categoryIndex * 53) % 100) / 100.0 - 0.5
    const clusterBoost = d >= categoryIndex * 24 && d < (categoryIndex + 1) * 24 ? 1.6 : 0.1
    const randomPerturbation = (Math.random() - 0.5) * noise
    vec.push((base * clusterBoost) + randomPerturbation)
  }

  // L2 normalize
  const norm = Math.sqrt(vec.reduce((acc, v) => acc + v * v, 0)) || 1e-6
  return vec.map((v) => Number((v / norm).toFixed(5)))
}

export const PRESETS: VectorPreset[] = [
  {
    name: 'LLM & Attention Research',
    category: 'nlp_research',
    description: 'Queries FlashAttention, RoPE scaling, Speculative Decoding, and QLoRA architectures.',
    vector: generateClusterVector(1, 0.05),
  },
  {
    name: 'Zero-Day / Cyber Anomaly',
    category: 'cybersecurity',
    description: 'Privilege escalation, eBPF telemetry, memory heap spray, and lateral movement.',
    vector: generateClusterVector(0, 0.05),
  },
  {
    name: 'High-Frequency Market Depth',
    category: 'financial_tech',
    description: 'Sub-microsecond orderbook state machines, arbitrage detection, and kernel bypass.',
    vector: generateClusterVector(2, 0.05),
  },
  {
    name: 'High-End Hardware & Servers',
    category: 'e_commerce',
    description: 'Hopper SXM5 GPU nodes, custom Hall-effect keyboards, and 4K reference monitors.',
    vector: generateClusterVector(3, 0.05),
  },
  {
    name: 'AlphaFold Biomolecular Assays',
    category: 'biomedical',
    description: 'Protein structure diffusion, CRISPR-Cas12a editors, and Cryo-EM reconstructions.',
    vector: generateClusterVector(4, 0.05),
  },
]

interface VectorInputProps {
  vector: number[]
  onChange: (vec: number[]) => void
  normalize: boolean
  onToggleNormalize: (norm: boolean) => void
}

export default function VectorInput({
  vector,
  onChange,
  normalize,
  onToggleNormalize,
}: VectorInputProps) {
  const [activeTab, setActiveTab] = useState<'presets' | 'custom'>('presets')
  const [customText, setCustomText] = useState('')
  const [selectedPresetName, setSelectedPresetName] = useState('LLM & Attention Research')

  const handleSelectPreset = (preset: VectorPreset) => {
    setSelectedPresetName(preset.name)
    onChange(preset.vector)
  }

  const handleRandomize = () => {
    const randomCat = Math.floor(Math.random() * 5)
    const randomized = generateClusterVector(randomCat, 0.35)
    setSelectedPresetName('Custom Randomized')
    onChange(randomized)
  }

  const handleApplyCustomJson = () => {
    try {
      const parsed = JSON.parse(customText)
      if (Array.isArray(parsed) && parsed.length === 128) {
        onChange(parsed.map(Number))
        setSelectedPresetName('Custom JSON')
      } else {
        alert('Expected a JSON array of exactly 128 float numbers.')
      }
    } catch (e) {
      alert('Invalid JSON array format.')
    }
  }

  return (
    <div className="glass-panel p-5 rounded-2xl">
      <div className="flex items-center justify-between pb-3 border-b border-slate-800">
        <div className="flex items-center gap-2">
          <Sparkles className="w-4 h-4 text-cyan-400" />
          <h3 className="font-semibold text-white text-sm">Query Vector Constructor</h3>
          <span className="text-[11px] font-mono px-2 py-0.5 rounded bg-cyan-950/60 text-cyan-400 border border-cyan-800/60">
            dim = 128 (Float32)
          </span>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={handleRandomize}
            className="flex items-center gap-1.5 text-xs font-mono px-2.5 py-1.5 rounded-lg bg-slate-800/80 hover:bg-slate-700 text-slate-300 transition-colors"
          >
            <RefreshCw className="w-3 h-3 text-cyan-400" />
            Random Perturb
          </button>
        </div>
      </div>

      {/* Preset selector pills */}
      <div className="mt-4">
        <div className="flex gap-2 mb-3">
          <button
            onClick={() => setActiveTab('presets')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all ${
              activeTab === 'presets'
                ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Sliders className="w-3.5 h-3.5" /> Semantic Presets
          </button>
          <button
            onClick={() => {
              setActiveTab('custom')
              setCustomText(JSON.stringify(vector))
            }}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all ${
              activeTab === 'custom'
                ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <FileJson className="w-3.5 h-3.5" /> Raw JSON Vector
          </button>
        </div>

        {activeTab === 'presets' ? (
          <div className="grid grid-cols-1 md:grid-cols-3 gap-2">
            {PRESETS.map((p) => {
              const isSelected = selectedPresetName === p.name
              return (
                <button
                  key={p.name}
                  onClick={() => handleSelectPreset(p)}
                  className={`text-left p-2.5 rounded-xl border transition-all ${
                    isSelected
                      ? 'bg-cyan-950/40 border-cyan-500/60 shadow-lg shadow-cyan-500/10'
                      : 'bg-slate-900/50 border-slate-800/80 hover:border-slate-700 text-slate-400'
                  }`}
                >
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-xs font-semibold text-slate-200">{p.name}</span>
                    <span className="text-[10px] font-mono uppercase px-1.5 py-0.5 rounded bg-slate-800 text-slate-300">
                      {p.category.replace('_', ' ')}
                    </span>
                  </div>
                  <p className="text-[11px] text-slate-400 line-clamp-2 leading-relaxed">
                    {p.description}
                  </p>
                </button>
              )
            })}
          </div>
        ) : (
          <div className="space-y-2">
            <textarea
              rows={3}
              value={customText}
              onChange={(e) => setCustomText(e.target.value)}
              className="w-full text-xs font-mono bg-slate-900 border border-slate-800 rounded-xl p-3 text-cyan-300 focus:border-cyan-500 focus:outline-none"
              placeholder="[0.05, -0.12, 0.44, ... 128 elements]"
            />
            <button
              onClick={handleApplyCustomJson}
              className="px-3 py-1 rounded-lg bg-cyan-500 text-slate-950 font-semibold text-xs hover:bg-cyan-400"
            >
              Parse & Load Vector
            </button>
          </div>
        )}
      </div>

      {/* Vector Heatmap Visualizer */}
      <div className="mt-4 pt-3 border-t border-slate-800/80">
        <div className="flex items-center justify-between text-xs font-mono text-slate-400 mb-2">
          <span>128-Dimension Intensity Heatmap</span>
          <label className="flex items-center gap-1.5 cursor-pointer text-[11px]">
            <input
              type="checkbox"
              checked={normalize}
              onChange={(e) => onToggleNormalize(e.target.checked)}
              className="accent-cyan-400 rounded"
            />
            Enforce L2 Unit Normalization
          </label>
        </div>

        <div className="h-10 w-full bg-slate-900/90 rounded-lg p-1.5 flex items-end gap-[1px] border border-slate-800">
          {vector.slice(0, 128).map((val, idx) => {
            const heightPercent = Math.min(100, Math.max(8, Math.abs(val) * 280))
            const isPositive = val >= 0
            return (
              <div
                key={idx}
                title={`Dim #${idx}: ${val}`}
                style={{ height: `${heightPercent}%` }}
                className={`flex-1 rounded-[1px] transition-all duration-300 ${
                  isPositive ? 'bg-cyan-400/80 hover:bg-cyan-300' : 'bg-rose-400/80 hover:bg-rose-300'
                }`}
              />
            )
          })}
        </div>
      </div>
    </div>
  )
}
