export interface OpenAiTranscribeCacheKeyInput {
    baseUrl: string;
    model: string;
    audioSha256: string;
}

export interface RemoveBgCacheKeyInput {
    baseUrl: string;
    imageSha256: string;
}

export interface GeminiCaptionRefineCacheKeyInput {
    baseUrl: string;
    model: string;
    segmentsSha256: string;
}

function bytesToHex(bytes: ArrayBuffer): string {
    const u8 = new Uint8Array(bytes);
    let out = "";
    for (let i = 0; i < u8.length; i++) {
        out += u8[i].toString(16).padStart(2, "0");
    }
    return out;
}

export async function sha256HexString(input: string): Promise<string> {
    const data = new TextEncoder().encode(input);
    const digest = await crypto.subtle.digest("SHA-256", data);
    return bytesToHex(digest);
}

export async function sha256HexBytes(bytes: Uint8Array): Promise<string> {
    // WebCrypto typing can be strict about ArrayBuffer vs SharedArrayBuffer.
    // Copy into an ArrayBuffer-backed view to keep this browser-safe and TS-friendly.
    const copy = new Uint8Array(bytes);
    const digest = await crypto.subtle.digest("SHA-256", copy.buffer);
    return bytesToHex(digest);
}

/**
 * Matches the Rust CLI cache key string:
 * `openai_transcribe|base_url=...|model=...|audio_sha256=...` -> sha256_hex
 */
export async function openAiTranscribeCacheKey(input: OpenAiTranscribeCacheKeyInput): Promise<string> {
    return sha256HexString(
        `openai_transcribe|base_url=${input.baseUrl}|model=${input.model}|audio_sha256=${input.audioSha256}`,
    );
}

/**
 * Matches the Rust CLI cache key string:
 * `removebg|base_url=...|img_sha256=...` -> sha256_hex
 */
export async function removeBgCacheKey(input: RemoveBgCacheKeyInput): Promise<string> {
    return sha256HexString(`removebg|base_url=${input.baseUrl}|img_sha256=${input.imageSha256}`);
}

/**
 * Matches the Rust CLI cache key string:
 * `gemini_caption_refine|base_url=...|model=...|segments_sha256=...` -> sha256_hex
 */
export async function geminiCaptionRefineCacheKey(input: GeminiCaptionRefineCacheKeyInput): Promise<string> {
    return sha256HexString(
        `gemini_caption_refine|base_url=${input.baseUrl}|model=${input.model}|segments_sha256=${input.segmentsSha256}`,
    );
}
