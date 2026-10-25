'use client'

import React, { useEffect, useRef, useState } from 'react'

interface NodePoint {
  x: number
  y: number
  z: number
  layer: number
  id: number
  category: string
  active?: boolean
  highlighted?: boolean
}

export default function Hnsw3dVisualizer({
  activeId,
  highlightedIds = [],
}: {
  activeId?: number | null
  highlightedIds?: number[]
}) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null)
  const [rotation, setRotation] = useState({ x: 0.35, y: -0.4 })
  const isDragging = useRef(false)
  const lastMouse = useRef({ x: 0, y: 0 })
  const animationFrameId = useRef<number>()

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    // Generate multi-layer HNSW graph nodes
    const nodes: NodePoint[] = []
    const layerCount = 3
    const categoryColors: Record<string, string> = {
      cybersecurity: '#f43f5e',
      nlp_research: '#38bdf8',
      financial_tech: '#10b981',
      e_commerce: '#f59e0b',
      biomedical: '#a855f7',
    }

    const categories = ['cybersecurity', 'nlp_research', 'financial_tech', 'e_commerce', 'biomedical']

    // Layer 0: Ground level dense nodes (60 nodes)
    for (let i = 0; i < 60; i++) {
      const angle = (i / 60) * Math.PI * 2
      const radius = 130 + (i % 7) * 15
      const cat = categories[i % categories.length]
      nodes.push({
        id: i + 1,
        x: Math.cos(angle) * radius + (Math.sin(i * 3) * 20),
        y: -60,
        z: Math.sin(angle) * radius + (Math.cos(i * 3) * 20),
        layer: 0,
        category: cat,
      })
    }

    // Layer 1: Medium skip-list nodes (20 nodes)
    for (let i = 0; i < 20; i++) {
      const angle = (i / 20) * Math.PI * 2
      const radius = 80 + (i % 4) * 12
      const cat = categories[i % categories.length]
      nodes.push({
        id: 100 + i,
        x: Math.cos(angle) * radius,
        y: 10,
        z: Math.sin(angle) * radius,
        layer: 1,
        category: cat,
      })
    }

    // Layer 2: Top sparse entry-point nodes (5 nodes)
    for (let i = 0; i < 5; i++) {
      const angle = (i / 5) * Math.PI * 2
      const radius = 40
      const cat = categories[i % categories.length]
      nodes.push({
        id: 200 + i,
        x: Math.cos(angle) * radius,
        y: 80,
        z: Math.sin(angle) * radius,
        layer: 2,
        category: cat,
      })
    }

    let autoAngle = 0

    const render = () => {
      if (!canvas) return
      const width = (canvas.width = canvas.parentElement?.clientWidth || 700)
      const height = (canvas.height = canvas.parentElement?.clientHeight || 450)

      ctx.clearRect(0, 0, width, height)

      autoAngle += 0.003
      const rotY = rotation.y + autoAngle
      const rotX = rotation.x

      const cosY = Math.cos(rotY)
      const sinY = Math.sin(rotY)
      const cosX = Math.cos(rotX)
      const sinX = Math.sin(rotX)

      const fov = 420
      const centerX = width / 2
      const centerY = height / 2

      // Project 3D to 2D
      const projected = nodes.map((node) => {
        // Rotate around Y
        const x1 = node.x * cosY - node.z * sinY
        const z1 = node.z * cosY + node.x * sinY

        // Rotate around X
        const y2 = node.y * cosX - z1 * sinX
        const z2 = z1 * cosX + node.y * sinX

        const scale = fov / (fov + z2 + 250)
        const px = centerX + x1 * scale
        const py = centerY - y2 * scale

        const isMatch = highlightedIds.includes(node.id) || (node.id === activeId)

        return {
          ...node,
          px,
          py,
          scale,
          z2,
          isMatch,
        }
      })

      // Sort by depth for correct painter's algorithm
      projected.sort((a, b) => b.z2 - a.z2)

      // Draw layer planes
      ;[-60, 10, 80].forEach((layerY, idx) => {
        ctx.strokeStyle = idx === 0 ? 'rgba(56, 189, 248, 0.08)' : idx === 1 ? 'rgba(168, 85, 247, 0.08)' : 'rgba(244, 63, 94, 0.1)'
        ctx.lineWidth = 1
        ctx.beginPath()

        const planeSize = 160 - idx * 30
        const corners = [
          { x: -planeSize, y: layerY, z: -planeSize },
          { x: planeSize, y: layerY, z: -planeSize },
          { x: planeSize, y: layerY, z: planeSize },
          { x: -planeSize, y: layerY, z: planeSize },
        ]

        corners.forEach((c, cIdx) => {
          const x1 = c.x * cosY - c.z * sinY
          const z1 = c.z * cosY + c.x * sinY
          const y2 = c.y * cosX - z1 * sinX
          const z2 = z1 * cosX + c.y * sinX
          const scale = fov / (fov + z2 + 250)
          const px = centerX + x1 * scale
          const py = centerY - y2 * scale
          if (cIdx === 0) ctx.moveTo(px, py)
          else ctx.lineTo(px, py)
        })
        ctx.closePath()
        ctx.stroke()
      })

      // Draw inter-node edges (HNSW Proximity Delaunay Graphs)
      ctx.lineWidth = 0.8
      for (let i = 0; i < projected.length; i++) {
        const a = projected[i]
        for (let j = i + 1; j < projected.length; j++) {
          const b = projected[j]
          // Within same layer proximity OR inter-layer hierarchy links
          const distSq = (a.x - b.x) ** 2 + (a.y - b.y) ** 2 + (a.z - b.z) ** 2
          const isInterLayer = Math.abs(a.layer - b.layer) === 1 && distSq < 7000
          const isIntraLayer = a.layer === b.layer && distSq < (a.layer === 0 ? 3200 : a.layer === 1 ? 5500 : 9000)

          if (isIntraLayer || isInterLayer) {
            ctx.beginPath()
            ctx.moveTo(a.px, a.py)
            ctx.lineTo(b.px, b.py)

            if (a.isMatch && b.isMatch) {
              ctx.strokeStyle = 'rgba(56, 189, 248, 0.8)'
              ctx.lineWidth = 2
            } else if (isInterLayer) {
              ctx.strokeStyle = 'rgba(168, 85, 247, 0.15)'
              ctx.lineWidth = 0.7
            } else {
              ctx.strokeStyle = 'rgba(148, 163, 184, 0.08)'
              ctx.lineWidth = 0.5
            }
            ctx.stroke()
          }
        }
      }

      // Draw Nodes
      projected.forEach((node) => {
        const color = categoryColors[node.category] || '#38bdf8'
        const radius = (node.isMatch ? 7 : node.layer === 2 ? 5 : node.layer === 1 ? 3.5 : 2.5) * node.scale

        ctx.beginPath()
        ctx.arc(node.px, node.py, radius, 0, Math.PI * 2)

        if (node.isMatch) {
          // Glowing search hit halo
          ctx.fillStyle = '#38bdf8'
          ctx.shadowColor = '#38bdf8'
          ctx.shadowBlur = 15
          ctx.fill()
          ctx.shadowBlur = 0

          ctx.strokeStyle = '#ffffff'
          ctx.lineWidth = 2
          ctx.stroke()
        } else {
          ctx.fillStyle = color
          ctx.globalAlpha = 0.7 * Math.min(1.2, node.scale)
          ctx.fill()
          ctx.globalAlpha = 1.0
        }
      })

      animationFrameId.current = requestAnimationFrame(render)
    }

    render()

    const handleMouseDown = (e: MouseEvent) => {
      isDragging.current = true
      lastMouse.current = { x: e.clientX, y: e.clientY }
    }

    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging.current) return
      const dx = e.clientX - lastMouse.current.x
      const dy = e.clientY - lastMouse.current.y
      setRotation((prev) => ({
        x: Math.max(-1.2, Math.min(1.2, prev.x + dy * 0.005)),
        y: prev.y + dx * 0.005,
      }))
      lastMouse.current = { x: e.clientX, y: e.clientY }
    }

    const handleMouseUp = () => {
      isDragging.current = false
    }

    canvas.addEventListener('mousedown', handleMouseDown)
    window.addEventListener('mousemove', handleMouseMove)
    window.addEventListener('mouseup', handleMouseUp)

    return () => {
      if (animationFrameId.current) cancelAnimationFrame(animationFrameId.current)
      canvas.removeEventListener('mousedown', handleMouseDown)
      window.removeEventListener('mousemove', handleMouseMove)
      window.removeEventListener('mouseup', handleMouseUp)
    }
  }, [rotation, activeId, highlightedIds])

  return (
    <div className="relative w-full h-full min-h-[380px] flex items-center justify-center overflow-hidden rounded-2xl bg-slate-950/70 border border-slate-800/80">
      <div className="absolute top-3 left-4 z-10 flex items-center gap-2 text-xs font-mono text-slate-400 bg-slate-900/80 px-3 py-1.5 rounded-lg border border-slate-800 backdrop-blur-md">
        <span className="w-2 h-2 rounded-full bg-cyan-400 animate-pulse" />
        <span>HNSW Multi-Layer Graph (L0-L2) • Drag to Orbit</span>
      </div>

      <div className="absolute bottom-3 right-4 z-10 flex flex-wrap gap-2 text-[11px] font-mono text-slate-400 bg-slate-900/80 px-3 py-1.5 rounded-lg border border-slate-800 backdrop-blur-md">
        <span className="flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-[#f43f5e]" /> Cyber</span>
        <span className="flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-[#38bdf8]" /> NLP</span>
        <span className="flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-[#10b981]" /> FinTech</span>
        <span className="flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-[#f59e0b]" /> E-Com</span>
        <span className="flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-[#a855f7]" /> Biotech</span>
      </div>

      <canvas
        ref={canvasRef}
        className="w-full h-full cursor-grab active:cursor-grabbing"
      />
    </div>
  )
}
