'use client'

import React, { useState } from 'react'
import { Layers, X, Plus, CheckCircle2, AlertCircle } from 'lucide-react'

interface Props {
  isOpen: boolean
  onClose: () => void
  onIndexCreated: (newIndexId: string) => void
}

export default function IndexManagerModal({ isOpen, onClose, onIndexCreated }: Props) {
  const [indexId, setIndexId] = useState('')
  const [dimension, setDimension] = useState(128)
  const [defaultEf, setDefaultEf] = useState(64)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [errorMsg, setErrorMsg] = useState<string | null>(null)
  const [successMsg, setSuccessMsg] = useState<string | null>(null)

  if (!isOpen) return null

  const dimensionPresets = [
    { label: '32-D (Micro)', val: 32 },
    { label: '64-D (Fast)', val: 64 },
    { label: '128-D (Standard)', val: 128 },
    { label: '256-D (Audio/Vision)', val: 256 },
    { label: '384-D (MiniLM)', val: 384 },
    { label: '768-D (BERT/RoBERTa)', val: 768 },
    { label: '1536-D (OpenAI text-3)', val: 1536 },
  ]

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!indexId.trim()) {
      setErrorMsg('Please enter a valid index ID.')
      return
    }

    setIsSubmitting(true)
    setErrorMsg(null)
    setSuccessMsg(null)

    try {
      const res = await fetch('/v1/indexes', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          index_id: indexId.trim(),
          dimension,
          default_ef: defaultEf,
        }),
      })

      const data = await res.json()

      if (!res.ok) {
        setErrorMsg(data.error || 'Failed to create index')
      } else {
        setSuccessMsg(`Index '${indexId}' created successfully!`)
        onIndexCreated(indexId.trim())
        setTimeout(() => {
          setSuccessMsg(null)
          onClose()
        }, 1000)
      }
    } catch (e: any) {
      setErrorMsg(e.message || 'Network error creating index')
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-md">
      <div className="glass-panel w-full max-w-md rounded-3xl p-6 border border-slate-700 shadow-2xl relative">
        <button
          onClick={onClose}
          className="absolute top-5 right-5 p-1.5 rounded-xl bg-slate-900 border border-slate-800 text-slate-400 hover:text-white"
        >
          <X className="w-4 h-4" />
        </button>

        <div className="flex items-center gap-2.5 pb-4 border-b border-slate-800">
          <div className="w-8 h-8 rounded-xl bg-purple-500/20 text-purple-400 flex items-center justify-center border border-purple-500/40">
            <Layers className="w-4 h-4" />
          </div>
          <div>
            <h3 className="text-base font-bold text-white">Create New Vector Index</h3>
            <p className="text-xs text-slate-400 font-mono">Dynamic Multi-Index Registry</p>
          </div>
        </div>

        {successMsg && (
          <div className="mt-4 p-3 rounded-xl bg-emerald-950/50 border border-emerald-800 text-emerald-300 text-xs flex items-center gap-2 font-mono">
            <CheckCircle2 className="w-4 h-4 shrink-0 text-emerald-400" />
            <span>{successMsg}</span>
          </div>
        )}

        {errorMsg && (
          <div className="mt-4 p-3 rounded-xl bg-rose-950/50 border border-rose-800 text-rose-300 text-xs flex items-center gap-2 font-mono">
            <AlertCircle className="w-4 h-4 shrink-0 text-rose-400" />
            <span>{errorMsg}</span>
          </div>
        )}

        <form onSubmit={handleSubmit} className="mt-4 space-y-4">
          <div>
            <label className="block text-xs font-mono text-slate-400 mb-1">Index Identifier</label>
            <input
              type="text"
              value={indexId}
              onChange={(e) => setIndexId(e.target.value)}
              placeholder="e.g. text-embedding-3-small"
              className="w-full text-xs font-mono bg-slate-900 border border-slate-800 rounded-xl px-3 py-2 text-white focus:border-cyan-500 focus:outline-none"
            />
          </div>

          <div>
            <label className="block text-xs font-mono text-slate-400 mb-1">
              Vector Dimension ({dimension}-D)
            </label>
            <div className="grid grid-cols-2 gap-1.5 mb-2">
              {dimensionPresets.map((p) => (
                <button
                  type="button"
                  key={p.val}
                  onClick={() => setDimension(p.val)}
                  className={`text-[11px] font-mono p-1.5 rounded-lg border text-left transition-all ${
                    dimension === p.val
                      ? 'bg-purple-950/60 border-purple-500 text-purple-300 font-bold'
                      : 'bg-slate-900 border-slate-800 text-slate-400 hover:text-slate-200'
                  }`}
                >
                  {p.label}
                </button>
              ))}
            </div>
            <input
              type="number"
              value={dimension}
              onChange={(e) => setDimension(parseInt(e.target.value, 10))}
              className="w-full text-xs font-mono bg-slate-900 border border-slate-800 rounded-xl px-3 py-1.5 text-white"
            />
          </div>

          <div>
            <label className="block text-xs font-mono text-slate-400 mb-1">
              Default Exploration Beam (`ef_search`: {defaultEf})
            </label>
            <input
              type="range"
              min="16"
              max="256"
              step="16"
              value={defaultEf}
              onChange={(e) => setDefaultEf(parseInt(e.target.value, 10))}
              className="w-full accent-purple-400 cursor-pointer bg-slate-800 rounded-lg h-2"
            />
          </div>

          <div className="flex justify-end gap-3 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 rounded-xl bg-slate-900 border border-slate-800 text-slate-400 text-xs hover:text-white"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSubmitting}
              className="flex items-center gap-1.5 px-5 py-2 rounded-xl bg-gradient-to-r from-purple-500 to-indigo-600 text-white font-bold text-xs hover:from-purple-400 hover:to-indigo-500 transition-all disabled:opacity-50"
            >
              <Plus className="w-3.5 h-3.5" />
              {isSubmitting ? 'Creating…' : 'Create Index'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}
