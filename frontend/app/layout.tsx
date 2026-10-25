import './globals.css'
import type { Metadata } from 'next'
import Navbar from './components/Navbar'

export const metadata: Metadata = {
  title: 'HNSW Filtered Vector Index | SIMD Acceleration & 3D Visualizer',
  description: 'Industrial-grade vector search engine with multi-layer HNSW graphs, Int8 quantization, and learned filter query planning.',
}

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <body className="bg-slate-950 text-slate-100 min-h-screen antialiased flex flex-col font-sans cyber-grid selection:bg-cyan-500 selection:text-slate-950">
        <Navbar />
        <main className="flex-1 relative z-10">{children}</main>
        <footer className="border-t border-slate-800/80 bg-slate-950/60 backdrop-blur-md py-6 mt-12 text-center text-xs font-mono text-slate-500">
          <div className="max-w-7xl mx-auto px-4 flex flex-col sm:flex-row items-center justify-between gap-3">
            <div>
              HNSW Filtered Vector Index • High-Performance Vector Database Architecture
            </div>
            <div className="flex items-center gap-4 text-slate-400">
              <span>AVX-512 / AVX2 / NEON</span>
              <span>•</span>
              <span>Int8 Quantization</span>
              <span>•</span>
              <span>Roaring Bitmap AST</span>
            </div>
          </div>
        </footer>
      </body>
    </html>
  )
}
