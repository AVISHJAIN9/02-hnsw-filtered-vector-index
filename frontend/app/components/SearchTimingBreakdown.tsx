'use client'

import React from 'react'
import { Zap, Clock, ShieldCheck, Database, Layers } from 'lucide-react'

export interface SearchTelemetry {
  took_ms: number
  candidates_scanned: number
  candidates_returned: number
  strategy_used: string
  selectivity: number
  timings_ns: {
    parse_ns: number
    filter_eval_ns: number
    retrieval_ns: number
    hydration_ns: number
    assembly_ns: number
    total_ns: number
  }
}

interface Props {
  telemetry: SearchTelemetry | null
}

export default function SearchTimingBreakdown({ telemetry }: Props) {
  if (!telemetry) return null

  const { timings_ns, took_ms, candidates_scanned, candidates_returned, strategy_used, selectivity } =
    telemetry
  const totalNs = timings_ns.total_ns || 1

  const stages = [
    {
      name: 'Query Validation & Parse',
      ns: timings_ns.parse_ns,
      color: 'bg-amber-400',
      textColor: 'text-amber-400',
      icon: Clock,
    },
    {
      name: 'Bitmap Filter Eval',
      ns: timings_ns.filter_eval_ns,
      color: 'bg-emerald-400',
      textColor: 'text-emerald-400',
      icon: ShieldCheck,
    },
    {
      name: 'HNSW Graph Vector Retrieval',
      ns: timings_ns.retrieval_ns,
      color: 'bg-cyan-400',
      textColor: 'text-cyan-400',
      icon: Zap,
    },
    {
      name: 'Metadata Payload Hydration',
      ns: timings_ns.hydration_ns,
      color: 'bg-purple-400',
      textColor: 'text-purple-400',
      icon: Database,
    },
  ]

  const formatTime = (ns: number) => {
    if (ns < 1000) return `${ns} ns`
    if (ns < 1_000_000) return `${(ns / 1000).toFixed(2)} µs`
    return `${(ns / 1_000_000).toFixed(3)} ms`
  }

  return (
    <div className="glass-panel p-5 rounded-2xl space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2 pb-3 border-b border-slate-800">
        <div className="flex items-center gap-2">
          <Zap className="w-4 h-4 text-cyan-400" />
          <h3 className="font-semibold text-white text-sm">Hardware Telemetry & Kernel Profiler</h3>
        </div>

        <div className="flex flex-wrap items-center gap-2 text-xs font-mono">
          <span className="px-2.5 py-1 rounded-lg bg-cyan-950/60 text-cyan-400 border border-cyan-800/60">
            Total Latency: {took_ms.toFixed(3)} ms ({formatTime(timings_ns.total_ns)})
          </span>
          <span className="px-2.5 py-1 rounded-lg bg-emerald-950/60 text-emerald-400 border border-emerald-800/60 uppercase">
            Strategy: {strategy_used.replace(/_/g, ' ')}
          </span>
          <span className="px-2.5 py-1 rounded-lg bg-purple-950/60 text-purple-400 border border-purple-800/60">
            Selectivity: {(selectivity * 100).toFixed(1)}% ({candidates_returned}/{candidates_scanned})
          </span>
        </div>
      </div>

      {/* Multi-Segment Execution Progress Bar */}
      <div className="space-y-1.5">
        <div className="flex justify-between text-xs font-mono text-slate-400">
          <span>Stage Latency Breakdown (% of Total Execution Time)</span>
          <span className="text-slate-200">{formatTime(timings_ns.total_ns)}</span>
        </div>
        <div className="h-3 w-full bg-slate-900 rounded-full overflow-hidden flex">
          {stages.map((stage) => {
            const pct = Math.max(1, (stage.ns / totalNs) * 100)
            return (
              <div
                key={stage.name}
                style={{ width: `${pct}%` }}
                title={`${stage.name}: ${formatTime(stage.ns)} (${pct.toFixed(1)}%)`}
                className={`${stage.color} transition-all duration-500`}
              />
            )
          })}
        </div>
      </div>

      {/* Stage Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        {stages.map((s) => {
          const Icon = s.icon
          const pct = ((s.ns / totalNs) * 100).toFixed(1)
          return (
            <div
              key={s.name}
              className="p-3 rounded-xl bg-slate-900/60 border border-slate-800/80 flex flex-col justify-between"
            >
              <div className="flex items-center gap-1.5 text-xs text-slate-400 mb-1">
                <Icon className={`w-3.5 h-3.5 ${s.textColor}`} />
                <span className="truncate">{s.name}</span>
              </div>
              <div className="flex items-baseline justify-between mt-1">
                <span className="text-sm font-bold font-mono text-white">{formatTime(s.ns)}</span>
                <span className={`text-[10px] font-mono ${s.textColor}`}>{pct}%</span>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
