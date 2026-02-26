// MCP Console panel (Task 10.4 & 10.5)
import { useState } from 'react';
import { useMcp } from '../hooks/useMcp';

interface Props {
    rest: (method: string, path: string, body?: unknown) => Promise<unknown>;
}

export function McpConsolePanel({ rest }: Props) {
    const { invoke, loading, error, result } = useMcp(rest);
    const [toolName, setToolName] = useState('vidra-add_scene');
    const [toolArgs, setToolArgs] = useState('{\n  "name": "New Scene"\n}');

    const handleInvoke = () => {
        try {
            const args = JSON.parse(toolArgs);
            invoke(toolName, args);
        } catch (e) {
            alert('Invalid JSON arguments');
        }
    };

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%', padding: '16px', gap: '16px' }}>
            <div className="prop-group">
                <div className="prop-group-title">MCP Console</div>
                <div style={{ fontSize: '12px', color: 'var(--text-muted)', marginBottom: '16px' }}>
                    Manually invoke registered MCP tools against the active project context.
                    Changes apply immediately via WebSocket.
                </div>

                <div className="prop-row" style={{ flexDirection: 'column', alignItems: 'stretch', gap: '6px' }}>
                    <span className="prop-label" style={{ fontWeight: 600 }}>Tool Name</span>
                    <select
                        className="prop-input"
                        value={toolName}
                        onChange={e => setToolName(e.target.value)}
                        style={{ padding: '8px', fontSize: '13px' }}
                    >
                        <option value="vidra-add_scene">vidra-add_scene</option>
                        <option value="vidra-add_layer">vidra-add_layer</option>
                        <option value="vidra-update_layer">vidra-update_layer</option>
                        <option value="vidra-generate_web_code">vidra-generate_web_code</option>
                    </select>
                </div>

                <div className="prop-row" style={{ flexDirection: 'column', alignItems: 'stretch', gap: '6px', marginTop: '12px' }}>
                    <span className="prop-label" style={{ fontWeight: 600 }}>Arguments (JSON)</span>
                    <textarea
                        className="prop-input"
                        style={{ height: '120px', fontFamily: 'monospace', resize: 'vertical', padding: '10px' }}
                        value={toolArgs}
                        onChange={e => setToolArgs(e.target.value)}
                    />
                </div>
            </div>

            <button
                className="btn btn-primary"
                onClick={handleInvoke}
                disabled={loading}
                style={{ padding: '10px', fontSize: '14px', alignSelf: 'flex-start' }}
            >
                {loading ? 'Invoking...' : 'Invoke Tool'}
            </button>

            <div className="prop-group" style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
                <div className="prop-group-title" style={{ marginTop: '8px' }}>Result</div>
                <textarea
                    className="prop-input"
                    style={{
                        flex: 1,
                        fontFamily: 'monospace',
                        fontSize: '11px',
                        resize: 'none',
                        background: 'var(--bg-tertiary)',
                        color: error ? 'var(--accent-red)' : 'var(--text-secondary)'
                    }}
                    readOnly
                    value={error ? `Error: ${error}` : (result || 'No output yet.')}
                />
            </div>
        </div>
    );
}
