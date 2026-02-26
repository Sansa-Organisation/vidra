// AI Chat panel (Task 10.1 & 10.3)
import { useState } from 'react';
import { useAI } from '../hooks/useAI';
import { useMcp } from '../hooks/useMcp';

interface Props {
    rest: (method: string, path: string, body?: unknown) => Promise<unknown>;
}

export function AiChatPanel({ rest }: Props) {
    const { messages, loading, error, sendMessage, clearChat } = useAI(rest);
    const { invoke, loading: mcpLoading } = useMcp(rest);
    const [input, setInput] = useState('');

    const handleSend = () => {
        if (!input.trim()) return;
        sendMessage(input);
        setInput('');
    };

    const attemptToParseToolCall = (content: string) => {
        try {
            // Find a JSON block in the AI response
            const match = content.match(/\{[\s\S]*"name"[\s\S]*"arguments"[\s\S]*\}/);
            if (match) {
                const parsed = JSON.parse(match[0]);
                if (parsed.name && parsed.arguments) {
                    return parsed;
                }
            }
        } catch {
            return null;
        }
        return null;
    };

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
            <div style={{ padding: '8px 12px', borderBottom: '1px solid var(--border-subtle)', display: 'flex', justifyContent: 'flex-end' }}>
                <button className="btn btn-outline" style={{ fontSize: '10px', padding: '2px 8px' }} onClick={clearChat}>
                    Clear Chat
                </button>
            </div>

            <div style={{ flex: 1, overflowY: 'auto', padding: '12px', display: 'flex', flexDirection: 'column', gap: '12px' }}>
                {messages.length === 0 && (
                    <div style={{ color: 'var(--text-muted)', fontSize: '12px', textAlign: 'center', marginTop: '40px' }}>
                        I'm the Vidra Assistant. Ask me to generate a web scene or modify layers!
                    </div>
                )}

                {messages.map((m, i) => {
                    const toolCall = m.role === 'assistant' ? attemptToParseToolCall(m.content) : null;

                    return (
                        <div key={i} style={{
                            background: m.role === 'user' ? 'rgba(88,166,255,0.1)' : 'var(--bg-elevated)',
                            padding: '10px 12px',
                            borderRadius: '8px',
                            fontSize: '13px',
                            color: m.role === 'user' ? 'var(--accent-blue)' : 'var(--text-primary)',
                            alignSelf: m.role === 'user' ? 'flex-end' : 'flex-start',
                            maxWidth: '92%',
                            wordWrap: 'break-word',
                            whiteSpace: 'pre-wrap',
                            border: m.role === 'user' ? '1px solid rgba(88,166,255,0.3)' : '1px solid var(--border-subtle)'
                        }}>
                            {m.content}

                            {toolCall && (
                                <div style={{ marginTop: '10px', paddingTop: '10px', borderTop: '1px solid var(--border-default)' }}>
                                    <span style={{ fontSize: '11px', color: 'var(--text-muted)', display: 'block', marginBottom: '6px' }}>
                                        Tool: {toolCall.name}
                                    </span>
                                    <button
                                        className="btn btn-primary"
                                        style={{ width: '100%', fontSize: '12px', padding: '6px' }}
                                        onClick={() => invoke(toolCall.name, toolCall.arguments)}
                                        disabled={mcpLoading}
                                    >
                                        {mcpLoading ? 'Applying...' : 'Apply Tool Call ⚡'}
                                    </button>
                                </div>
                            )}
                        </div>
                    );
                })}
                {loading && <div style={{ color: 'var(--text-muted)', fontSize: '12px', alignSelf: 'flex-start' }}>Agent is thinking...</div>}
                {error && <div style={{ color: 'var(--accent-red)', fontSize: '12px', background: 'rgba(248,81,73,0.1)', padding: '8px', borderRadius: '4px' }}>Error: {error}</div>}
            </div>

            <div style={{ padding: '12px', borderTop: '1px solid var(--border-subtle)', background: 'var(--bg-secondary)' }}>
                <div style={{ display: 'flex', gap: '8px' }}>
                    <input
                        className="prop-input"
                        value={input}
                        onChange={e => setInput(e.target.value)}
                        onKeyDown={e => {
                            if (e.key === 'Enter') handleSend();
                        }}
                        placeholder="e.g. Add a text layer saying Hello"
                        disabled={loading}
                        style={{ padding: '8px 12px', fontSize: '13px' }}
                    />
                    <button
                        className="btn btn-primary"
                        onClick={handleSend}
                        disabled={loading || !input.trim()}
                        style={{ padding: '0 16px' }}
                    >
                        Send
                    </button>
                </div>
            </div>
        </div>
    );
}
