'use client'

import React, { useState } from 'react'
import { Tag, Star, ChevronDown, ChevronUp, Code2, ExternalLink } from 'lucide-react'

export interface SearchMatch {
  id: number
  distance: number
  score: number
  metadata?: {
    id?: number
    title?: string
    category?: string
    status?: string
    score_rating?: number
    metric_val?: number
    summary?: string
    tags?: string[]
    created_at?: string
    [key: string]: any
  }
}

interface Props {
  match: SearchMatch
  rank: number
  isSelected?: boolean
  onSelect?: () => void
}

export default function MatchCard({ match, rank, isSelected, onSelect }: Props) {
  const [showJson, setShowJson] = useState(false)
  const meta = match.metadata || {}

  const categoryColors: Record<string, { bg: string; text: string; border: string }> = {
    cybersecurity: { bg: 'bg-rose-950/40', text: 'text-rose-400', border: 'border-rose-800/60' },
    nlp_research: { bg: 'bg-cyan-950/40', text: 'text-cyan-400', border: 'border-cyan-800/60' },
    financial_tech: { bg: 'bg-emerald-950/40', text: 'text-emerald-400', border: 'border-emerald-800/60' },
    e_commerce: { bg: 'bg-amber-950/40', text: 'text-amber-400', border: 'border-amber-800/60' },
    biomedical: { bg: 'bg-purple-950/40', text: 'text-purple-400', border: 'border-purple-800/60' },
  }

  const catStyle = categoryColors[meta.category || ''] || {
    bg: 'bg-slate-900',
    text: 'text-slate-300',
    border: 'border-slate-800',
  }

  return (
    <div
      onClick={onSelect}
      className={`glass-card p-5 rounded-2xl cursor-pointer transition-all duration-300 ${
        isSelected
          ? 'border-cyan-400 ring-2 ring-cyan-500/20 shadow-xl shadow-cyan-500/10'
          : 'hover:border-slate-700'
      }`}
    >
      <div className="flex items-start justify-between gap-3 mb-2">
        <div className="flex items-center gap-2">
          <span className="flex items-center justify-center w-6 h-6 rounded-lg bg-cyan-500/20 text-cyan-400 font-mono font-bold text-xs border border-cyan-500/40">
            #{rank}
          </span>
          <span className={`text-[11px] font-mono px-2.5 py-0.5 rounded-full border uppercase ${catStyle.bg} ${catStyle.text} ${catStyle.border}`}>
            {meta.category?.replace('_', ' ') || 'UNCLASSIFIED'}
          </span>
          <span className="text-xs font-mono text-slate-500">ID: {match.id}</span>
        </div>

        {/* Similarity Score and Distance */}
        <div className="flex items-center gap-2 text-right">
          <div className="bg-slate-900/90 border border-slate-800 px-2.5 py-1 rounded-xl">
            <div className="text-[10px] font-mono text-slate-400">Score</div>
            <div className="text-xs font-bold font-mono text-cyan-400">
              {(match.score * 100).toFixed(2)}%
            </div>
          </div>
          <div className="bg-slate-900/90 border border-slate-800 px-2.5 py-1 rounded-xl">
            <div className="text-[10px] font-mono text-slate-400">Distance</div>
            <div className="text-xs font-bold font-mono text-slate-200">
              {match.distance.toFixed(4)}
            </div>
          </div>
        </div>
      </div>

      {/* Document Title */}
      <h4 className="text-base font-semibold text-white mt-2 leading-snug">
        {meta.title || `Document #${match.id}`}
      </h4>

      {/* Summary */}
      {meta.summary && (
        <p className="text-xs text-slate-400 mt-2 leading-relaxed">
          {meta.summary}
        </p>
      )}

      {/* Tags and Ratings */}
      <div className="flex flex-wrap items-center justify-between gap-2 mt-4 pt-3 border-t border-slate-800/80">
        <div className="flex flex-wrap gap-1.5">
          {meta.tags?.map((t: string) => (
            <span
              key={t}
              className="flex items-center gap-1 text-[11px] font-mono px-2 py-0.5 rounded-md bg-slate-900 text-slate-300 border border-slate-800"
            >
              <Tag className="w-2.5 h-2.5 text-cyan-400" />
              {t}
            </span>
          ))}
        </div>

        <div className="flex items-center gap-3 text-xs font-mono text-slate-400">
          {meta.score_rating && (
            <span className="flex items-center gap-1 text-amber-400">
              <Star className="w-3.5 h-3.5 fill-amber-400" />
              {meta.score_rating.toFixed(1)} / 10
            </span>
          )}

          {meta.metric_val && (
            <span className="text-slate-300">
              {meta.metric_val > 1000 ? `$${meta.metric_val.toLocaleString()}` : `Yr: ${meta.metric_val}`}
            </span>
          )}

          <button
            onClick={(e) => {
              e.stopPropagation()
              setShowJson(!showJson)
            }}
            className="flex items-center gap-1 text-[11px] text-cyan-400 hover:text-cyan-300 px-2 py-1 rounded bg-slate-900 border border-slate-800"
          >
            <Code2 className="w-3 h-3" />
            {showJson ? 'Hide JSON' : 'JSON'}
            {showJson ? <ChevronUp className="w-3 h-3" /> : <ChevronDown className="w-3 h-3" />}
          </button>
        </div>
      </div>

      {/* Collapsible JSON Drawer */}
      {showJson && (
        <pre className="mt-3 p-3 rounded-xl bg-slate-950 border border-slate-800 text-[11px] font-mono text-emerald-400 overflow-x-auto">
          {JSON.stringify(meta, null, 2)}
        </pre>
      )}
    </div>
  )
}
