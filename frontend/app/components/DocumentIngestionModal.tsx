'use client'

import React, { useState } from 'react'
import { PlusCircle, X, CheckCircle2, AlertCircle, Sparkles, RefreshCw } from 'lucide-react'
import { generateClusterVector } from './VectorInput'

interface Props {
  isOpen: boolean
  onClose: () => void
  currentIndex: string
  currentDimension: number
  onDocumentInserted: () => void
}

export default function DocumentIngestionModal({
  isOpen,
  onClose,
  currentIndex,
  currentDimension,
  onDocumentInserted,
}: Props) {
  const [docId, setDocId] = useState<number>(() => Math.floor(1000 + Math.random() * 9000))
  const [title, setTitle] = useState('')
  const [category, setCategory] = useState('quantum')
  const [scoreRating, setScoreRating] = useState(9.8)
  const [metricVal, setMetricVal] = useState(2024)
  const [summary, setSummary] = useState('')
  const [tagsInput, setTagsInput] = useState('quantum, superconducting, qubit')
  const [vector, setVector] = useState<number[]>(() => generateClusterVector(0, 0.1))
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [successMsg, setSuccessMsg] = useState<string | null>(null)
  const [errorMsg, setErrorMsg] = useState<string | null>(null)

  if (!isOpen) return null

  const handleGenerateVector = () => {
    const catIdx = Math.floor(Math.random() * 5)
    setVector(generateClusterVector(catIdx, 0.2))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!title.trim()) {
      setErrorMsg('Please enter a document title.')
      return
    }

    setIsSubmitting(true)
    setErrorMsg(null)
    setSuccessMsg(null)

    const tags = tagsInput
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean)

    const payload = {
      id: docId,
      vector,
      metadata: {
        id: docId,
        title,
        category,
        status: scoreRating >= 9.0 ? 'HIGH_CONFIDENCE' : 'STANDARD',
        score_rating: scoreRating,
        metric_val: metricVal,
        summary,
        tags,
        created_at: new Date().toISOString(),
      },
    }

    try {
      const res = await fetch(`/v1/indexes/${currentIndex}/documents`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })

      const data = await res.json()

      if (!res.ok) {
        setErrorMsg(data.error?.message || data.error || 'Failed to insert document')
      } else {
        setSuccessMsg(`Document #${docId} inserted successfully into '${currentIndex}'!`)
        onDocumentInserted()
        setTimeout(() => {
          setSuccessMsg(null)
          onClose()
        }, 1200)
      }
    } catch (e: any) {
      setErrorMsg(e.message || 'Network error inserting document')
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-md">
      <div className="glass-panel w-full max-w-xl rounded-3xl p-6 border border-slate-700 shadow-2xl relative max-h-[90vh] overflow-y-auto">
        <button
          onClick={onClose}
          className="absolute top-5 right-5 p-1.5 rounded-xl bg-slate-900 border border-slate-800 text-slate-400 hover:text-white"
        >
          <X className="w-4 h-4" />
        </button>

        <div className="flex items-center gap-2.5 pb-4 border-b border-slate-800">
          <div className="w-8 h-8 rounded-xl bg-cyan-500/20 text-cyan-400 flex items-center justify-center border border-cyan-500/40">
            <PlusCircle className="w-4 h-4" />
          </div>
          <div>
            <h3 className="text-base font-bold text-white">Dynamic Vector Ingestion</h3>
            <p className="text-xs text-slate-400 font-mono">
              Target Index: <span className="text-cyan-400">{currentIndex}</span> ({currentDimension}-D)
            </p>
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
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-xs font-mono text-slate-400 mb-1">Document ID</label>
              <input
                type="number"
                value={docId}
                onChange={(e) => setDocId(parseInt(e.target.value, 10))}
                className="w-full text-xs font-mono bg-slate-900 border border-slate-800 rounded-xl px-3 py-2 text-white focus:border-cyan-500 focus:outline-none"
              />
            </div>

            <div>
              <label className="block text-xs font-mono text-slate-400 mb-1">Category</label>
              <input
                type="text"
                value={category}
                onChange={(e) => setCategory(e.target.value)}
                placeholder="e.g. quantum, robotics, finance"
                className="w-full text-xs font-mono bg-slate-900 border border-slate-800 rounded-xl px-3 py-2 text-white focus:border-cyan-500 focus:outline-none"
              />
            </div>
          </div>

          <div>
            <label className="block text-xs font-mono text-slate-400 mb-1">Document Title</label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="e.g. Topological Quantum Compilers with Anyonic Braiding"
              className="w-full text-xs font-sans bg-slate-900 border border-slate-800 rounded-xl px-3 py-2 text-white focus:border-cyan-500 focus:outline-none"
            />
          </div>

          <div>
            <label className="block text-xs font-mono text-slate-400 mb-1">Summary / Abstract</label>
            <textarea
              rows={2}
              value={summary}
              onChange={(e) => setSummary(e.target.value)}
              placeholder="Brief summary describing the vector document payload..."
              className="w-full text-xs font-sans bg-slate-900 border border-slate-800 rounded-xl px-3 py-2 text-slate-300 focus:border-cyan-500 focus:outline-none"
            />
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-xs font-mono text-slate-400 mb-1">
                Score Rating ({scoreRating.toFixed(1)} / 10)
              </label>
              <input
                type="range"
                min="0"
                max="10"
                step="0.1"
                value={scoreRating}
                onChange={(e) => setScoreRating(parseFloat(e.target.value))}
                className="w-full accent-cyan-400 cursor-pointer bg-slate-800 rounded-lg h-2"
              />
            </div>

            <div>
              <label className="block text-xs font-mono text-slate-400 mb-1">Tags (Comma-separated)</label>
              <input
                type="text"
                value={tagsInput}
                onChange={(e) => setTagsInput(e.target.value)}
                placeholder="quantum, physics, qiskit"
                className="w-full text-xs font-mono bg-slate-900 border border-slate-800 rounded-xl px-3 py-2 text-white focus:border-cyan-500 focus:outline-none"
              />
            </div>
          </div>

          {/* Vector generation section */}
          <div className="pt-2 border-t border-slate-800">
            <div className="flex items-center justify-between text-xs font-mono text-slate-400 mb-2">
              <span>{currentDimension}-D Embedding Vector (Normalized)</span>
              <button
                type="button"
                onClick={handleGenerateVector}
                className="flex items-center gap-1 text-[11px] text-cyan-400 hover:text-cyan-300"
              >
                <RefreshCw className="w-3 h-3" /> Re-synthesize
              </button>
            </div>

            <div className="h-8 w-full bg-slate-900 rounded-lg p-1 flex items-end gap-[1px] border border-slate-800">
              {vector.slice(0, currentDimension).map((val, idx) => (
                <div
                  key={idx}
                  style={{ height: `${Math.max(10, Math.abs(val) * 200)}%` }}
                  className="flex-1 bg-cyan-400/80 rounded-[1px]"
                />
              ))}
            </div>
          </div>

          <div className="flex justify-end gap-3 pt-3">
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
              className="flex items-center gap-1.5 px-5 py-2 rounded-xl bg-gradient-to-r from-cyan-500 to-blue-600 text-slate-950 font-bold text-xs hover:from-cyan-400 hover:to-blue-500 transition-all disabled:opacity-50"
            >
              <PlusCircle className="w-3.5 h-3.5" />
              {isSubmitting ? 'Ingesting…' : 'Ingest Document'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}
