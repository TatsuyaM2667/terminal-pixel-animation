import { useState, useEffect, useRef, useCallback, createContext, useContext } from "react";
import type { ReactNode } from "react";
import init, {
  render_braille as wasmRenderBraille,
  render_half_block as wasmRenderHalfBlock,
} from "terminal-pixel-animation";

// ── Wasm Context ──────────────────────────────────────────────────────────────

interface WasmState {
  ready: boolean;
  error: Error | null;
}

const WasmContext = createContext<WasmState>({ ready: false, error: null });

export interface WasmProviderProps {
  children: ReactNode;
  /** Optional URL/path to the .wasm file. If omitted, uses default from import.meta.url. */
  wasmUrl?: string;
}

export function WasmProvider({ children, wasmUrl }: WasmProviderProps) {
  const [state, setState] = useState<WasmState>({ ready: false, error: null });

  useEffect(() => {
    let cancelled = false;
    init(wasmUrl)
      .then(() => {
        if (!cancelled) setState({ ready: true, error: null });
      })
      .catch((err) => {
        if (!cancelled)
          setState({ ready: false, error: err instanceof Error ? err : new Error(String(err)) });
      });
    return () => {
      cancelled = true;
    };
  }, [wasmUrl]);

  return <WasmContext.Provider value={state}>{children}</WasmContext.Provider>;
}

export function useWasm(): WasmState {
  return useContext(WasmContext);
}

// ── Braille ───────────────────────────────────────────────────────────────────

export interface BrailleCell {
  char: string;
  r: number;
  g: number;
  b: number;
}

export interface UseBrailleResult {
  cells: Uint8Array | null;
  decoded: BrailleCell[] | null;
  loading: boolean;
  error: Error | null;
}

export function useBraille(
  pixels: Uint8Array | null,
  inWidth: number,
  inHeight: number,
  cols: number,
  rows: number
): UseBrailleResult {
  const { ready, error: wasmError } = useWasm();
  const [result, setResult] = useState<UseBrailleResult>({
    cells: null,
    decoded: null,
    loading: false,
    error: null,
  });
  const prevPixelsRef = useRef<Uint8Array | null>(null);

  const render = useCallback(() => {
    if (!ready || !pixels || inWidth === 0 || inHeight === 0 || cols === 0 || rows === 0) {
      return;
    }

    if (pixels === prevPixelsRef.current) return;
    prevPixelsRef.current = pixels;

    try {
      setResult((prev) => ({ ...prev, loading: true, error: null }));
      const cells = wasmRenderBraille(pixels, inWidth, inHeight, cols, rows);
      const decoded = decodeBrailleCells(cells, cols, rows);
      setResult({ cells, decoded, loading: false, error: null });
    } catch (err) {
      setResult({
        cells: null,
        decoded: null,
        loading: false,
        error: err instanceof Error ? err : new Error(String(err)),
      });
    }
  }, [ready, pixels, inWidth, inHeight, cols, rows]);

  useEffect(() => {
    render();
  }, [render]);

  if (wasmError) {
    return { cells: null, decoded: null, loading: false, error: wasmError };
  }

  return result;
}

function decodeBrailleCells(cells: Uint8Array, cols: number, rows: number): BrailleCell[] {
  const result: BrailleCell[] = [];
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const idx = (row * cols + col) * 8;
      const cp = cells[idx] | (cells[idx + 1] << 8) | (cells[idx + 2] << 16) | (cells[idx + 3] << 24);
      result.push({
        char: String.fromCodePoint(cp),
        r: cells[idx + 4],
        g: cells[idx + 5],
        b: cells[idx + 6],
      });
    }
  }
  return result;
}

// ── Half-block ────────────────────────────────────────────────────────────────

export interface HalfBlockCell {
  rFg: number;
  gFg: number;
  bFg: number;
  rBg: number;
  gBg: number;
  bBg: number;
}

export interface UseHalfBlockResult {
  cells: Uint8Array | null;
  decoded: HalfBlockCell[] | null;
  loading: boolean;
  error: Error | null;
}

export function useHalfBlock(
  pixels: Uint8Array | null,
  inWidth: number,
  inHeight: number,
  cols: number,
  rows: number
): UseHalfBlockResult {
  const { ready, error: wasmError } = useWasm();
  const [result, setResult] = useState<UseHalfBlockResult>({
    cells: null,
    decoded: null,
    loading: false,
    error: null,
  });
  const prevPixelsRef = useRef<Uint8Array | null>(null);

  const render = useCallback(() => {
    if (!ready || !pixels || inWidth === 0 || inHeight === 0 || cols === 0 || rows === 0) {
      return;
    }

    if (pixels === prevPixelsRef.current) return;
    prevPixelsRef.current = pixels;

    try {
      setResult((prev) => ({ ...prev, loading: true, error: null }));
      const cells = wasmRenderHalfBlock(pixels, inWidth, inHeight, cols, rows);
      const decoded = decodeHalfBlockCells(cells, cols, rows);
      setResult({ cells, decoded, loading: false, error: null });
    } catch (err) {
      setResult({
        cells: null,
        decoded: null,
        loading: false,
        error: err instanceof Error ? err : new Error(String(err)),
      });
    }
  }, [ready, pixels, inWidth, inHeight, cols, rows]);

  useEffect(() => {
    render();
  }, [render]);

  if (wasmError) {
    return { cells: null, decoded: null, loading: false, error: wasmError };
  }

  return result;
}

function decodeHalfBlockCells(cells: Uint8Array, cols: number, rows: number): HalfBlockCell[] {
  const result: HalfBlockCell[] = [];
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const idx = (row * cols + col) * 6;
      result.push({
        rFg: cells[idx],
        gFg: cells[idx + 1],
        bFg: cells[idx + 2],
        rBg: cells[idx + 3],
        gBg: cells[idx + 4],
        bBg: cells[idx + 5],
      });
    }
  }
  return result;
}
