'use client'

import React, { useState, useEffect } from 'react'
import {
  Search,
  Sparkles,
  Zap,
  Sliders,
  AlertCircle,
  RefreshCw,
  Clock,
  Layers,
  CheckCircle2,
  PlusCircle,
  Database,
} from 'lucide-react'
import Hnsw3dVisualizer from '../components/Hnsw3dVisualizer'
import VectorInput, { generateClusterVector } from '../components/VectorInput'
import FilterBuilder, { FilterState } from '../components/FilterBuilder'
import SearchTimingBreakdown, { SearchTelemetry } from '../components/SearchTimingBreakdown'
import MatchCard, { SearchMatch } from '../components/MatchCard'
import DocumentIngestionModal from '../components/DocumentIngestionModal'
import IndexManagerModal from '../components/IndexManagerModal'
import ExportResults from '../components/ExportResults'

export interface IndexInfo {
  index_id: string
  dimension: number
  total_vectors: number
}

export default function SearchExplorerPage() {
  const [selectedIndex, setSelectedIndex] = useState<string>('default')
  const [indexes, setIndexes] = useState<IndexInfo[]>([
    { index_id: 'default', dimension: 128, total_vectors: 40 },
  ])
  const [currentDimension, setCurrentDimension] = useState<number>(128)

  const [vector, setVector] = useState<number[]>(() => generateClusterVector(1, 0.05))
  const [normalize, setNormalize] = useState(true)
  const [filters, setFilters] = useState<FilterState>({
    category: '',
    minRating: 0,
    status: '',
    k: 6,
    efSearch: 64,
    metric: 'cosine',
    mode: 'auto',
  })

  const [isLoading, setIsLoading] = useState(false)
  const [matches, setMatches] = useState<SearchMatch[]>([])
  const [telemetry, setTelemetry] = useState<SearchTelemetry | null>(null)
  const [selectedMatchId, setSelectedMatchId] = useState<number | null>(null)
  const [errorMsg, setErrorMsg] = useState<string | null>(null)

  const [isIngestModalOpen, setIsIngestModalOpen] = useState(false)
  const [isIndexModalOpen, setIsIndexModalOpen] = useState(false)

  const fetchIndexes = async () => {
    try {
      const res = await fetch('/v1/indexes')
      if (res.ok) {
        const data: IndexInfo[] = await res.json()
        setIndexes(data)
        const match = data.find((i) => i.index_id === selectedIndex)
        if (match) {
          setCurrentDimension(match.dimension)
        }
      }
    } catch (e) {
      // ignore
    }
  }

  const handleIndexChange = (newIndexId: string) => {
    setSelectedIndex(newIndexId)
    const match = indexes.find((i) => i.index_id === newIndexId)
    if (match) {
      setCurrentDimension(match.dimension)
      // Adjust query vector dimension to match index dimension
      if (match.dimension !== vector.length) {
        const adjusted: number[] = []
        for (let d = 0; d < match.dimension; d++) {
          adjusted.push(Number(((Math.random() - 0.5) * 0.2).toFixed(5)))
        }
        setVector(adjusted)
      }
    }
  }

  const executeSearch = async () => {
    setIsLoading(true)
    setErrorMsg(null)

    const payload: any = {
      vector,
      k: filters.k,
      ef_search: filters.efSearch,
      metric: filters.metric,
      mode: filters.mode,
      include_metadata: true,
      normalize_vector: normalize,
    }

    if (filters.category && filters.minRating > 0) {
      payload.filter = {
        $and: [
          { category: filters.category },
          { score_rating: { $gte: filters.minRating } },
        ],
      }
    } else if (filters.category) {
      payload.filter = { category: filters.category }
    } else if (filters.minRating > 0) {
      payload.filter = { score_rating: { $gte: filters.minRating } }
    }

    try {
      const res = await fetch(`/v1/indexes/${selectedIndex}/search`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })

      const data = await res.json()

      if (!res.ok) {
        const msg = data.error?.message || data.error || data.message || `HTTP ${res.status}: Failed search`
        setErrorMsg(msg)
        setMatches([])
        setTelemetry(null)
      } else {
        setMatches(data.matches || [])
        setTelemetry(data.telemetry || null)
        if (data.matches && data.matches.length > 0) {
          setSelectedMatchId(data.matches[0].id)
        }
      }
    } catch (e: any) {
      setErrorMsg(e.message || 'Network error connecting to Axum backend')
    } finally {
      setIsLoading(false)
    }
  }

  useEffect(() => {
    fetchIndexes()
    executeSearch()
  }, [selectedIndex])

  const highlightedIds = matches.map((m) => m.id)

  return (
    <div className="max-w-7xl mx-auto px-4 py-8 space-y-8">
      {/* Top Banner Toolbar */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl sm:text-3xl font-extrabold text-white flex items-center gap-3">
            <Search className="w-7 h-7 text-cyan-400" />
            HNSW Search Explorer
          </h1>
          <p className="text-slate-400 text-xs sm:text-sm mt-1">
            Perform SIMD-accelerated approximate nearest neighbor retrieval with multi-attribute roaring bitmap filtering.
          </p>
        </div>

        {/* Action Buttons: Multi-Index Switcher + Ingest Document + Execute */}
        <div className="flex flex-wrap items-center gap-3">
          {/* Index Switcher Dropdown */}
          <div className="flex items-center gap-2 bg-slate-900 border border-slate-800 rounded-xl px-3 py-2 text-xs font-mono">
            <Database className="w-3.5 h-3.5 text-purple-400" />
            <select
              value={selectedIndex}
              onChange={(e) => handleIndexChange(e.target.value)}
              className="bg-transparent text-white focus:outline-none cursor-pointer"
            >
              {indexes.map((idx) => (
                <option key={idx.index_id} value={idx.index_id} className="bg-slate-900 text-white">
                  {idx.index_id} ({idx.dimension}-D &bull; {idx.total_vectors} docs)
                </option>
              ))}
            </select>
          </div>

          <button
            onClick={() => setIsIngestModalOpen(true)}
            className="flex items-center gap-1.5 px-4 py-2.5 rounded-xl bg-emerald-950/50 border border-emerald-800/80 hover:bg-emerald-900/60 text-emerald-300 font-bold text-xs font-mono transition-all hover:scale-105"
          >
            <PlusCircle className="w-3.5 h-3.5 text-emerald-400" />
            <span>+ Insert Vector</span>
          </button>

          <button
            onClick={executeSearch}
            disabled={isLoading}
            className="flex items-center justify-center gap-2 px-6 py-2.5 rounded-xl bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-400 hover:to-blue-500 text-slate-950 font-bold text-xs font-mono shadow-lg shadow-cyan-500/20 hover:scale-[1.02] transition-all disabled:opacity-50"
          >
            {isLoading ? (
              <>
                <RefreshCw className="w-3.5 h-3.5 animate-spin text-slate-950" />
                <span>Searching…</span>
              </>
            ) : (
              <>
                <Zap className="w-3.5 h-3.5 text-slate-950 fill-slate-950" />
                <span>Search Index</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Error message banner */}
      {errorMsg && (
        <div className="p-4 rounded-xl bg-rose-950/50 border border-rose-800 text-rose-300 flex items-center gap-3 text-xs font-mono">
          <AlertCircle className="w-5 h-5 text-rose-400 shrink-0" />
          <span>{errorMsg}</span>
        </div>
      )}

      {/* Main Grid: Left Controls vs Right 3D Visualizer & Telemetry */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">
        {/* Left Column: Vector and Filter Builder (7 cols) */}
        <div className="lg:col-span-7 space-y-6">
          <VectorInput
            vector={vector}
            onChange={setVector}
            normalize={normalize}
            onToggleNormalize={setNormalize}
          />

          <FilterBuilder filters={filters} onChange={setFilters} />
        </div>

        {/* Right Column: 3D Visualizer & Hardware Telemetry (5 cols) */}
        <div className="lg:col-span-5 space-y-6">
          <div className="h-[360px]">
            <Hnsw3dVisualizer activeId={selectedMatchId} highlightedIds={highlightedIds} />
          </div>

          <div className="glass-panel p-4 rounded-2xl text-xs font-mono space-y-2">
            <div className="text-slate-400 font-semibold text-[11px] uppercase tracking-wider flex justify-between">
              <span>Target Index: {selectedIndex}</span>
              <span className="text-cyan-400 font-bold">{currentDimension}-D</span>
            </div>
            <div className="flex items-center justify-between text-slate-300">
              <span>Filter Pushdown:</span>
              <span className="text-cyan-400 font-bold">
                {filters.category || filters.minRating > 0 ? 'Bitmap Mask Evaluator' : 'Exact Beam Scan'}
              </span>
            </div>
            <div className="flex items-center justify-between text-slate-300">
              <span>Distance Metric:</span>
              <span className="text-purple-400 font-bold uppercase">{filters.metric}</span>
            </div>
            <div className="flex items-center justify-between text-slate-300">
              <span>Exploration Beam:</span>
              <span className="text-emerald-400 font-bold">ef_search = {filters.efSearch}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Telemetry Stage Latency Breakdown */}
      {telemetry && <SearchTimingBreakdown telemetry={telemetry} />}

      {/* Results Section with Export Toolbar */}
      <div className="space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex items-center gap-2">
            <h3 className="text-lg font-bold text-white flex items-center gap-2">
              <span>Nearest Neighbor Matches</span>
              <span className="text-xs font-mono px-2 py-0.5 rounded-full bg-cyan-950 text-cyan-400 border border-cyan-800">
                {matches.length} Results
              </span>
            </h3>
          </div>

          {/* Export Results to JSON / CSV */}
          <ExportResults
            matches={matches}
            telemetry={telemetry}
            indexId={selectedIndex}
          />
        </div>

        {matches.length === 0 ? (
          <div className="glass-panel p-12 text-center text-slate-400 rounded-2xl">
            <AlertCircle className="w-8 h-8 text-amber-400 mx-auto mb-2" />
            <p className="text-sm font-semibold text-slate-300">No matching vectors found</p>
            <p className="text-xs text-slate-500 mt-1">
              Try loosening filter predicate thresholds or selecting a different semantic preset.
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {matches.map((match, idx) => (
              <MatchCard
                key={match.id}
                match={match}
                rank={idx + 1}
                isSelected={selectedMatchId === match.id}
                onSelect={() => setSelectedMatchId(match.id)}
              />
            ))}
          </div>
        )}
      </div>

      {/* Ingestion Modal */}
      <DocumentIngestionModal
        isOpen={isIngestModalOpen}
        onClose={() => setIsIngestModalOpen(false)}
        currentIndex={selectedIndex}
        currentDimension={currentDimension}
        onDocumentInserted={() => {
          fetchIndexes()
          executeSearch()
        }}
      />

      {/* Index Creation Modal */}
      <IndexManagerModal
        isOpen={isIndexModalOpen}
        onClose={() => setIsIndexModalOpen(false)}
        onIndexCreated={(newId) => {
          fetchIndexes()
          handleIndexChange(newId)
        }}
      />
    </div>
  )
}
