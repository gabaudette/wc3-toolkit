import { createContext } from "react";

/** Shows an ability on the Abilities tab; `null` where there is nowhere to go. */
export const OpenAbilityContext = createContext<((id: string) => void) | null>(null);
