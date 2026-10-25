'use client'

import React from 'react'
import { Download, FileJson, FileSpreadsheet } from 'lucide-react'
import { SearchMatch } from './MatchCard'
import { SearchTelemetry } from './SearchTimingBreakdown'

interface Props {
  matches: SearchMatch[]
  telemetry: SearchTelemetry | null
  indexId: string
}

export default function ExportResults({ matches, telemetry, indexId }: Props) {
  if (!matches || matches.length === 0) return null

  const handleExportJson = () => {
    const dataToExport = {
      index_id: indexId,
      exported_at: new Date().toISOString(),
      total_matches: matches.length,
      telemetry,
      matches,
    }

    const blob = new Blob([JSON.stringify(dataToExport, null, 2)], {
      type: 'application/json',
    })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `hnsw_search_${indexId}_${Date.now()}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  const handleExportCsv = () => {
    const headers = [
      'Rank',
      'ID',
      'SimilarityScore',
      'Distance',
      'Category',
      'Title',
      'Rating',
      'Status',
      'Tags',
      'Summary',
    ]

    const rows = matches.map((m, idx) => {
      const meta = m.metadata || {}
      return [
        idx + 1,
        m.id,
        (m.score * 100).toFixed(2) + '%',
        m.distance.toFixed(6),
        `"${(meta.category || '').replace(/"/g, '""')}"`,
        `"${(meta.title || '').replace(/"/g, '""')}"`,
        meta.score_rating || '',
        meta.status || '',
        `"${(meta.tags || []).join('; ').replace(/"/g, '""')}"`,
        `"${(meta.summary || '').replace(/"/g, '""')}"`,
      ].join(',')
    })

    const csvContent = [headers.join(','), ...rows].join('\n')
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `hnsw_search_${indexId}_${Date.now()}.csv`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  return (
    <div className="flex items-center gap-2">
      <button
        onClick={handleExportJson}
        className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-slate-900 border border-slate-800 text-slate-300 hover:text-white hover:border-slate-700 text-xs font-mono transition-all"
        title="Download search results as JSON"
      >
        <FileJson className="w-3.5 h-3.5 text-cyan-400" />
        JSON Export
      </button>

      <button
        onClick={handleExportCsv}
        className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-slate-900 border border-slate-800 text-slate-300 hover:text-white hover:border-slate-700 text-xs font-mono transition-all"
        title="Download search results as CSV"
      >
        <FileSpreadsheet className="w-3.5 h-3.5 text-emerald-400" />
        CSV Export
      </button>
    </div>
  )
}
