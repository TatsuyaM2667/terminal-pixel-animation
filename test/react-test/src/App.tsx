import { useState, useEffect, useRef, useCallback } from "react";
import { WasmProvider, useBraille, useHalfBlock } from "terminal-pixel-animation-react";

// ── Pixel capture hook ───────────────────────────────────────────────────────

function useVideoFrames(videoRef: React.RefObject<HTMLVideoElement | null>) {
  const [pixels, setPixels] = useState<Uint8Array | null>(null);
  const [size, setSize] = useState({ w: 0, h: 0 });
  const offscreenRef = useRef<HTMLCanvasElement | null>(null);
  const ctxRef = useRef<CanvasRenderingContext2D | null>(null);

  useEffect(() => {
    const canvas = document.createElement("canvas");
    offscreenRef.current = canvas;
    ctxRef.current = canvas.getContext("2d", { willReadFrequently: true });
    return () => {
      offscreenRef.current = null;
      ctxRef.current = null;
    };
  }, []);

  const capture = useCallback(() => {
    const video = videoRef.current;
    const off = offscreenRef.current;
    const ctx = ctxRef.current;
    if (!video || !off || !ctx || video.readyState < 2) return;

    const vw = video.videoWidth;
    const vh = video.videoHeight;
    if (off.width !== vw || off.height !== vh) {
      off.width = vw;
      off.height = vh;
    }

    ctx.drawImage(video, 0, 0, vw, vh);
    const imageData = ctx.getImageData(0, 0, vw, vh);

    const rgb = new Uint8Array(vw * vh * 3);
    for (let i = 0, j = 0; i < imageData.data.length; i += 4, j += 3) {
      rgb[j] = imageData.data[i];
      rgb[j + 1] = imageData.data[i + 1];
      rgb[j + 2] = imageData.data[i + 2];
    }

    setPixels(rgb);
    setSize({ w: vw, h: vh });
  }, [videoRef]);

  return { pixels, size, capture };
}

// ── FPS counter hook ─────────────────────────────────────────────────────────

function useFps() {
  const [fps, setFps] = useState(0);
  const countRef = useRef(0);
  const lastRef = useRef(performance.now());

  const tick = useCallback(() => {
    countRef.current++;
    const now = performance.now();
    if (now - lastRef.current >= 1000) {
      setFps(countRef.current);
      countRef.current = 0;
      lastRef.current = now;
    }
  }, []);

  return { fps, tick };
}

// ── Braille renderer ─────────────────────────────────────────────────────────

function BrailleCanvas({ pixels, width, height, cols, rows }: {
  pixels: Uint8Array; width: number; height: number; cols: number; rows: number;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { decoded, error } = useBraille(pixels, width, height, cols, rows);
  const { fps, tick } = useFps();

  useEffect(() => {
    if (!decoded || !canvasRef.current) return;
    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d")!;

    const cellW = 8;
    const cellH = 14;
    canvas.width = cols * cellW;
    canvas.height = rows * cellH;

    ctx.fillStyle = "#000";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.font = `${cellH}px monospace`;
    ctx.textBaseline = "top";

    for (let i = 0; i < decoded.length; i++) {
      const cell = decoded[i];
      const col = i % cols;
      const row = Math.floor(i / cols);
      if (cell.char !== "\0" && cell.char !== " ") {
        ctx.fillStyle = `rgb(${cell.r},${cell.g},${cell.b})`;
        ctx.fillText(cell.char, col * cellW, row * cellH);
      }
    }

    tick();
  }, [decoded, cols, rows, tick]);

  if (error) return <p style={{ color: "red" }}>Error: {error.message}</p>;

  return (
    <div>
      <div style={{ marginBottom: 8, fontSize: 12, color: "#888" }}>
        Braille {cols}x{rows} | FPS: {fps}
      </div>
      <canvas ref={canvasRef} style={{ imageRendering: "pixelated", background: "#000" }} />
    </div>
  );
}

// ── Half-block renderer ──────────────────────────────────────────────────────

function HalfBlockCanvas({ pixels, width, height, cols, rows }: {
  pixels: Uint8Array; width: number; height: number; cols: number; rows: number;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { decoded, error } = useHalfBlock(pixels, width, height, cols, rows);
  const { fps, tick } = useFps();

  useEffect(() => {
    if (!decoded || !canvasRef.current) return;
    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d")!;

    const px = 8;
    canvas.width = cols * px;
    canvas.height = rows * px;

    for (let i = 0; i < decoded.length; i++) {
      const cell = decoded[i];
      const col = i % cols;
      const row = Math.floor(i / cols);

      ctx.fillStyle = `rgb(${cell.rFg},${cell.gFg},${cell.bFg})`;
      ctx.fillRect(col * px, row * px, px, px / 2);
      ctx.fillStyle = `rgb(${cell.rBg},${cell.gBg},${cell.bBg})`;
      ctx.fillRect(col * px, row * px + px / 2, px, px / 2);
    }

    tick();
  }, [decoded, cols, rows, tick]);

  if (error) return <p style={{ color: "red" }}>Error: {error.message}</p>;

  return (
    <div>
      <div style={{ marginBottom: 8, fontSize: 12, color: "#888" }}>
        Half-block {cols}x{rows} | FPS: {fps}
      </div>
      <canvas ref={canvasRef} style={{ imageRendering: "pixelated", background: "#000" }} />
    </div>
  );
}

// ── Main App ─────────────────────────────────────────────────────────────────

type Renderer = "braille" | "halfblock";

function Demo() {
  const videoRef = useRef<HTMLVideoElement>(null);
  const imageRef = useRef<HTMLImageElement>(null);
  const [renderer, setRenderer] = useState<Renderer>("braille");
  const [sourceType, setSourceType] = useState<"video" | "image" | null>(null);
  const [source, setSource] = useState<string | null>(null);
  const [stream, setStream] = useState<MediaStream | null>(null);
  const [cols, setCols] = useState(100);
  const [rows, setRows] = useState(45);
  const [imagePixels, setImagePixels] = useState<Uint8Array | null>(null);
  const [imageSize, setImageSize] = useState({ w: 0, h: 0 });
  const fileInputRef = useRef<HTMLInputElement>(null);
  const imageInputRef = useRef<HTMLInputElement>(null);

  const { pixels: videoPixels, size: videoSize, capture } = useVideoFrames(videoRef);
  const animRef = useRef<number>(0);

  const pixels = sourceType === "image" ? imagePixels : videoPixels;
  const size = sourceType === "image" ? imageSize : videoSize;

  // Render loop (video only)
  useEffect(() => {
    let running = true;
    function loop() {
      if (!running) return;
      capture();
      animRef.current = requestAnimationFrame(loop);
    }
    if (source && sourceType === "video") loop();
    return () => { running = false; cancelAnimationFrame(animRef.current); };
  }, [source, sourceType, capture]);

  // Cleanup stream on unmount
  useEffect(() => {
    return () => { stream?.getTracks().forEach((t) => t.stop()); };
  }, [stream]);

  const handleWebcam = useCallback(async () => {
    try {
      const s = await navigator.mediaDevices.getUserMedia({
        video: { width: { ideal: 1280 }, height: { ideal: 720 } },
      });
      stream?.getTracks().forEach((t) => t.stop());
      setStream(s);
      setImagePixels(null);
      setImageSize({ w: 0, h: 0 });
      if (videoRef.current) {
        videoRef.current.srcObject = s;
        videoRef.current.play();
      }
      setSourceType("video");
      setSource("webcam");
    } catch (e) {
      alert(`Webcam error: ${e}`);
    }
  }, [stream]);

  const handleVideoFile = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleVideoFileChange = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    stream?.getTracks().forEach((t) => t.stop());
    setStream(null);
    setImagePixels(null);
    setImageSize({ w: 0, h: 0 });
    if (videoRef.current) {
      videoRef.current.srcObject = null;
      videoRef.current.src = URL.createObjectURL(file);
      videoRef.current.loop = true;
      videoRef.current.play();
    }
    setSourceType("video");
    setSource(file.name);
  }, [stream]);

  const handleImageFile = useCallback(() => {
    imageInputRef.current?.click();
  }, []);

  const handleImageFileChange = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    stream?.getTracks().forEach((t) => t.stop());
    setStream(null);
    if (videoRef.current) {
      videoRef.current.srcObject = null;
      videoRef.current.src = "";
    }
    cancelAnimationFrame(animRef.current);

    const img = imageRef.current;
    if (!img) return;
    const url = URL.createObjectURL(file);
    img.onload = () => {
      const w = img.naturalWidth;
      const h = img.naturalHeight;
      const canvas = document.createElement("canvas");
      canvas.width = w;
      canvas.height = h;
      const ctx = canvas.getContext("2d", { willReadFrequently: true })!;
      ctx.drawImage(img, 0, 0);
      const imageData = ctx.getImageData(0, 0, w, h);
      const rgb = new Uint8Array(w * h * 3);
      for (let i = 0, j = 0; i < imageData.data.length; i += 4, j += 3) {
        rgb[j] = imageData.data[i];
        rgb[j + 1] = imageData.data[i + 1];
        rgb[j + 2] = imageData.data[i + 2];
      }
      setImagePixels(rgb);
      setImageSize({ w, h });
      URL.revokeObjectURL(url);
    };
    img.src = url;
    setSourceType("image");
    setSource(file.name);
  }, [stream]);

  const handleDisconnect = useCallback(() => {
    stream?.getTracks().forEach((t) => t.stop());
    setStream(null);
    if (videoRef.current) {
      videoRef.current.srcObject = null;
      videoRef.current.src = "";
    }
    cancelAnimationFrame(animRef.current);
    setImagePixels(null);
    setImageSize({ w: 0, h: 0 });
    setSourceType(null);
    setSource(null);
  }, [stream]);

  const btnStyle = (active: boolean): React.CSSProperties => ({
    background: active ? "#0a2a0a" : "#1a1a1a",
    border: `1px solid ${active ? "#0f0" : "#333"}`,
    color: active ? "#0f0" : "#ccc",
    padding: "6px 14px",
    borderRadius: 4,
    cursor: "pointer",
    fontFamily: "inherit",
    fontSize: 12,
  });

  return (
    <div style={{ minHeight: "100vh", background: "#0a0a0a", color: "#e0e0e0", fontFamily: "'JetBrains Mono', monospace" }}>
      {/* Header */}
      <div style={{ padding: "16px 24px", borderBottom: "1px solid #222", display: "flex", alignItems: "center", gap: 12 }}>
        <h1 style={{ fontSize: 16, fontWeight: 600 }}>
          terminal-pixel-animation <span style={{ color: "#666" }}>::</span> React demo
        </h1>
        <span style={{ display: "inline-block", width: 8, height: 18, background: "#0f0", animation: "blink 1s step-end infinite" }} />
      </div>

      {/* Controls */}
      <div style={{ display: "flex", gap: 1, background: "#222" }}>
        <div style={{ background: "#0a0a0a", padding: "16px 24px", flex: 1 }}>
          <div style={{ fontSize: 11, textTransform: "uppercase", letterSpacing: 2, color: "#888", marginBottom: 12 }}>Renderer</div>
          <div style={{ display: "flex", gap: 8 }}>
            <button style={btnStyle(renderer === "braille")} onClick={() => setRenderer("braille")}>Braille (Odin)</button>
            <button style={btnStyle(renderer === "halfblock")} onClick={() => setRenderer("halfblock")}>Half-block (Zig)</button>
          </div>
        </div>
        <div style={{ background: "#0a0a0a", padding: "16px 24px", flex: 2 }}>
          <div style={{ fontSize: 11, textTransform: "uppercase", letterSpacing: 2, color: "#888", marginBottom: 12 }}>Source</div>
          <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
            <button style={btnStyle(false)} onClick={handleWebcam}>Webcam</button>
            <button style={btnStyle(false)} onClick={handleVideoFile}>Video file</button>
            <button style={btnStyle(false)} onClick={handleImageFile}>Image file</button>
            {source && (
              <button style={{ ...btnStyle(false), borderColor: "#a00", color: "#a00" }} onClick={handleDisconnect}>Disconnect</button>
            )}
            <label style={{ fontSize: 12, color: "#888" }}>
              Cols{" "}
              <input type="number" value={cols} min={10} max={300}
                onChange={(e) => setCols(Number(e.target.value))}
                style={{ background: "#111", border: "1px solid #333", color: "#ccc", padding: "4px 8px", width: 60, borderRadius: 3, fontFamily: "inherit", fontSize: 12, textAlign: "center" }}
              />
            </label>
            <label style={{ fontSize: 12, color: "#888" }}>
              Rows{" "}
              <input type="number" value={rows} min={5} max={150}
                onChange={(e) => setRows(Number(e.target.value))}
                style={{ background: "#111", border: "1px solid #333", color: "#ccc", padding: "4px 8px", width: 60, borderRadius: 3, fontFamily: "inherit", fontSize: 12, textAlign: "center" }}
              />
            </label>
            <div style={{ display: "flex", gap: 4 }}>
              {(["80x24", "100x45", "120x50", "160x60"] as const).map((preset) => {
                const [pCols, pRows] = preset.split("x").map(Number);
                const active = cols === pCols && rows === pRows;
                return (
                  <button key={preset} style={btnStyle(active)} onClick={() => { setCols(pCols); setRows(pRows); }}>
                    {preset}
                  </button>
                );
              })}
            </div>
          </div>
          <div style={{ fontSize: 11, color: "#666", marginTop: 8 }}>
            {source ? `Source: ${source}` : "Waiting for source..."}
          </div>
        </div>
      </div>

      {/* Canvas */}
      <div style={{ display: "flex", justifyContent: "center", padding: 24 }}>
        {pixels && size.w > 0 ? (
          renderer === "braille" ? (
            <BrailleCanvas pixels={pixels} width={size.w} height={size.h} cols={cols} rows={rows} />
          ) : (
            <HalfBlockCanvas pixels={pixels} width={size.w} height={size.h} cols={cols} rows={rows} />
          )
        ) : (
          <div style={{ color: "#444", fontSize: 14, padding: 48 }}>No source selected</div>
        )}
      </div>

      {/* Hidden elements */}
      <video ref={videoRef} autoPlay playsInline style={{ display: "none" }} />
      <img ref={imageRef} style={{ display: "none" }} crossOrigin="anonymous" />
      <input ref={fileInputRef} type="file" accept="video/*" onChange={handleVideoFileChange} style={{ display: "none" }} />
      <input ref={imageInputRef} type="file" accept="image/*" onChange={handleImageFileChange} style={{ display: "none" }} />

      <style>{`@keyframes blink { 50% { opacity: 0; } }`}</style>
    </div>
  );
}

export default function App() {
  return (
    <WasmProvider>
      <Demo />
    </WasmProvider>
  );
}
