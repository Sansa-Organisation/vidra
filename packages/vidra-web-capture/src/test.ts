// Standalone test for VidraCapture graceful degradation.
// Run with: node dist/test.js

import { VidraCapture } from './index.js';

// ── Test 1: VidraCapture outside harness returns defaults ───────────
const capture = new VidraCapture();
const state = capture.getState();

console.assert(state.capturing === false, 'Expected capturing=false in standalone');
console.assert(state.frame === 0, 'Expected frame=0 in standalone');
console.assert(typeof state.time === 'number' && state.time >= 0, 'Expected time>=0');
console.assert(state.fps === 60, 'Expected fps=60 default');
console.assert(typeof state.vars === 'object', 'Expected vars to be an object');
console.assert(Object.keys(state.vars).length === 0, 'Expected empty vars');
console.assert(typeof state.emit === 'function', 'Expected emit to be a function');

// emit() should not throw when not capturing
state.emit('test_key', 42);

// ── Test 2: isCapturing() returns false ─────────────────────────────
console.assert(capture.isCapturing() === false, 'isCapturing should be false');

// ── Test 3: emit() convenience method works without error ───────────
capture.emit('layout_height', 1080);

console.log('✅ All VidraCapture standalone tests passed');
