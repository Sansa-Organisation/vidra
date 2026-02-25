// useProject — project state + undo/redo (Task 9.3)
import { create } from 'zustand';

interface ProjectScene {
    id: string;
    name: string;
    duration_frames: number;
    layers: ProjectLayer[];
}

interface ProjectLayer {
    id: string;
    content_type: string;
    label?: string;
}

interface ProjectState {
    irJson: string | null;
    source: string | null;
    scenes: ProjectScene[];
    selectedLayerId: string | null;
    dirty: boolean;
    undoStack: string[];
    redoStack: string[];
    frame: number;
    playing: boolean;

    setIr: (ir: string) => void;
    setSource: (src: string) => void;
    setScenes: (scenes: ProjectScene[]) => void;
    selectLayer: (id: string | null) => void;
    setFrame: (f: number) => void;
    setPlaying: (p: boolean) => void;
    pushUndo: () => void;
    undo: () => void;
    redo: () => void;
}

export const useProjectStore = create<ProjectState>((set, get) => ({
    irJson: null,
    source: null,
    scenes: [],
    selectedLayerId: null,
    dirty: false,
    undoStack: [],
    redoStack: [],
    frame: 0,
    playing: false,

    setIr: (ir) => set({ irJson: ir, dirty: false }),
    setSource: (src) => set({ source: src }),
    setScenes: (scenes) => set({ scenes }),
    selectLayer: (id) => set({ selectedLayerId: id }),
    setFrame: (f) => set({ frame: f }),
    setPlaying: (p) => set({ playing: p }),

    pushUndo: () => {
        const { source, undoStack } = get();
        if (source) {
            set({ undoStack: [...undoStack, source], redoStack: [], dirty: true });
        }
    },

    undo: () => {
        const { undoStack, source } = get();
        if (undoStack.length === 0) return;
        const prev = undoStack[undoStack.length - 1];
        set({
            undoStack: undoStack.slice(0, -1),
            redoStack: source ? [...get().redoStack, source] : get().redoStack,
            source: prev,
            dirty: true,
        });
    },

    redo: () => {
        const { redoStack, source } = get();
        if (redoStack.length === 0) return;
        const next = redoStack[redoStack.length - 1];
        set({
            redoStack: redoStack.slice(0, -1),
            undoStack: source ? [...get().undoStack, source] : get().undoStack,
            source: next,
            dirty: true,
        });
    },
}));

export type { ProjectScene, ProjectLayer };
