// Vidra Editor — main application shell (Tasks 9.1-9.12)
import { useCallback, useEffect, useState, useRef } from 'react';
import { useBackend } from './hooks/useBackend';
import { useProjectStore } from './hooks/useProject';
import { CanvasPanel } from './panels/CanvasPanel';
import { TimelinePanel } from './panels/TimelinePanel';
import { SceneGraphPanel } from './panels/SceneGraphPanel';
import { PropertyPanel } from './panels/PropertyPanel';
import { CodeEditorPanel } from './panels/CodeEditorPanel';
import { Toolbar } from './panels/Toolbar';
import { AiChatPanel } from './panels/AiChatPanel';
import { McpConsolePanel } from './panels/McpConsolePanel';

type Tab = 'scene-graph' | 'code';
type RightTab = 'properties' | 'ai-chat' | 'mcp-console';

function App() {
  const { connected, meta, error, requestFrame, setFrameCallback, rest, setError } = useBackend();
  const { setIr, setSource, setScenes, pushUndo } = useProjectStore();
  const [leftTab, setLeftTab] = useState<Tab>('scene-graph');
  const [rightTab, setRightTab] = useState<RightTab>('properties');

  // Layout state
  const [leftWidth, setLeftWidth] = useState(260);
  const [rightWidth, setRightWidth] = useState(300);
  const isDraggingLeft = useRef(false);
  const isDraggingRight = useRef(false);

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

  // Resizing handlers
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (isDraggingLeft.current) {
        setLeftWidth(Math.max(200, Math.min(e.clientX - 44, 600))); // 44px toolbar offset
      } else if (isDraggingRight.current) {
        setRightWidth(Math.max(250, Math.min(window.innerWidth - e.clientX, 600)));
      }
    };
    const handleMouseUp = () => {
      isDraggingLeft.current = false;
      isDraggingRight.current = false;
      document.body.style.cursor = '';
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, []);

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
        <div className="panel-left" style={{ width: leftWidth }}>
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

        {/* Resizer Left */}
        <div
          className="resizer"
          onMouseDown={() => {
            isDraggingLeft.current = true;
            document.body.style.cursor = 'col-resize';
          }}
        />

        {/* Center */}
        <div className="panel-center">
          <CanvasPanel
            meta={meta}
            requestFrame={requestFrame}
            setFrameCallback={setFrameCallback}
          />
          <TimelinePanel meta={meta} />
        </div>

        {/* Resizer Right */}
        <div
          className="resizer"
          onMouseDown={() => {
            isDraggingRight.current = true;
            document.body.style.cursor = 'col-resize';
          }}
        />

        {/* Right panel */}
        <div className="panel-right" style={{ width: rightWidth }}>
          <div style={{ display: 'flex', borderBottom: '1px solid var(--border-subtle)' }}>
            <button
              className={`toolbar-btn ${rightTab === 'properties' ? 'active' : ''}`}
              style={{ flex: 1, borderRadius: 0, height: '32px', fontSize: '11px' }}
              onClick={() => setRightTab('properties')}
            >Props</button>
            <button
              className={`toolbar-btn ${rightTab === 'ai-chat' ? 'active' : ''}`}
              style={{ flex: 1, borderRadius: 0, height: '32px', fontSize: '11px' }}
              onClick={() => setRightTab('ai-chat')}
            >AI</button>
            <button
              className={`toolbar-btn ${rightTab === 'mcp-console' ? 'active' : ''}`}
              style={{ flex: 1, borderRadius: 0, height: '32px', fontSize: '11px' }}
              onClick={() => setRightTab('mcp-console')}
            >MCP</button>
          </div>

          <div style={{ flex: 1, overflowY: 'hidden', display: 'flex', flexDirection: 'column' }}>
            {rightTab === 'properties' && <PropertyPanel onPatch={handlePatch} />}
            {rightTab === 'ai-chat' && <AiChatPanel rest={rest} />}
            {rightTab === 'mcp-console' && <McpConsolePanel rest={rest} />}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
