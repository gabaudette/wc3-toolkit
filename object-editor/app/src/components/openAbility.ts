import { createContext } from "react";

export const OpenAbilityContext = createContext<((id: string) => void) | null>(null);
