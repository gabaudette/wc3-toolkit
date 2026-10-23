export type Parameter = { name: string; type: string };
export type Declaration = {
  name: string;
  kind: "native" | "function" | "type" | "constant";
  source: string;
  line: number;
  parameters: Parameter[];
  returns: string;
};

// Only declarations are retained. Game implementations and comments stay local.
export function parseJass(text: string, source: string): Declaration[] {
  const declarations: Declaration[] = [];
  const names = new Set<string>();
  let inFunction = false;
  let inGlobals = false;
  const lines = text.replace(/^\uFEFF/, "").split(/\r?\n/);

  for (const [index, raw] of lines.entries()) {
    const line = raw.replace(/\/\/.*$/, "").trim();
    if (line === "endfunction") {
      inFunction = false;
      continue;
    }
    if (inFunction || !line) {
      continue;
    }
    if (line === "globals" || line === "endglobals") {
      inGlobals = line === "globals";
      continue;
    }

    const location = index + 1;
    let declaration: Declaration | undefined;
    const type = /^type\s+(\w+)\s+extends\s+(\w+)$/.exec(line);
    const constant = /^constant\s+(\w+)\s+(\w+)\s*=/.exec(line);
    const callable =
      /^(?:constant\s+)?(native|function)\s+(\w+)\s+takes\s+(.+?)\s+returns\s+(\w+)$/.exec(line);

    if (type) {
      declaration = {
        name: type[1],
        kind: "type",
        source,
        line: location,
        parameters: [],
        returns: type[2],
      };
    } else if (inGlobals && constant) {
      declaration = {
        name: constant[2],
        kind: "constant",
        source,
        line: location,
        parameters: [],
        returns: constant[1],
      };
    } else if (callable) {
      const parameters: Parameter[] = [];
      if (callable[3] !== "nothing") {
        for (const rawParameter of callable[3].split(",")) {
          const parameter = /^(\w+)\s+(\w+)$/.exec(rawParameter.trim());
          if (!parameter) {
            throw new Error(`${source}:${location}: invalid parameter ${rawParameter}`);
          }
          parameters.push({ type: parameter[1], name: parameter[2] });
        }
      }
      const kind = callable[1];
      if (kind !== "native" && kind !== "function") {
        throw new Error(`${source}:${location}: invalid callable kind`);
      }
      declaration = {
        name: callable[2],
        kind,
        source,
        line: location,
        parameters,
        returns: callable[4],
      };
      inFunction = kind === "function";
    } else if (/^(?:constant\s+)?(?:native|function|type)\s/.test(line)) {
      throw new Error(`${source}:${location}: unsupported declaration: ${line}`);
    }

    if (declaration) {
      if (names.has(declaration.name)) {
        throw new Error(`${source}:${location}: duplicate declaration ${declaration.name}`);
      }
      names.add(declaration.name);
      declarations.push(declaration);
    }
  }

  if (inFunction || inGlobals) {
    throw new Error(`${source}: unterminated function or globals block`);
  }
  if (declarations.length === 0) {
    throw new Error(`${source}: no declarations found`);
  }
  return declarations;
}

export function signatures(declaration: Declaration): { jass: string; lua: string } {
  const { name, kind, parameters, returns } = declaration;
  if (kind === "type") {
    return {
      jass: `type ${name} extends ${returns}`,
      lua: `-- ${name}: game handle derived from ${returns}`,
    };
  }
  if (kind === "constant") {
    return { jass: `constant ${returns} ${name}`, lua: `-- ${name}: ${returns}` };
  }

  let takes = "nothing";
  if (parameters.length > 0) {
    takes = parameters.map((parameter) => `${parameter.type} ${parameter.name}`).join(", ");
  }
  return {
    jass: `${kind} ${name} takes ${takes} returns ${returns}`,
    lua: `${name}(${parameters.map((parameter) => parameter.name).join(", ")})`,
  };
}
