import { ZoomIn, ZoomOut } from "@untitledui/icons";
import {
  type KeyboardEvent,
  type PointerEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import type { MapView, PlacedUnit, PlacementEdit } from "../types";
import { classes } from "../ui";
import { Button } from "./Button";
import { plainGameText } from "./GameText";

const PLAYER_COLORS = [
  "#ff0303",
  "#0042ff",
  "#1ce6b9",
  "#540081",
  "#fffc00",
  "#fe8a0e",
  "#20c000",
  "#e55bb0",
  "#959697",
  "#7ebff1",
  "#106246",
  "#4e2a04",
  "#9b0000",
  "#0000c3",
  "#00eaff",
  "#be00fe",
  "#ebcd87",
  "#f8a48b",
  "#bfff80",
  "#dcb9eb",
  "#282828",
  "#ebf0ff",
  "#00781e",
  "#a46f33",
];
const NEUTRAL_COLOR = "#1a1a1a";
const UNIT_RADIUS_PX = 5;
const HERO_RADIUS_PX = 7;
const BUILDING_HALF_PX = 6;
const FACING_PX = 12;
const HANDLE_PX = 30;
const HANDLE_RADIUS_PX = 5;
const SELECTED_RING_PX = 4;
const DASH_PX = 2;
const FIT_MARGIN = 0.95;
const MIN_ZOOM_OF_FIT = 0.5;
const MAX_ZOOM = 4;
const ZOOM_STEP = 1.25;
const WHEEL_ZOOM = 0.0015;
const DRAG_THRESHOLD_PX = 3;
const NUDGE = 32;
const BIG_NUDGE = 128;
const TURN_STEP = 15;
const FULL_TURN = 360;
const HALF_TURN = 180;

type Camera = { x: number; y: number; zoom: number };

type Size = { width: number; height: number };

type Drag =
  | { kind: "pan"; startX: number; startY: number; camera: Camera }
  | { kind: "move"; index: number; startX: number; startY: number; moved: boolean }
  | { kind: "turn"; index: number };

type Preview = { index: number; x: number; y: number; facing: number };

export type Placing = { id: string; name: string };

type PlacedMapProps = {
  view: MapView;
  units: PlacedUnit[];
  selected: number | null;
  placing: Placing | null;
  onSelect: (index: number) => void;
  onMove: (index: number, x: number, y: number) => void;
  onTurn: (index: number, facing: number) => void;
  onPlace: (x: number, y: number) => void;
  onCancelPlace: () => void;
};

export function playerColor(player: number): string {
  return PLAYER_COLORS[player] ?? NEUTRAL_COLOR;
}

function fitCamera(view: MapView, size: Size): Camera {
  const [left, bottom, right, top] = view.bounds;
  const zoom = Math.min(size.width / (right - left), size.height / (top - bottom)) * FIT_MARGIN;

  return { x: (left + right) / 2, y: (bottom + top) / 2, zoom };
}

function clamp(value: number, low: number, high: number): number {
  return Math.min(high, Math.max(low, value));
}

function nudgeOf(big: boolean): number {
  if (big) {
    return BIG_NUDGE;
  }

  return NUDGE;
}

function turned(facing: number, left: boolean): number {
  if (left) {
    return (facing + TURN_STEP) % FULL_TURN;
  }

  return (facing - TURN_STEP + FULL_TURN) % FULL_TURN;
}

function radiusOf(unit: PlacedUnit): number {
  if (unit.hero) {
    return HERO_RADIUS_PX;
  }

  return UNIT_RADIUS_PX;
}

function dashOf(unit: PlacedUnit, length: number): string | undefined {
  if (unit.random) {
    return `${length} ${length}`;
  }

  return undefined;
}

function turnOf(dx: number, dy: number): number {
  const degrees = (Math.atan2(dy, dx) * HALF_TURN) / Math.PI;

  return Math.round((degrees + FULL_TURN) % FULL_TURN);
}

/** What a key does to the picked unit: move, turn or remove it. */
export function placementKey(
  event: KeyboardEvent,
  unit: PlacedUnit,
  view: MapView,
): PlacementEdit | "remove" | null {
  const [left, bottom, right, top] = view.bounds;
  const step = nudgeOf(event.shiftKey);
  const moves: Record<string, [number, number]> = {
    ArrowLeft: [-step, 0],
    ArrowRight: [step, 0],
    ArrowUp: [0, step],
    ArrowDown: [0, -step],
  };
  const move = moves[event.key];

  if (unit.random) {
    return null;
  }

  if (move) {
    return {
      x: clamp(Math.round(unit.x + move[0]), left, right),
      y: clamp(Math.round(unit.y + move[1]), bottom, top),
    };
  }

  if (event.key === "[" || event.key === "]") {
    return { facing: turned(unit.facing, event.key === "[") };
  }

  if (event.key === "Delete") {
    return "remove";
  }

  return null;
}

export function PlacedMap({
  view,
  units,
  selected,
  placing,
  onSelect,
  onMove,
  onTurn,
  onPlace,
  onCancelPlace,
}: PlacedMapProps) {
  const box = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState<Size | null>(null);
  const [camera, setCamera] = useState<Camera | null>(null);
  const [drag, setDrag] = useState<Drag | null>(null);
  const [preview, setPreview] = useState<Preview | null>(null);
  const [pointer, setPointer] = useState<{ x: number; y: number } | null>(null);
  const [left, bottom, right, top] = view.bounds;

  useEffect(() => {
    const element = box.current;

    if (!element) {
      return;
    }

    const observer = new ResizeObserver(([entry]) => {
      if (entry) {
        setSize({ width: entry.contentRect.width, height: entry.contentRect.height });
      }
    });

    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (size && !camera) {
      setCamera(fitCamera(view, size));
    }
  }, [size, camera, view]);

  useEffect(() => {
    if (units) {
      setPreview(null);
    }
  }, [units]);

  const limits = useCallback(
    (zoom: number) => {
      if (!size) {
        return zoom;
      }

      return clamp(zoom, fitCamera(view, size).zoom * MIN_ZOOM_OF_FIT, MAX_ZOOM);
    },
    [size, view],
  );

  const toGame = useCallback(
    (clientX: number, clientY: number) => {
      const rect = box.current?.getBoundingClientRect();

      if (!rect || !camera) {
        return null;
      }

      return {
        x: camera.x + (clientX - rect.left - rect.width / 2) / camera.zoom,
        y: camera.y - (clientY - rect.top - rect.height / 2) / camera.zoom,
      };
    },
    [camera],
  );

  useEffect(() => {
    const element = box.current;

    if (!element) {
      return;
    }

    function wheel(event: WheelEvent) {
      event.preventDefault();
      const at = toGame(event.clientX, event.clientY);

      setCamera((current) => {
        if (!current || !at) {
          return current;
        }

        const zoom = limits(current.zoom * Math.exp(-event.deltaY * WHEEL_ZOOM));
        const ratio = current.zoom / zoom;

        return {
          x: at.x - (at.x - current.x) * ratio,
          y: at.y - (at.y - current.y) * ratio,
          zoom,
        };
      });
    }

    element.addEventListener("wheel", wheel, { passive: false });
    return () => element.removeEventListener("wheel", wheel);
  }, [toGame, limits]);

  const selectedUnit = units.find((u) => u.index === selected);

  useEffect(() => {
    if (!selectedUnit || !size) {
      return;
    }

    setCamera((current) => {
      if (!current) {
        return current;
      }

      const halfWidth = size.width / 2 / current.zoom;
      const halfHeight = size.height / 2 / current.zoom;
      const visible =
        Math.abs(selectedUnit.x - current.x) < halfWidth &&
        Math.abs(selectedUnit.y - current.y) < halfHeight;

      if (visible) {
        return current;
      }

      return { ...current, x: selectedUnit.x, y: selectedUnit.y };
    });
  }, [selectedUnit, size]);

  function zoomBy(factor: number) {
    setCamera((current) => current && { ...current, zoom: limits(current.zoom * factor) });
  }

  function fit() {
    if (size) {
      setCamera(fitCamera(view, size));
    }
  }

  function inside(x: number, y: number) {
    return { x: Math.round(clamp(x, left, right)), y: Math.round(clamp(y, bottom, top)) };
  }

  function positionOf(unit: PlacedUnit) {
    if (preview && preview.index === unit.index) {
      return preview;
    }

    return { x: unit.x, y: unit.y, facing: unit.facing };
  }

  function pressGround(event: PointerEvent) {
    if (event.button !== 0 || !camera) {
      return;
    }

    const at = toGame(event.clientX, event.clientY);

    if (placing && at) {
      const spot = inside(at.x, at.y);

      onPlace(spot.x, spot.y);
      return;
    }

    event.currentTarget.setPointerCapture(event.pointerId);
    setDrag({ kind: "pan", startX: event.clientX, startY: event.clientY, camera });
  }

  function pressUnit(event: PointerEvent, unit: PlacedUnit) {
    if (event.button !== 0 || placing) {
      return;
    }

    event.stopPropagation();
    event.preventDefault();
    box.current?.setPointerCapture(event.pointerId);
    onSelect(unit.index);

    if (!unit.random) {
      setDrag({
        kind: "move",
        index: unit.index,
        startX: event.clientX,
        startY: event.clientY,
        moved: false,
      });
    }
  }

  function pressHandle(event: PointerEvent, unit: PlacedUnit) {
    if (event.button !== 0) {
      return;
    }

    event.stopPropagation();
    event.preventDefault();
    box.current?.setPointerCapture(event.pointerId);
    setDrag({ kind: "turn", index: unit.index });
  }

  function movePointer(event: PointerEvent) {
    const at = toGame(event.clientX, event.clientY);

    setPointer(at && inside(at.x, at.y));

    if (!drag || !at) {
      return;
    }

    if (drag.kind === "pan") {
      setCamera({
        ...drag.camera,
        x: drag.camera.x - (event.clientX - drag.startX) / drag.camera.zoom,
        y: drag.camera.y + (event.clientY - drag.startY) / drag.camera.zoom,
      });

      return;
    }

    const unit = units.find((u) => u.index === drag.index);

    if (!unit) {
      return;
    }

    if (drag.kind === "turn") {
      const from = positionOf(unit);

      setPreview({ ...from, index: unit.index, facing: turnOf(at.x - from.x, at.y - from.y) });
      return;
    }

    const far =
      Math.hypot(event.clientX - drag.startX, event.clientY - drag.startY) > DRAG_THRESHOLD_PX;

    if (drag.moved || far) {
      setDrag({ ...drag, moved: true });
      setPreview({ index: unit.index, ...inside(at.x, at.y), facing: unit.facing });
    }
  }

  function release() {
    if (drag?.kind === "move" && drag.moved && preview) {
      onMove(drag.index, preview.x, preview.y);
    }

    if (drag?.kind === "turn" && preview) {
      onTurn(drag.index, preview.facing);
    }

    setDrag(null);
  }

  useEffect(() => {
    if (!placing) {
      return;
    }

    function stopOnEscape(event: globalThis.KeyboardEvent) {
      if (event.key === "Escape") {
        onCancelPlace();
      }
    }

    window.addEventListener("keydown", stopOnEscape);
    return () => window.removeEventListener("keydown", stopOnEscape);
  }, [placing, onCancelPlace]);

  const scale = camera?.zoom ?? 1;
  const px = (pixels: number) => pixels / scale;
  const viewBox =
    camera &&
    size &&
    [
      camera.x - size.width / 2 / scale,
      -camera.y - size.height / 2 / scale,
      size.width / scale,
      size.height / scale,
    ].join(" ");

  function marker(unit: PlacedUnit) {
    const at = positionOf(unit);
    const angle = (at.facing * Math.PI) / HALF_TURN;
    const radius = px(radiusOf(unit));
    const half = px(BUILDING_HALF_PX);
    const isSelected = unit.index === selected;
    const fill = playerColor(unit.player);
    const dash = dashOf(unit, px(DASH_PX));
    const label = `${plainGameText(unit.name)}, ${unit.playerName}`;

    return (
      <g
        key={unit.index}
        onPointerDown={(e) => pressUnit(e, unit)}
        className={classes(unit.random && "cursor-pointer", !unit.random && "cursor-move")}
      >
        <title>{label}</title>
        {!unit.building && (
          <line
            x1={at.x}
            y1={-at.y}
            x2={at.x + Math.cos(angle) * px(FACING_PX)}
            y2={-at.y - Math.sin(angle) * px(FACING_PX)}
            stroke="var(--color-ink)"
            strokeWidth={1.5}
            vectorEffect="non-scaling-stroke"
          />
        )}
        {unit.building && (
          <rect
            x={at.x - half}
            y={-at.y - half}
            width={half * 2}
            height={half * 2}
            fill={fill}
            stroke="var(--color-ink)"
            strokeDasharray={dash}
            vectorEffect="non-scaling-stroke"
          />
        )}
        {!unit.building && (
          <circle
            cx={at.x}
            cy={-at.y}
            r={radius}
            fill={fill}
            stroke="var(--color-ink)"
            strokeDasharray={dash}
            vectorEffect="non-scaling-stroke"
          />
        )}
        {isSelected && (
          <circle
            cx={at.x}
            cy={-at.y}
            r={Math.max(radius, half) + px(SELECTED_RING_PX)}
            fill="none"
            stroke="var(--color-glow)"
            strokeWidth={2}
            vectorEffect="non-scaling-stroke"
          />
        )}
      </g>
    );
  }

  function handle(unit: PlacedUnit) {
    const at = positionOf(unit);
    const angle = (at.facing * Math.PI) / HALF_TURN;
    const x = at.x + Math.cos(angle) * px(HANDLE_PX);
    const y = -at.y - Math.sin(angle) * px(HANDLE_PX);

    return (
      <g onPointerDown={(e) => pressHandle(e, unit)} className="cursor-grab">
        <title>Drag to turn</title>
        <line
          pointerEvents="none"
          x1={at.x}
          y1={-at.y}
          x2={x}
          y2={y}
          stroke="var(--color-glow)"
          strokeDasharray="3 3"
          vectorEffect="non-scaling-stroke"
        />
        <circle
          cx={x}
          cy={y}
          r={px(HANDLE_RADIUS_PX)}
          fill="var(--color-surface-1)"
          stroke="var(--color-glow)"
          strokeWidth={2}
          vectorEffect="non-scaling-stroke"
        />
      </g>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-2 border-b border-line px-3 py-2 text-xs text-faint">
        <span className="min-w-0 flex-1 truncate">
          {placing &&
            `Click the map to place ${plainGameText(placing.name)}. Escape stops placing.`}
          {!placing &&
            "Drag to pan, scroll to zoom. Drag a unit to move it, or its handle to turn it."}
        </span>
        <span className="w-24 shrink-0 text-right font-mono">
          {pointer && `${pointer.x}, ${pointer.y}`}
        </span>
        <Button small label="Zoom in" onPress={() => zoomBy(ZOOM_STEP)}>
          <ZoomIn className="size-4" aria-hidden="true" />
        </Button>
        <Button small label="Zoom out" onPress={() => zoomBy(1 / ZOOM_STEP)}>
          <ZoomOut className="size-4" aria-hidden="true" />
        </Button>
        <Button small onPress={fit}>
          Whole map
        </Button>
      </div>
      <div
        ref={box}
        onPointerDown={pressGround}
        onPointerMove={movePointer}
        onPointerUp={release}
        onPointerLeave={() => setPointer(null)}
        className={classes(
          "relative min-h-0 flex-1 overflow-hidden bg-surface-0",
          placing && "cursor-crosshair",
          !placing && "cursor-grab",
        )}
      >
        {viewBox && size && (
          <svg
            width={size.width}
            height={size.height}
            viewBox={viewBox}
            className="absolute inset-0 select-none"
            aria-hidden="true"
          >
            <image
              href={view.image}
              x={left}
              y={-top}
              width={right - left}
              height={top - bottom}
              preserveAspectRatio="none"
            />
            {units.map(marker)}
            {selectedUnit && !selectedUnit.random && !selectedUnit.building && handle(selectedUnit)}
          </svg>
        )}
      </div>
    </div>
  );
}
