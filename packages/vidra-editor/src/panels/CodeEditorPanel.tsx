// Code Editor panel — VidraScript source editor (Task 9.8)
import { useState, useEffect, useRef } from 'react';
import { useProjectStore } from '../hooks/useProject';

interface Props {
    onSave: (source: string) => void;
}

export function CodeEditorPanel({ onSave }: Props) {
    const source = useProjectStore(s => s.source);
    const [localSource, setLocalSource] = useState(source || '');
    const textareaRef = useRef<HTMLTextAreaElement>(null);

    useEffect(() => {
        if (source !== null && source !== localSource) {
            setLocalSource(source);
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [source]);

    const handleSave = () => {
        onSave(localSource);
    };

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
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
            <textarea
                ref={textareaRef}
                className="code-pane"
                value={localSource}
                onChange={e => setLocalSource(e.target.value)}
                spellCheck={false}
            />
        </div>
    );
}
