import { convertFileSrc } from "@tauri-apps/api/core";
import { mat4, quat, vec3 } from "gl-matrix";
import { type PointerEvent, useEffect, useRef, useState, type WheelEvent } from "react";
import { ModelRenderer, parseMDX } from "war3-model";
import { errorMessage } from "../api";

type Model = ReturnType<typeof parseMDX>;

type ModelViewerProps = {
  /** The unit's model path as stored in its data, e.g. `units\human\Footman\Footman`. */
  path: string | null;
};

type Animation = {
  index: number;
  name: string;
  /** Start frame: unique even when names repeat. */
  start: number;
};

type Status =
  | { kind: "empty" }
  | { kind: "loading" }
  | { kind: "ready" }
  | { kind: "error"; message: string };

const MODEL_SCHEME = "wc3model";
const TEXTURE_SCHEME = "wc3tex";
/** Raw textures start with width and height as little-endian u32s. */
const TEXTURE_HEADER_BYTES = 8;
const BYTES_PER_PIXEL = 4;

const FIELD_OF_VIEW_DEGREES = 45;
const DEGREES_PER_HALF_TURN = 180;
const FIELD_OF_VIEW = (FIELD_OF_VIEW_DEGREES * Math.PI) / DEGREES_PER_HALF_TURN;
const NEAR_PLANE = 1;
const FAR_PLANE = 100_000;
/** Distance from the model, in model radiuses. */
const START_DISTANCE = 2.6;
const MIN_DISTANCE = 0.8;
const MAX_DISTANCE = 8;
const ZOOM_STEP = 0.0015;
const START_PITCH = 0.35;
const PITCH_LIMIT = 1.45;
const DRAG_SPEED = 0.01;
/** Radians per second when turning on its own. */
const AUTO_TURN_SPEED = 0.35;
const MS_PER_SECOND = 1000;
/** Warcraft III models are Z-up. */
const UP = vec3.fromValues(0, 0, 1);
const PLAYER_RED = vec3.fromValues(1, 0, 0);
/** The light sits this many model radiuses above the camera, so the side in view is lit. */
const LIGHT_HEIGHT = 1.5;
/**
 * The HD shader divides the base color by pi and, without reflection maps,
 * adds almost no ambient light; at intensity 1 models show up nearly black.
 * Around 15 matches the game's look; higher washes out light colors.
 */
const LIGHT_INTENSITY = 15;
const LIGHT_COLOR = vec3.fromValues(LIGHT_INTENSITY, LIGHT_INTENSITY, LIGHT_INTENSITY);
/** The page's darkest surface color (#090d1a) as GL color channels. */
const BACKGROUND_RED = 0.035;
const BACKGROUND_GREEN = 0.05;
const BACKGROUND_BLUE = 0.1;
const STAND_PREFIX = "stand";

function resetStatus(path: string | null): Status {
  if (path) {
    return { kind: "loading" };
  }

  return { kind: "empty" };
}

/**
 * Reflection maps: loading one makes the renderer pre-process it with a shader
 * that fails on some graphics drivers. A preview does without reflections.
 */
const ENVIRONMENT_MAP = /environmentmap/i;

/** Uploads every texture the model names; a missing one only leaves that part untextured. */
async function loadTextures(renderer: ModelRenderer, model: Model): Promise<number> {
  let missing = 0;
  const images = model.Textures.map((t) => t.Image).filter(
    (image) => image && !ENVIRONMENT_MAP.test(image),
  );

  await Promise.all(
    images.map(async (image) => {
      try {
        const response = await fetch(convertFileSrc(image, TEXTURE_SCHEME));
        if (!response.ok) {
          missing += 1;
          return;
        }

        const buffer = await response.arrayBuffer();
        const header = new DataView(buffer, 0, TEXTURE_HEADER_BYTES);
        const width = header.getUint32(0, true);
        const height = header.getUint32(BYTES_PER_PIXEL, true);
        const pixels = new Uint8ClampedArray(
          buffer,
          TEXTURE_HEADER_BYTES,
          width * height * BYTES_PER_PIXEL,
        );
        renderer.setTextureImageData(image, [new ImageData(pixels, width, height)]);
      } catch {
        missing += 1;
      }
    }),
  );

  return missing;
}

const MEDIUM_PRECISION = /precision mediump float/g;
const HIGH_PRECISION = "precision highp float";

/**
 * Compiles every shader with full float precision. war3-model's shaders ask
 * for medium precision, which the Linux webview runs as 16-bit floats: the HD
 * shader's light and view vectors overflow at model scale and HD models
 * render nearly black. WebGL2 always supports high precision.
 */
function forceFullPrecision(gl: WebGL2RenderingContext) {
  const shaderSource = gl.shaderSource.bind(gl);
  gl.shaderSource = (shader, source) =>
    shaderSource(shader, source.replace(MEDIUM_PRECISION, HIGH_PRECISION));
}

/**
 * Frees the renderer's GL objects. For SD models, war3-model's `destroy` throws
 * at its very end, where it frees reflection shaders that it only creates for
 * HD models; everything this renderer did create is freed by then.
 */
function destroyRenderer(renderer: ModelRenderer) {
  try {
    renderer.destroy();
  } catch {
    // Only the reflection shaders that were never created are left.
  }
}

type Bounds = { center: vec3; radius: number };

const COORDS_PER_VERTEX = 3;
const HALFWAY = 0.5;

/**
 * The box around the full-detail meshes' vertices, as a center and half its
 * diagonal. The extents stored in game models do not fit the mesh: the bounds
 * radius is 0, geoset boxes are empty, and the model box is about twice the size.
 */
function bounds(model: Model): Bounds {
  const min = vec3.fromValues(Infinity, Infinity, Infinity);
  const max = vec3.fromValues(-Infinity, -Infinity, -Infinity);
  const point = vec3.create();

  for (const geoset of model.Geosets) {
    if (geoset.LevelOfDetail) {
      continue;
    }

    const vertices = geoset.Vertices;
    for (let i = 0; i + COORDS_PER_VERTEX <= vertices.length; i += COORDS_PER_VERTEX) {
      const [x = 0, y = 0, z = 0] = vertices.subarray(i, i + COORDS_PER_VERTEX);
      vec3.set(point, x, y, z);
      vec3.min(min, min, point);
      vec3.max(max, max, point);
    }
  }

  if (min[0] === Infinity) {
    return { center: vec3.create(), radius: 1 };
  }

  const center = vec3.lerp(vec3.create(), min, max, HALFWAY);
  return { center, radius: Math.max(vec3.distance(min, max) * HALFWAY, 1) };
}

/**
 * A turntable view of a unit's model, drawn with war3-model (WebGL). Drag to
 * rotate, scroll to zoom, pick an animation below.
 */
export function ModelViewer({ path }: ModelViewerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rendererRef = useRef<ModelRenderer | null>(null);
  const view = useRef({
    yaw: 0,
    pitch: START_PITCH,
    distance: START_DISTANCE,
    dragging: false,
    lastX: 0,
    lastY: 0,
  });
  const [status, setStatus] = useState<Status>(resetStatus(path));
  const [sequences, setSequences] = useState<Animation[]>([]);
  const [sequence, setSequence] = useState(0);
  // Turning is motion the viewer did not ask for: off when the system asks for less.
  const [turning, setTurning] = useState(
    () => !window.matchMedia("(prefers-reduced-motion: reduce)").matches,
  );
  const [missingTextures, setMissingTextures] = useState(0);
  const turningRef = useRef(turning);
  turningRef.current = turning;

  useEffect(() => {
    setStatus(resetStatus(path));
    setSequences([]);
    setMissingTextures(0);
    const canvas = canvasRef.current;
    if (!path || !canvas) {
      return;
    }

    // WebGL2 for the newer HD shader. Reflection maps are never loaded (see
    // `ENVIRONMENT_MAP`), so the renderer's reflection pre-processing never runs.
    const gl = canvas.getContext("webgl2");
    if (!gl) {
      setStatus({ kind: "error", message: "WebGL is not available in this window." });
      return;
    }

    forceFullPrecision(gl);
    let cancelled = false;
    let frame = 0;

    async function load(
      context: WebGL2RenderingContext | WebGLRenderingContext,
      modelPath: string,
    ) {
      const response = await fetch(convertFileSrc(modelPath, MODEL_SCHEME));
      if (!response.ok) {
        throw new Error(await response.text());
      }

      const model = parseMDX(await response.arrayBuffer());
      const renderer = new ModelRenderer(model);
      renderer.initGL(context);
      renderer.setTeamColor(PLAYER_RED);
      renderer.setLightColor(LIGHT_COLOR);

      const animations = model.Sequences.map((s, index) => ({
        index,
        name: s.Name,
        start: s.Interval[0] ?? index,
      }));
      const stand = animations.findIndex((a) => a.name.toLowerCase().startsWith(STAND_PREFIX));
      const first = Math.max(stand, 0);
      if (animations.length > 0) {
        renderer.setSequence(first);
      }

      const missing = await loadTextures(renderer, model);
      if (cancelled) {
        destroyRenderer(renderer);
        return;
      }

      rendererRef.current = renderer;
      setSequences(animations);
      setSequence(first);
      setMissingTextures(missing);
      setStatus({ kind: "ready" });
      view.current.distance = START_DISTANCE;

      const { center: target, radius: size } = bounds(model);
      const eye = vec3.create();
      const light = vec3.create();
      const modelView = mat4.create();
      const projection = mat4.create();
      const inverse = mat4.create();
      const rotation = quat.create();
      let last = performance.now();

      const draw = (now: number) => {
        const delta = now - last;
        last = now;
        const v = view.current;
        if (turningRef.current && !v.dragging) {
          v.yaw += (AUTO_TURN_SPEED * delta) / MS_PER_SECOND;
        }

        const distance = size * v.distance;
        vec3.set(
          eye,
          target[0] + distance * Math.cos(v.pitch) * Math.cos(v.yaw),
          target[1] + distance * Math.cos(v.pitch) * Math.sin(v.yaw),
          target[2] + distance * Math.sin(v.pitch),
        );
        vec3.scaleAndAdd(light, eye, UP, size * LIGHT_HEIGHT);
        mat4.lookAt(modelView, eye, target, UP);
        mat4.perspective(
          projection,
          FIELD_OF_VIEW,
          context.canvas.width / context.canvas.height,
          NEAR_PLANE,
          FAR_PLANE,
        );
        mat4.invert(inverse, modelView);
        mat4.getRotation(rotation, inverse);

        context.viewport(0, 0, context.canvas.width, context.canvas.height);
        context.clearColor(BACKGROUND_RED, BACKGROUND_GREEN, BACKGROUND_BLUE, 1);
        context.clear(context.COLOR_BUFFER_BIT | context.DEPTH_BUFFER_BIT);
        context.enable(context.DEPTH_TEST);

        renderer.setCamera(eye, rotation);
        renderer.setLightPosition(light);
        renderer.update(delta);
        renderer.render(modelView, projection, { wireframe: false, useEnvironmentMap: false });
        frame = window.requestAnimationFrame(draw);
      };

      frame = window.requestAnimationFrame(draw);
    }

    load(gl, path).catch((e: unknown) => {
      if (!cancelled) {
        setStatus({ kind: "error", message: `Could not show this model: ${errorMessage(e)}` });
      }
    });

    return () => {
      cancelled = true;
      window.cancelAnimationFrame(frame);
      if (rendererRef.current) {
        destroyRenderer(rendererRef.current);
      }
      rendererRef.current = null;
    };
  }, [path]);

  function pickSequence(index: number) {
    setSequence(index);
    rendererRef.current?.setSequence(index);
  }

  function pointerDown(e: PointerEvent<HTMLCanvasElement>) {
    e.currentTarget.setPointerCapture(e.pointerId);
    view.current.dragging = true;
    view.current.lastX = e.clientX;
    view.current.lastY = e.clientY;
  }

  function pointerMove(e: PointerEvent<HTMLCanvasElement>) {
    const v = view.current;
    if (!v.dragging) {
      return;
    }

    v.yaw -= (e.clientX - v.lastX) * DRAG_SPEED;
    v.pitch = Math.min(
      PITCH_LIMIT,
      Math.max(-PITCH_LIMIT, v.pitch + (e.clientY - v.lastY) * DRAG_SPEED),
    );
    v.lastX = e.clientX;
    v.lastY = e.clientY;
  }

  function wheel(e: WheelEvent<HTMLCanvasElement>) {
    const v = view.current;
    v.distance = Math.min(
      MAX_DISTANCE,
      Math.max(MIN_DISTANCE, v.distance * (1 + e.deltaY * ZOOM_STEP)),
    );
  }

  return (
    <div className="flex w-72 shrink-0 flex-col gap-1.5">
      <div className="relative aspect-square overflow-hidden rounded-[3px] border border-line bg-surface-0">
        <canvas
          ref={canvasRef}
          width={576}
          height={576}
          onPointerDown={pointerDown}
          onPointerMove={pointerMove}
          onPointerUp={() => {
            view.current.dragging = false;
          }}
          onWheel={wheel}
          aria-label="3D model preview. Drag to rotate, scroll to zoom."
          className="size-full cursor-grab touch-none active:cursor-grabbing"
        />
        {status.kind === "loading" && (
          <p className="absolute inset-0 flex items-center justify-center text-sm text-muted">
            Loading model…
          </p>
        )}
        {status.kind === "empty" && (
          <p className="absolute inset-0 flex items-center justify-center text-sm text-faint">
            No model
          </p>
        )}
        {status.kind === "error" && (
          <p className="absolute inset-0 flex items-center justify-center p-4 text-center text-sm text-danger">
            {status.message}
          </p>
        )}
      </div>
      {status.kind === "ready" && (
        <div className="flex items-center gap-2 text-xs">
          <select
            value={sequence}
            onChange={(e) => pickSequence(Number(e.target.value))}
            aria-label="Animation"
            className="min-w-0 flex-1 rounded border border-field bg-well px-1.5 py-1 outline-none focus:border-glow"
          >
            {sequences.map((animation) => (
              <option key={`${animation.start}-${animation.name}`} value={animation.index}>
                {animation.name}
              </option>
            ))}
          </select>
          <label className="flex items-center gap-1 text-muted">
            <input
              type="checkbox"
              checked={turning}
              onChange={(e) => setTurning(e.target.checked)}
            />
            Turn
          </label>
        </div>
      )}
      {missingTextures > 0 && (
        <p className="text-xs text-faint">{missingTextures} texture(s) could not be loaded.</p>
      )}
    </div>
  );
}
