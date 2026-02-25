// Toolbar (Task 9.10)
import { useProjectStore } from '../hooks/useProject';

interface Props {
    onExport: () => void;
    rest: (method: string, path: string, body?: unknown) => Promise<unknown>;
}

export function Toolbar({ onExport }: Props) {
    const { undo, redo, undoStack, redoStack } = useProjectStore();

    return (
        <div className="toolbar">
            <button className="toolbar-btn" title="Undo (Ctrl+Z)" disabled={undoStack.length === 0} onClick={undo}>↩</button>
            <button className="toolbar-btn" title="Redo (Ctrl+Y)" disabled={redoStack.length === 0} onClick={redo}>↪</button>
            <div style={{ height: '1px', background: 'var(--border-default)', width: '24px', margin: '4px 0' }} />
            <button className="toolbar-btn btn-primary" title="Export" onClick={onExport} style={{ background: 'var(--accent-blue)', color: '#fff' }}>⬇</button>
        </div>
    );
}
