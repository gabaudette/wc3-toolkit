import type { Declaration } from "./jass";

type Category = {
  id: string;
  title: string;
  description: string;
  names: RegExp;
  types: string[];
};

// Specific operations precede their argument types: saving a unit handle is storage,
// registering a unit event is a trigger, and adding an ability belongs to abilities.
export const API_CATEGORIES: Category[] = [
  {
    id: "storage-and-sync",
    title: "Storage and synchronization",
    description: "Hashtables, game caches, stored values and multiplayer data synchronization.",
    names:
      /Hashtable|GameCache|Stored|^Store|^RestoreUnit|^GetLastRestoredUnit|^Flush|^Sync|SyncData|SyncStart|SyncReady|^Blz(?:SendSync|GetTriggerSync)|^(?:Save|Load|HaveSaved|RemoveSaved)(?!Game|And)/i,
    types: ["hashtable", "gamecache"],
  },
  {
    id: "ai-and-melee",
    title: "AI and melee",
    description: "Computer opponents, melee initialization, guard positions and creep behavior.",
    names:
      /Melee|GuardPosition|CreepSleep|CreepCamp|Town|Captain|CommandAI|CampaignAI|AIDifficulty|Start.*AI$|AI_DIFFICULTY|^bj_MELEE/i,
    types: ["aidifficulty"],
  },
  {
    id: "triggers-and-events",
    title: "Triggers and events",
    description: "Event registration, conditions, actions, callbacks and event-response values.",
    names:
      /Trigger|^EVENT_|^Get(?:Event|Trigger|Entering|Leaving|Dying|Killing|Attacker|Attacked|Learning|Learned|Leveling|Reviving|Revivable|Rescuer|Rescued|Decaying|Constructing|Constructed|Cancelled|Researching|Researched|Trained|Detected|Summoning|Summoned|Transport|Loaded|Selling|Sold|Buying|Changing|Manipulat|Equipped|Unequipped|Ordered|Issued|OrderPoint|OrderTarget|Winning|Clicked|Spell)|^BlzGet(?:Event|Trigger|Equipped|Unequipped|Absorb|Manipulat|Stacking)|^(?:And|Or|Not|Condition|Filter|DestroyCondition|DestroyFilter|DestroyBoolExpr|ExecuteFunc)$/i,
    types: [
      "event",
      "trigger",
      "triggercondition",
      "triggeraction",
      "boolexpr",
      "conditionfunc",
      "filterfunc",
      "eventid",
      "gameevent",
      "playerevent",
      "playerunitevent",
      "unitevent",
      "widgetevent",
      "dialogevent",
      "limitop",
    ],
  },
  {
    id: "timers",
    title: "Timers and waits",
    description: "Timers, countdown dialogs, elapsed time and polling waits.",
    names: /Timer|PolledWait|WaitForCondition|WAIT_FOR_COND|POLLED_WAIT|TIMETYPE/i,
    types: ["timer", "timerdialog"],
  },
  {
    id: "groups-and-forces",
    title: "Groups and forces",
    description: "Unit groups, player forces, enumeration, filtering and collection orders.",
    names:
      /Group|^ForForce|Force(?:Add|Remove|Clear|Enum|Pick)|(?:Get|Count)Players(?:All|By|Allies|Enemies|Matching|InForce)|^GetForce|^Get(?:Enum|Filter)(?:Unit|Player)$/i,
    types: ["group", "force"],
  },
  {
    id: "quests",
    title: "Quests and objectives",
    description: "Quests, requirements, objective messages and defeat conditions.",
    names: /Quest|DefeatCondition|TEXT_DELAY_|QUEUE_DELAY_/i,
    types: ["quest", "questitem", "defeatcondition"],
  },
  {
    id: "camera-and-cinematics",
    title: "Camera and cinematics",
    description: "Camera movement, cinematic scenes, transmissions, fades and filters.",
    names:
      /Camera|Cinematic|CineMode|CineFade|CineFilter|CineScene|CinePortrait|Transmission|DialogueFrom|SmartPan|CancelCine/i,
    types: ["camerafield", "camerasetup"],
  },
  {
    id: "sound-and-music",
    title: "Sound and music",
    description: "Sound playback, music, volume groups, ambient audio and speech.",
    names: /Sound|Music|VolumeGroup|SPEECH_VOLUME|DawnDusk|DNCSounds/i,
    types: ["sound", "soundtype", "volumegroup"],
  },
  {
    id: "ui-and-input",
    title: "UI and input",
    description:
      "Frames, dialogs, scoreboards, floating text, selection, keyboard and mouse input.",
    names:
      /Frame|OSKEY|METAKEY|Mouse|KeyEvent|Dialog|Leaderboard|Multiboard|TextTag|Interface|ForceUI|UserControl|SelectUnit|SelectGroup|ClearSelection|Display.*Text|TextMessage|CommandButton|SORTTYPE|TextAlign|TEXT_JUSTIFY|HUDScale|SelectionEnabled|SelectionCircle|EnableUserUI|UIAutoPosition|LoadTOC|EnableCursor|^Blz(?:Get|Set)Mouse|EnableSelect|EnableDragSelect|EnablePreSelect/i,
    types: [
      "framehandle",
      "originframetype",
      "framepointtype",
      "textaligntype",
      "frameeventtype",
      "oskeytype",
      "metakeytype",
      "mousebuttontype",
      "dialog",
      "button",
      "leaderboard",
      "multiboard",
      "multiboarditem",
      "texttag",
      "trackable",
      "commandbuttoneffect",
    ],
  },
  {
    id: "abilities-and-upgrades",
    title: "Abilities, buffs and upgrades",
    description: "Ability fields and levels, buffs, auras, hero skills, research and technology.",
    names: /Ability|Buff|Aura|Spell|LearnSkill|HeroSkill|Tech|Research|REMOVEBUFFS/i,
    types: [
      "ability",
      "buff",
      "abilityintegerfield",
      "abilityrealfield",
      "abilitybooleanfield",
      "abilitystringfield",
      "abilityintegerlevelfield",
      "abilityreallevelfield",
      "abilitybooleanlevelfield",
      "abilitystringlevelfield",
      "abilityintegerlevelarrayfield",
      "abilityreallevelarrayfield",
      "abilitybooleanlevelarrayfield",
      "abilitystringlevelarrayfield",
    ],
  },
  {
    id: "items-and-inventory",
    title: "Items and inventory",
    description: "Items, charges, shops, loot pools, equipment and inventory slots.",
    names: /Item|Inventory|Equipment|Loadout|Purchased|Stock/i,
    types: [
      "item",
      "itempool",
      "itemtype",
      "itemintegerfield",
      "itemrealfield",
      "itembooleanfield",
      "itemstringfield",
      "equipmenttype",
      "itemtag",
      "loadoutslot",
    ],
  },
  {
    id: "combat",
    title: "Combat and damage",
    description: "Damage events and operations, attack types, weapons, armor and defense types.",
    names: /Damage|Attack|Armor|Defens|Weapon/i,
    types: [
      "attacktype",
      "damagetype",
      "weapontype",
      "armortype",
      "defensetype",
      "unitweaponintegerfield",
      "unitweaponrealfield",
      "unitweaponbooleanfield",
      "unitweaponstringfield",
    ],
  },
  {
    id: "effects-and-animation",
    title: "Effects and animation",
    description: "Special effects, lightning, images, splats, animation and visual indicators.",
    names:
      /Effect|Lightning|Ubersplat|Image|Anim|Indicator|HeroGlow|VertexColor|BLEND_MODE|TEXMAP_FLAG/i,
    types: [
      "effect",
      "effecttype",
      "lightning",
      "image",
      "ubersplat",
      "animtype",
      "subanimtype",
      "blendmode",
      "texmapflags",
    ],
  },
  {
    id: "destructables",
    title: "Destructables and gates",
    description: "Trees, destructable objects, gates, elevators and their properties.",
    names: /Destructable|Elevator|Gate|NudgeObjects/i,
    types: ["destructable"],
  },
  {
    id: "units-and-heroes",
    title: "Units and heroes",
    description: "Creation, movement, orders, hero statistics, unit fields, life and mana.",
    names:
      /Unit|Hero|Widget|Corpse|Order|ResourceAmount|Rally|Sleep|Acquisition|FoodUsed|FoodMade|Decay|Skeleton|Rescuable|RescueBuilding|REGENERATION|MOVEMENT_TYPE|TARGET_FLAG|^SetResourceAmount/i,
    types: [
      "unit",
      "widget",
      "unitpool",
      "unitstate",
      "unittype",
      "unitintegerfield",
      "unitrealfield",
      "unitbooleanfield",
      "unitstringfield",
      "movetype",
      "targetflag",
      "heroattribute",
      "regentype",
      "unitcategory",
      "raritycontrol",
    ],
  },
  {
    id: "players-and-alliances",
    title: "Players and alliances",
    description: "Player properties, teams, resources, races, alliances and handicaps.",
    names: /Player|Alliance|Handicap|Race|TaxRate|Team|NeutralVictim/i,
    types: [
      "player",
      "race",
      "alliancetype",
      "racepreference",
      "playerstate",
      "playerscore",
      "playergameresult",
      "playercolor",
      "playerslotstate",
    ],
  },
  {
    id: "terrain-and-environment",
    title: "Terrain and environment",
    description: "Terrain tiles, cliffs, weather, water, blight, pathing and day/night settings.",
    names:
      /Terrain|Weather|Water|Blight|Doodad|Pathing|Cliff|DayNight|TimeOfDay|TOD_|CellWidth|DNC|Dawn|Dusk|SkyModel|SkyBox|PortraitLight/i,
    types: ["weathereffect", "terraindeformation", "pathingtype", "pathingflag"],
  },
  {
    id: "visibility-and-minimap",
    title: "Visibility and minimap",
    description: "Fog of war, terrain fog, visibility modifiers, minimap icons and pings.",
    names:
      /Fog|Visibility|Visible|Vision|Occlusion|Masked|Fogged|Revealed|Minimap|MiniMapping|CampPing|PingMinimap/i,
    types: ["fogstate", "fogmodifier", "fogstyle", "minimapicon"],
  },
  {
    id: "geometry",
    title: "Locations, regions and geometry",
    description: "Points, locations, rectangles, regions, distances and angles.",
    names: /Location|Rect|Region|Point|Distance|Angle|Polar|Coord|WorldBounds|GetLoc|OffsetLoc/i,
    types: ["location", "region", "rect"],
  },
  {
    id: "game-and-campaign",
    title: "Game setup and campaigns",
    description: "Map settings, game flow, victories, save games, campaigns and missions.",
    names:
      /Map|Game|Campaign|Mission|Victory|Defeat|Tournament|Version|StartLoc|Density|InitBlizzard|InitSummon|InitNeutralBuildings|InitPlayerSlots|SlotAvail|ChangeLevel|SaveBasicFilename|SaveReplay|Tutorial|IntroShot|^SetNextLevel|^SetOpCinematic|^Reload|^Restart/i,
    types: [
      "gamestate",
      "igamestate",
      "fgamestate",
      "gamespeed",
      "gamedifficulty",
      "gametype",
      "mapflag",
      "mapvisibility",
      "mapsetting",
      "mapdensity",
      "mapcontrol",
      "placement",
      "startlocprio",
      "version",
    ],
  },
  {
    id: "math-and-strings",
    title: "Math, strings and conversions",
    description:
      "Arithmetic, trigonometry, random values, text, localization and raw-code conversion.",
    names:
      /Math|String|Integer|Real|Random|Convert|Id2|Localized|BlzBit|GetBoolean|^(?:[RI](?:Min|Max|Abs|Sign)BJ|Deg2Rad|Rad2Deg|Sin|Cos|Tan|Asin|Acos|Atan2?|SquareRoot|Pow|I2R|R2I|I2S|R2S|R2SW|S2I|S2R|Modulo|Percent|GetRandom)|^bj_(?:PI$|E$|RADTODEG|DEGTORAD|MODIFYMETHOD)/i,
    types: [],
  },
  {
    id: "runtime",
    title: "Runtime and general utilities",
    description: "Core handles, debugging, script runtime settings and general-purpose helpers.",
    names:
      /HandleId|Debug|Cheat|Preload|JASS_|FALSE|TRUE|ParseTags|Handle|Version|IsLastInstanceObject/i,
    types: ["agent", "handle"],
  },
];

const typeCategories = new Map(
  API_CATEGORIES.flatMap((category) => category.types.map((type) => [type, category] as const)),
);

const categoriesById = new Map(API_CATEGORIES.map((category) => [category.id, category]));

export function apiCategory(id: string): Category {
  const category = categoriesById.get(id);
  if (!category) {
    throw new Error(`Unknown API category: ${id}`);
  }
  return category;
}

/** Every symbol has one browse topic; source/kind and the original URL remain independent. */
export function categoryFor(declaration: Declaration): Category {
  const name = declaration.name;
  if (declaration.kind === "type") {
    const category = typeCategories.get(name.toLowerCase());
    if (category) {
      return category;
    }
  }
  if (declaration.kind === "constant" || name.startsWith("Convert")) {
    const category = typeCategories.get(declaration.returns.toLowerCase());
    if (category) {
      return category;
    }
  }
  // Handle storage wins even when a future API name mentions the stored object first.
  if (
    declaration.parameters.some((parameter) => ["hashtable", "gamecache"].includes(parameter.type))
  ) {
    return apiCategory("storage-and-sync");
  }
  if (/^(?:UnitId|AbilityId|OrderId)/.test(name)) {
    return apiCategory("math-and-strings");
  }
  const named = API_CATEGORIES.find((category) => category.names.test(name));
  if (named) {
    return named;
  }
  const returned = typeCategories.get(declaration.returns.toLowerCase());
  if (returned) {
    return returned;
  }
  for (const parameter of declaration.parameters) {
    const category = typeCategories.get(parameter.type.toLowerCase());
    if (category) {
      return category;
    }
  }
  return apiCategory("runtime");
}
