import { useState, useCallback, useRef, useEffect } from 'react'
import { OfficeState } from './office/engine/officeState.js'
import { OfficeCanvas } from './office/components/OfficeCanvas.js'
import { ToolOverlay } from './office/components/ToolOverlay.js'
import { EditorState } from './office/editor/editorState.js'
import { useAgentEvents } from './hooks/useAgentEvents.js'
import { loadAllAssets } from './office/assetLoader.js'
import { migrateLayoutColors } from './office/layout/layoutSerializer.js'
import { PULSE_ANIMATION_DURATION_SEC, ZOOM_DEFAULT_DPR_FACTOR } from './office/constants.js'
import { startWatching } from './bridge.js'

// Game state lives outside React — updated imperatively by event handlers
const officeStateRef = { current: null as OfficeState | null }
const editorState = new EditorState()

function getOfficeState(): OfficeState {
  if (!officeStateRef.current) {
    officeStateRef.current = new OfficeState()
  }
  return officeStateRef.current
}

function App() {
  const [layoutReady, setLayoutReady] = useState(false)
  const [zoom, setZoom] = useState(Math.round(window.devicePixelRatio || 1) * ZOOM_DEFAULT_DPR_FACTOR)
  const panRef = useRef({ x: 0, y: 0 })
  const containerRef = useRef<HTMLDivElement>(null)

  const { agents, agentTools, subagentCharacters } = useAgentEvents(getOfficeState)

  // Load assets + default layout on mount
  useEffect(() => {
    let cancelled = false
    ;(async () => {
      const layout = await loadAllAssets()
      if (cancelled) return
      if (layout) {
        const migrated = migrateLayoutColors(layout)
        // Recreate OfficeState with the loaded layout
        officeStateRef.current = new OfficeState(migrated)
      }
      setLayoutReady(true)

      // Auto-start watching the current project (try to detect from env)
      try {
        await startWatching('')
      } catch {
        // Backend may not be ready yet — that's OK for dev
        console.warn('[App] Could not start watching — backend not available')
      }
    })()
    return () => { cancelled = true }
  }, [])

  const handleClick = useCallback((_agentId: number) => {
    // In standalone mode, clicking an agent just toggles selection
    // (selection/follow handled in OfficeCanvas)
  }, [])

  const handleCloseAgent = useCallback((_id: number) => {
    // No-op in standalone — can't close terminals
  }, [])

  const handleZoomChange = useCallback((newZoom: number) => {
    setZoom(newZoom)
  }, [])

  // Editor no-ops for MVP (edit mode disabled)
  const noopTile = useCallback((_col: number, _row: number) => {}, [])
  const noopSelection = useCallback(() => {}, [])
  const noopDelete = useCallback(() => {}, [])
  const noopRotate = useCallback(() => {}, [])
  const noopDrag = useCallback((_uid: string, _col: number, _row: number) => {}, [])

  const officeState = getOfficeState()

  if (!layoutReady) {
    return (
      <div style={{
        width: '100%',
        height: '100%',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        color: 'var(--vscode-foreground)',
        background: 'var(--pixel-bg)',
        fontSize: '24px',
      }}>
        Loading...
      </div>
    )
  }

  return (
    <div ref={containerRef} style={{ width: '100%', height: '100%', position: 'relative', overflow: 'hidden' }}>
      <style>{`
        @keyframes pixel-agents-pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.3; }
        }
        .pixel-agents-pulse { animation: pixel-agents-pulse ${PULSE_ANIMATION_DURATION_SEC}s ease-in-out infinite; }
      `}</style>

      <OfficeCanvas
        officeState={officeState}
        onClick={handleClick}
        isEditMode={false}
        editorState={editorState}
        onEditorTileAction={noopTile}
        onEditorEraseAction={noopTile}
        onEditorSelectionChange={noopSelection}
        onDeleteSelected={noopDelete}
        onRotateSelected={noopRotate}
        onDragMove={noopDrag}
        editorTick={0}
        zoom={zoom}
        onZoomChange={handleZoomChange}
        panRef={panRef}
      />

      {/* Vignette overlay */}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          background: 'var(--pixel-vignette)',
          pointerEvents: 'none',
          zIndex: 40,
        }}
      />

      <ToolOverlay
        officeState={officeState}
        agents={agents}
        agentTools={agentTools}
        subagentCharacters={subagentCharacters}
        containerRef={containerRef}
        zoom={zoom}
        panRef={panRef}
        onCloseAgent={handleCloseAgent}
      />
    </div>
  )
}

export default App
