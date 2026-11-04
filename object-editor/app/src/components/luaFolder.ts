import { createContext, type Dispatch, type SetStateAction, use, useState } from "react";

export type LuaFolder = {
  folder: string;
  setFolder: Dispatch<SetStateAction<string>>;
};

export const LuaFolderContext = createContext<LuaFolder | null>(null);

export function useLuaFolder(): LuaFolder {
  const shared = use(LuaFolderContext);
  const [folder, setFolder] = useState("");

  return shared ?? { folder, setFolder };
}
