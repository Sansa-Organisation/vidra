// Code Editor panel — VidraScript source editor (Task 9.8)
import { useState, useEffect, useCallback } from 'react';
import { useProjectStore } from '../hooks/useProject';
import Editor, { useMonaco } from '@monaco-editor/react';

interface Props {
    onSave: (source: string) => void;
}

export function CodeEditorPanel({ onSave }: Props) {
    const source = useProjectStore(s => s.source);
    const [localSource, setLocalSource] = useState(source || '');
    const monaco = useMonaco();

    useEffect(() => {
        if (source !== null && source !== localSource) {
            setLocalSource(source);
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [source]);

    useEffect(() => {
        if (monaco) {
            // Register incredibly basic VidraScript syntax highlighting
            monaco.languages.register({ id: 'vidra' });
            monaco.languages.setMonarchTokensProvider('vidra', {
                tokenizer: {
                    root: [
                        [/project|scene|layer|content|web|text|solid/, 'keyword'],
                        [/([a-z_][a-zA-Z0-9_]*)(?=\s*:)/, 'type'],
                        [/"([^"\\]|\\.)*"/, 'string'],
                        [/\/\/.*$/, 'comment'],
                        [/#([0-9a-fA-F]{3,8})/, 'number.hex'],
                        [/\d+(px|em|rem|vw|vh|s|ms|fps)?/, 'number'],
                    ]
                }
            });
            monaco.editor.defineTheme('vidra-dark', {
                base: 'vs-dark',
                inherit: true,
                rules: [
                    { token: 'keyword', foreground: '569cd6', fontStyle: 'bold' },
                    { token: 'type', foreground: '9cdcfe' },
                    { token: 'string', foreground: 'ce9178' },
                    { token: 'comment', foreground: '6a9955' },
                    { token: 'number', foreground: 'b5cea8' },
                    { token: 'number.hex', foreground: '4ec9b0' }
                ],
                colors: {
                    'editor.background': '#1e1e1e',
                }
            });
        }
    }, [monaco]);

    const handleSave = () => {
        onSave(localSource);
    };

    const handleEditorChange = useCallback((value: string | undefined) => {
        if (value !== undefined) {
            setLocalSource(value);
        }
    }, []);

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%', width: '100%' }}>
            <div style={{
                display: 'flex', alignItems: 'center', gap: '8px',
                padding: '6px 12px', borderBottom: '1px solid var(--border-subtle)',
            }}>
                <span style={{ fontSize: '11px', color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.08em', fontWeight: 600 }}>
                    Source
                </span>
                <div style={{ flex: 1 }} />
                <button className="btn btn-primary" onClick={handleSave} style={{ padding: '3px 10px', fontSize: '11px' }}>
                    Save & Compile
                </button>
            </div>
            <div style={{ flex: 1, position: 'relative' }}>
                <Editor
                    height="100%"
                    width="100%"
                    language="vidra"
                    theme="vidra-dark"
                    value={localSource}
                    onChange={handleEditorChange}
                    options={{
                        minimap: { enabled: false },
                        fontSize: 13,
                        fontFamily: "'SF Mono', Consolas, 'Courier New', monospace",
                        scrollBeyondLastLine: false,
                        wordWrap: 'on',
                        automaticLayout: true,
                        bracketPairColorization: { enabled: true },
                    }}
                />
            </div>
        </div>
    );
}
