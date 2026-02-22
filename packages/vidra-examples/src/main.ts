import './style.css';
import { VidraEngine, Project, Scene, Layer, Easing, hex, rgba } from '@sansavision/vidra-player';

// ── DOM Elements ──────────────────────────────────────────────────────────
const qs = (sel: string) => document.querySelector(sel);
const canvas = qs('#vidraCanvas') as HTMLCanvasElement;
const compileBtn = qs('#compileBtn') as HTMLButtonElement;
const playBtn = qs('#playBtn') as HTMLButtonElement;
const pauseBtn = qs('#pauseBtn') as HTMLButtonElement;
const stopBtn = qs('#stopBtn') as HTMLButtonElement;
const codeEditor = qs('#codeEditor') as HTMLTextAreaElement;
const statusBadge = qs('#statusBadge') as HTMLElement;
const metaInfo = qs('#metaInfo') as HTMLElement;
const currentTimeEl = qs('#currentTime') as HTMLElement;
const totalTimeEl = qs('#totalTime') as HTMLElement;
const progressFill = qs('#progressFill') as HTMLElement;
const progressBar = qs('.progress-bar') as HTMLElement;
const assetDropZone = qs('#assetDropZone') as HTMLElement;
const fileInput = qs('#fileInput') as HTMLInputElement;
const assetList = qs('#assetList') as HTMLElement;

let currentMode: 'vidrascript' | 'sdk' = 'vidrascript';

// ── Editor Content ───────────────────────────────────────────────────────
const defaultScripts = {
  vidrascript: `project(1920, 1080, 60) {
    scene("welcome", 3s) {
        layer("bg") {
            solid(#09090b)
        }
        layer("title") {
            text("Vidra Studio", font: "Inter", size: 140, color: #ffffff)
            position(960, 540)
            animation(opacity, from: 0, to: 1, duration: 2s, easing: easeOut)
            animation(scale, from: 0.9, to: 1.0, duration: 2s, easing: easeOutBack)
        }
    }
    
    scene("features", 4s) {
        layer("bg2") {
            solid(#18181b)
        }
        layer("headline") {
            text("Showcase Your Engine", font: "Inter", size: 80, color: #3b82f6)
            position(960, 400)
            animation(positionY, from: 450, to: 400, duration: 1s, easing: easeOut)
            animation(opacity, from: 0, to: 1, duration: 1s)
        }
        layer("sub") {
            text("Drop an image layer to try the asset manager!", font: "Inter", size: 40, color: #94a3b8)
            position(960, 600)
            animation(opacity, from: 0, to: 1, duration: 1s, delay: 0.5s)
        }
    }
}`,
  sdk: `// Use the fluent TypeScript / JS API
const project = new Project(1920, 1080, 60)
    .background("#09090b");

const s1 = new Scene("intro", 3.0);
s1.addLayers(
    new Layer("bg").solid("#09090b"),
    new Layer("text")
        .text("Powered by JS SDK", "Inter", 120, "#10b981")
        .position(960, 540)
        .animate("opacity", 0, 1, 1.5, Easing.EaseOut)
);
project.addScene(s1);

// Try uploading an image and referencing it here like:
// const imageScene = new Scene("img", 3.0);
// imageScene.addLayer(new Layer("logo").image("your-asset-id").position(960, 540));
// project.addScene(imageScene);

return project;`
};

codeEditor.value = defaultScripts.vidrascript;

// ── Tab Switching ────────────────────────────────────────────────────────
document.querySelectorAll('.tab-btn').forEach(btn => {
  btn.addEventListener('click', (e) => {
    const target = e.target as HTMLButtonElement;
    document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    target.classList.add('active');

    currentMode = target.dataset.mode as 'vidrascript' | 'sdk';
    codeEditor.value = defaultScripts[currentMode];
  });
});

// ── Engine Initialization ─────────────────────────────────────────────────
const engine = new VidraEngine(canvas, {
  onReady: () => {
    statusBadge.textContent = "Engine Ready";
    statusBadge.className = "status-badge ready";
    compileBtn.disabled = false;
  },
  onStateChange: (state) => {
    playBtn.disabled = state === "loading" || state === "idle";
    pauseBtn.disabled = state !== "playing";
    stopBtn.disabled = state === "loading" || state === "idle";
  },
  onFrame: (frame) => {
    updateTimeDisplay(frame);
  },
  onError: (err) => {
    console.error(err);
    statusBadge.textContent = "Error";
    statusBadge.className = "status-badge";
    alert(`Render Error: ${err}`);
  }
});

// Initialize the engine!
(async () => {
  try {
    await engine.init();
    console.log("Vidra WASM Engine v" + engine.getVersion() + " initialized.");
  } catch (err) {
    statusBadge.textContent = "Failed to load WASM";
    console.error(err);
  }
})();

// ── Asset Management ──────────────────────────────────────────────────────
const uploadedAssets: { id: string, url: string, file: File }[] = [];

fileInput.addEventListener('change', async (e) => {
  const files = (e.target as HTMLInputElement).files;
  if (!files) return;
  await handleFiles(Array.from(files));
});

assetDropZone.addEventListener('dragover', (e) => {
  e.preventDefault();
  assetDropZone.style.borderColor = '#3b82f6';
  assetDropZone.style.background = 'rgba(59, 130, 246, 0.1)';
});

assetDropZone.addEventListener('dragleave', (e) => {
  e.preventDefault();
  assetDropZone.style.borderColor = '';
  assetDropZone.style.background = '';
});

assetDropZone.addEventListener('drop', async (e) => {
  e.preventDefault();
  assetDropZone.style.borderColor = '';
  assetDropZone.style.background = '';
  if (e.dataTransfer?.files) {
    await handleFiles(Array.from(e.dataTransfer.files));
  }
});

async function handleFiles(files: File[]) {
  for (const file of files) {
    const id = file.name.split('.')[0].replace(/[^a-zA-Z0-9_-]/g, '_');
    const url = URL.createObjectURL(file);
    uploadedAssets.push({ id, url, file });

    // Create UI item
    const li = document.createElement('li');
    li.className = 'asset-item';
    li.innerHTML = `
      <span class="asset-name">${file.name}</span>
      <span class="asset-id">${id}</span>
    `;
    assetList.appendChild(li);

    // If it's an image, load it into WASM renderer cache
    if (file.type.startsWith('image/')) {
      try {
        await engine.loadImageAsset(id, url);
        console.log(`Loaded image asset ${id} into WASM cache.`);
      } catch (e) {
        console.error('Failed to load asset into engine', e);
      }
    }
  }
}

// ── Compilation & Playback ────────────────────────────────────────────────
function formatTime(seconds: number) {
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  const ms = Math.floor((seconds % 1) * 100);
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}.${ms.toString().padStart(2, '0')}`;
}

function updateTimeDisplay(frame: number) {
  const info = engine.getProjectInfo();
  if (!info) return;
  const currTime = frame / info.fps;
  currentTimeEl.textContent = formatTime(currTime);
  progressFill.style.width = `${(frame / Math.max(1, info.totalFrames - 1)) * 100}%`;
}

progressBar.addEventListener('click', (e) => {
  const info = engine.getProjectInfo();
  if (!info) return;
  const rect = progressBar.getBoundingClientRect();
  const clickX = Math.max(0, e.clientX - rect.left);
  const percent = clickX / rect.width;
  const targetFrame = Math.floor(percent * info.totalFrames);
  engine.seekToFrame(targetFrame);
  if (engine.getState() !== "playing") {
    updateTimeDisplay(targetFrame);
  }
});

compileBtn.addEventListener('click', () => {
  const code = codeEditor.value;
  try {
    let info;
    if (currentMode === 'vidrascript') {
      info = engine.loadSource(code);
    } else {
      // Evaluate JS SDK Context
      // Expose necessary classes
      const buildFn = new Function(
        "Project", "Scene", "Layer", "Easing", "hex", "rgba",
        code
      );
      const project = buildFn(Project, Scene, Layer, Easing, hex, rgba);
      // For assets uploaded, the user can just reference the ID. 
      // The image content is already in WASM via loadImageAsset.
      info = engine.loadProject(project);
    }

    // Update Meta Info
    metaInfo.textContent = `Resolution: ${info.width}×${info.height} | FPS: ${info.fps}`;
    totalTimeEl.textContent = formatTime(info.totalDuration);
    updateTimeDisplay(0);

    // Autosave as new default for tab
    defaultScripts[currentMode] = code;

  } catch (err: any) {
    console.error(err);
    alert(`Compilation Error: ${err.message || err}`);
  }
});

playBtn.addEventListener('click', () => engine.play());
pauseBtn.addEventListener('click', () => engine.pause());
stopBtn.addEventListener('click', () => engine.stop());

