import { useEffect, useState } from "react";
import { api } from "../api";
import { choosePath, chooseSavePath } from "../dialogs";
import { settle } from "../result";
import type { CsvValues, ExportFormat, ExportOrigin, ExportResult, ObjectType } from "../types";
import { counted } from "../ui";
import { Button } from "./Button";
import { Checkbox } from "./Checkbox";
import { DialogShell } from "./DialogShell";
import { type ExportScope, OBJECT_TYPES, shownCount } from "./exportObjects";
import { Select } from "./Select";

type Which = "shown" | "selected" | "type" | "every";

type ExportDialogProps = {
  scope: ExportScope | null;
  mapName: string | null;
  onClose: () => void;
  onError: (message: string | null) => void;
};

const GAME_FILE_NAME = "game-objects";
const MAP_EXTENSION = /\.w3[mx]$/i;

const FORMATS: { value: ExportFormat; label: string }[] = [
  { value: "csv", label: "CSV: a table per object type" },
  { value: "json", label: "JSON: one file for tools" },
];
const ORIGINS: { value: ExportOrigin; label: string }[] = [
  { value: "all", label: "All of them, with this map's changes" },
  { value: "changed", label: "Only what this map adds or changes" },
  { value: "custom", label: "Only custom objects this map adds" },
];
const CELL_VALUES: { value: CsvValues; label: string }[] = [
  { value: "display", label: "Values as the editor shows them" },
  { value: "raw", label: "Values as stored in the data" },
];

function chosen<T extends string>(options: { value: T }[], value: string, current: T): T {
  return options.find((o) => o.value === value)?.value ?? current;
}

function firstChoice(scope: ExportScope): Which {
  if (scope.type && scope.shown.length > 0) {
    return "shown";
  }

  if (scope.type) {
    return "type";
  }

  return "every";
}

function choices(scope: ExportScope): { value: Which; label: string }[] {
  const list: { value: Which; label: string }[] = [];
  const type = scope.type;

  if (type && scope.shown.length > 0) {
    list.push({
      value: "shown",
      label: `The ${shownCount(scope.shown.length, type)} the list shows`,
    });
  }

  if (type && scope.selected) {
    list.push({ value: "selected", label: `Only ${scope.selected.name} (${scope.selected.id})` });
  }

  if (type) {
    list.push({ value: "type", label: `All ${type}` });
  }

  list.push({ value: "every", label: "Every object type" });
  return list;
}

function idsFor(which: Which, scope: ExportScope): string[] | null {
  if (which === "shown") {
    return scope.shown;
  }

  if (which === "selected" && scope.selected) {
    return [scope.selected.id];
  }

  return null;
}

function typesFor(which: Which, scope: ExportScope): ObjectType[] {
  if (which === "every" || !scope.type) {
    return OBJECT_TYPES.map((t) => t.type);
  }

  return [scope.type];
}

function fileStem(name: string, types: ObjectType[]): string {
  const [only] = types;

  if (types.length === 1 && only) {
    return `${name}-${only}`;
  }

  return name;
}

function baseName(mapName: string | null): string {
  if (!mapName) {
    return GAME_FILE_NAME;
  }

  return mapName.replace(MAP_EXTENSION, "");
}

export function ExportDialog({ scope, mapName, onClose, onError }: ExportDialogProps) {
  const [which, setWhich] = useState<Which>("every");
  const [format, setFormat] = useState<ExportFormat>("csv");
  const [values, setValues] = useState<CsvValues>("display");
  const [origin, setOrigin] = useState<ExportOrigin>("all");
  const [plainText, setPlainText] = useState(false);
  const [busy, setBusy] = useState(false);
  const [done, setDone] = useState<ExportResult | null>(null);

  useEffect(() => {
    if (scope) {
      setWhich(firstChoice(scope));
      setDone(null);
    }
  }, [scope]);

  if (!scope) {
    return null;
  }

  const types = typesFor(which, scope);
  const folder = format === "csv" && types.length > 1;

  async function pickPath(): Promise<string | null> {
    const name = baseName(mapName);

    if (folder) {
      const picked = await choosePath({
        title: "Choose a folder for the CSV files",
        directory: true,
      });

      if (!picked.ok) {
        onError(picked.error);
        return null;
      }

      return picked.value;
    }

    const picked = await chooseSavePath({
      title: "Export objects",
      defaultPath: `${fileStem(name, types)}.${format}`,
      filters: [{ name: format.toUpperCase(), extensions: [format] }],
    });

    if (!picked.ok) {
      onError(picked.error);
      return null;
    }

    return picked.value;
  }

  async function run() {
    if (!scope) {
      return;
    }

    const path = await pickPath();

    if (!path) {
      return;
    }

    setBusy(true);
    await settle(
      api.exportObjects({
        selection: { types, ids: idsFor(which, scope), origin, plainText },
        format,
        values,
        path,
      }),
      (result) => {
        onError(null);
        setDone(result);
      },
      onError,
    );
    setBusy(false);
  }

  return (
    <DialogShell
      open
      title="Export objects"
      onClose={onClose}
      footer={
        <>
          <Button onPress={onClose}>
            {done && "Close"}
            {!done && "Cancel"}
          </Button>
          <Button primary disabled={busy} onPress={() => void run()}>
            {busy && "Exporting…"}
            {!busy && "Export…"}
          </Button>
        </>
      }
    >
      <p className="text-sm text-muted">
        Every field of the chosen objects, as stored and as the editor shows it
        {mapName && `, with the changes in ${mapName}`}. Spreadsheets open CSV; other tools and
        sites read JSON.
      </p>
      <Select
        label="Objects"
        value={which}
        onChange={(value) => setWhich(chosen(choices(scope), value, which))}
        options={choices(scope)}
      />
      {mapName && (
        <Select
          label="From this map"
          value={origin}
          onChange={(value) => setOrigin(chosen(ORIGINS, value, origin))}
          options={ORIGINS}
        />
      )}
      <Select
        label="Format"
        value={format}
        onChange={(value) => setFormat(chosen(FORMATS, value, format))}
        options={FORMATS}
      />
      {format === "csv" && (
        <Select
          label="Cells hold"
          value={values}
          onChange={(value) => setValues(chosen(CELL_VALUES, value, values))}
          options={CELL_VALUES}
        />
      )}
      <Checkbox isSelected={plainText} onChange={setPlainText}>
        Remove color codes from text
      </Checkbox>
      {folder && (
        <p className="text-sm text-faint">
          Several object types as CSV make one file per type in the folder you choose.
        </p>
      )}
      <p role="status" className="text-sm text-gold-bright">
        {done &&
          `Exported ${counted(done.objects, "object")} to ${counted(done.files.length, "file")}.`}
      </p>
    </DialogShell>
  );
}
