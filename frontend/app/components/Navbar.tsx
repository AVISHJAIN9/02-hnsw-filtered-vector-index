'use client'

import React, { useEffect, useState } from 'react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import {
  Layers,
  Activity,
  Search,
  Compass,
  BookOpen,
  ExternalLink,
  Cpu,
  PlusCircle,
  Database,
} from 'lucide-react'
import IndexManagerModal from './IndexManagerModal'

export interface IndexSummary {
  index_id: string
  dimension: number
  total_vectors: number
}

export default function Navbar() {
  const pathname = usePathname()
  const [backendStatus, setBackendStatus] = useState<'checking' | 'healthy' | 'offline'>('checking')
  const [latencyMs, setLatencyMs] = useState<number | null>(null)
  const [indexes, setIndexes] = useState<IndexSummary[]>([])
  const [isIndexModalOpen, setIsIndexModalOpen] = useState(false)

  const fetchIndexes = async () => {
    try {
      const res = await fetch('/v1/indexes')
      if (res.ok) {
        const data = await res.json()
        setIndexes(data)
      }
    } catch (e) {
      // ignore
    }
  }

  const checkHealth = async () => {
    const start = performance.now()
    try {
      const res = await fetch('/v1/health', { cache: 'no-store' })
      const end = performance.now()
      if (res.ok) {
        setBackendStatus('healthy')
        setLatencyMs(Math.round(end - start))
        fetchIndexes()
      } else {
        setBackendStatus('offline')
      }
    } catch (e) {
      setBackendStatus('offline')
    }
  }

  useEffect(() => {
    checkHealth()
    const interval = setInterval(checkHealth, 5000)
    return () => clearInterval(interval)
  }, [])

  const navLinks = [
    { href: '/', label: 'Overview', icon: Compass },
    { href: '/search', label: 'Search Explorer', icon: Search },
    { href: '/dashboard', label: 'Telemetry & Benchmarks', icon: Activity },
    { href: '/ai-insights', label: 'Learned Query Planner', icon: Cpu },
  ]

  return (
    <>
      <header className="sticky top-0 z-50 backdrop-blur-xl bg-slate-950/80 border-b border-slate-800/80">
        <div className="max-w-7xl mx-auto px-4 h-16 flex items-center justify-between">
          <Link href="/" className="flex items-center gap-3 group">
            <div className="w-9 h-9 rounded-xl bg-gradient-to-tr from-cyan-500 to-indigo-600 flex items-center justify-center text-slate-950 shadow-lg shadow-cyan-500/20 group-hover:scale-105 transition-transform">
              <Layers className="w-5 h-5 text-slate-950 font-bold" />
            </div>
            <div>
              <div className="font-extrabold text-sm tracking-wide text-white flex items-center gap-1.5">
                HNSW<span className="text-cyan-400">INDEX</span>
                <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-cyan-950 text-cyan-400 border border-cyan-800">
                  SIMD Engine
                </span>
              </div>
              <div className="text-[10px] text-slate-400 font-mono">
                {indexes.length} Active {indexes.length === 1 ? 'Index' : 'Indices'}
              </div>
            </div>
          </Link>

          {/* Navigation links */}
          <nav className="hidden md:flex items-center space-x-1">
            {navLinks.map((link) => {
              const Icon = link.icon
              const isActive = pathname === link.href
              return (
                <Link
                  key={link.href}
                  href={link.href}
                  className={`flex items-center gap-2 px-3.5 py-2 rounded-xl text-xs font-medium transition-all ${
                    isActive
                      ? 'bg-cyan-500/15 text-cyan-300 border border-cyan-500/30 font-semibold'
                      : 'text-slate-300 hover:text-white hover:bg-slate-900/60'
                  }`}
                >
                  <Icon className={`w-3.5 h-3.5 ${isActive ? 'text-cyan-400' : 'text-slate-400'}`} />
                  {link.label}
                </Link>
              )
            })}
          </nav>

          {/* Actions & Live Backend Status */}
          <div className="flex items-center gap-3">
            <button
              onClick={() => setIsIndexModalOpen(true)}
              className="hidden sm:flex items-center gap-1.5 text-xs font-mono text-purple-300 hover:text-purple-200 px-3 py-1.5 rounded-xl bg-purple-950/40 border border-purple-800/60 transition-all hover:scale-105"
            >
              <Database className="w-3.5 h-3.5 text-purple-400" />
              <span>+ Index</span>
            </button>

            <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-slate-900/90 border border-slate-800 text-xs font-mono">
              <span
                className={`w-2 h-2 rounded-full ${
                  backendStatus === 'healthy'
                    ? 'bg-emerald-400 shadow-[0_0_8px_#34d399]'
                    : backendStatus === 'checking'
                    ? 'bg-amber-400 animate-ping'
                    : 'bg-rose-500'
                }`}
              />
              <span className="text-slate-300">
                {backendStatus === 'healthy'
                  ? `Axum :8080 (${latencyMs}ms)`
                  : backendStatus === 'checking'
                  ? 'Connecting…'
                  : 'Offline'}
              </span>
            </div>

            <a
              href="/v1/openapi.json"
              target="_blank"
              rel="noreferrer"
              className="hidden lg:flex items-center gap-1 text-xs font-mono text-cyan-400 hover:text-cyan-300 px-3 py-1.5 rounded-lg bg-slate-900 border border-slate-800"
            >
              <BookOpen className="w-3.5 h-3.5" />
              <span>OpenAPI</span>
              <ExternalLink className="w-2.5 h-2.5 opacity-70" />
            </a>
          </div>
        </div>
      </header>

      <IndexManagerModal
        isOpen={isIndexModalOpen}
        onClose={() => setIsIndexModalOpen(false)}
        onIndexCreated={() => fetchIndexes()}
      />
    </>
  )
}
