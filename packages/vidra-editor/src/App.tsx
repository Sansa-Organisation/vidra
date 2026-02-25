// Vidra Editor — main application shell (Tasks 9.1-9.12)
import { useCallback, useEffect, useState } from 'react';
import { useBackend } from './hooks/useBackend';
import { useProjectStore } from './hooks/useProject';
import { CanvasPanel } from './panels/CanvasPanel';
import { TimelinePanel } from './panels/TimelinePanel';
import { SceneGraphPanel } from './panels/SceneGraphPanel';
import { PropertyPanel } from './panels/PropertyPanel';
import { CodeEditorPanel } from './panels/CodeEditorPanel';
import { Toolbar } from './panels/Toolbar';

type Tab = 'scene-graph' | 'code';

function App() {
  const { connected, meta, error, requestFrame, setFrameCallback, rest, setError } = useBackend();
  const { setIr, setSource, setScenes, pushUndo } = useProjectStore();
  const [leftTab, setLeftTab] = useState<Tab>('scene-graph');

  // Load project on connect
  useEffect(() => {
    if (!connected) return;

    rest('GET', '/api/project').then((res: unknown) => {
      const data = res as { ir?: string; error?: string };
      if (data.ir) {
        setIr(data.ir);
        // Parse scenes from IR
        try {
          const proj = JSON.parse(data.ir);
          if (proj.scenes) {
            setScenes(proj.scenes.map((s: { id: string; layers?: { id: string; content?: { type: string } }[] }) => ({
              id: s.id,
              name: s.id,
              duration_frames: 150,
              layers: (s.layers || []).map((l: { id: string; content?: { type: string } }) => ({
                id: l.id,
                content_type: l.content?.type || 'unknown',
                label: l.id,
              })),
            })));
          }
        } catch { /* ignore parse errors */ }
      }
    });

    rest('GET', '/api/project/source').then((res: unknown) => {
      const data = res as { source?: string };
      if (data.source) setSource(data.source);
    });
  }, [connected, rest, setIr, setSource, setScenes]);

  const handlePatch = useCallback(async (layerId: string, props: Record<string, unknown>) => {
    pushUndo();
    await rest('POST', '/api/project/patch', { layer_id: layerId, properties: props });
  }, [rest, pushUndo]);

  const handleSaveSource = useCallback(async (source: string) => {
    pushUndo();
    await rest('PUT', '/api/project/source', { source });
  }, [rest, pushUndo]);

  const handleExport = useCallback(async () => {
    await rest('POST', '/api/render/export');
  }, [rest]);

  return (
    <div className="editor-shell">
      {/* Error bar */}
      {error && (
        <div className="error-bar">
          <span>⚠ {error}</span>
          <button className="dismiss" onClick={() => setError(null)}>✕</button>
        </div>
      )}

      {/* Header */}
      <header className="editor-header">
        <span className="logo">▸ Vidra Editor</span>
        <div className="spacer" />
        <span className={`status-dot ${connected ? 'connected' : ''}`} />
        <span className="status-label">{connected ? 'Connected' : 'Disconnected'}</span>
      </header>

      {/* Body */}
      <div className="editor-body">
        <Toolbar onExport={handleExport} rest={rest} />

        {/* Left panel */}
        <div className="panel-left">
          <div style={{ display: 'flex', borderBottom: '1px solid var(--border-subtle)' }}>
            <button
              className={`toolbar-btn ${leftTab === 'scene-graph' ? 'active' : ''}`}
              style={{ flex: 1, borderRadius: 0, height: '32px', fontSize: '11px' }}
              onClick={() => setLeftTab('scene-graph')}
            >Scenes</button>
            <button
              className={`toolbar-btn ${leftTab === 'code' ? 'active' : ''}`}
              style={{ flex: 1, borderRadius: 0, height: '32px', fontSize: '11px' }}
              onClick={() => setLeftTab('code')}
            >Code</button>
          </div>
          {leftTab === 'scene-graph' ? (
            <SceneGraphPanel />
          ) : (
            <CodeEditorPanel onSave={handleSaveSource} />
          )}
        </div>

        {/* Center */}
        <div className="panel-center">
          <CanvasPanel
            meta={meta}
            requestFrame={requestFrame}
            setFrameCallback={setFrameCallback}
          />
          <TimelinePanel meta={meta} />
        </div>

        {/* Right panel */}
        <div className="panel-right">
          <div className="panel-title">Properties</div>
          <PropertyPanel onPatch={handlePatch} />
        </div>
      </div>
    </div>
  );
}

export default App;
