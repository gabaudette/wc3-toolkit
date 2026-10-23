---

title: "Write your first spell trigger"
description: "Register a spell-effect event and filter a custom ability in JASS and Lua."
---

This example registers the spell-effect event for every player slot and reacts only to a chosen ability. Replace `A000` with your custom ability’s raw code.

## Lua

For the [Lua harness](../../editor/lua-harness/), put this in a module:

```lua
Dev.OnStart(function()
    local trigger = CreateTrigger()
    for index = 0, bj_MAX_PLAYER_SLOTS - 1 do
        TriggerRegisterPlayerUnitEvent(trigger, Player(index),
            EVENT_PLAYER_UNIT_SPELL_EFFECT, nil)
    end

    TriggerAddAction(trigger, function()
        if GetSpellAbilityId() ~= FourCC("A000") then
            return
        end
        local caster = GetTriggerUnit()
        DisplayTimedTextToPlayer(GetOwningPlayer(caster), 0, 0, 5,
            "Custom spell cast!")
    end)
end)
```

Keep the trigger for the map’s lifetime. If you build a temporary system, keep its handle somewhere you can destroy it when that system ends.

## JASS

In a JASS map, add these functions in custom script in declaration order, and call `InitCustomSpell()` once from a map initialization trigger:

```jass
function CustomSpellAction takes nothing returns nothing
    if GetSpellAbilityId() == 'A000' then
        call DisplayTimedTextToPlayer(GetOwningPlayer(GetTriggerUnit()), 0.0, 0.0, 5.0, "Custom spell cast!")
    endif
endfunction

function InitCustomSpell takes nothing returns nothing
    local trigger t = CreateTrigger()
    local integer index = 0
    loop
        exitwhen index >= bj_MAX_PLAYER_SLOTS
        call TriggerRegisterPlayerUnitEvent(t, Player(index), EVENT_PLAYER_UNIT_SPELL_EFFECT, null)
        set index = index + 1
    endloop
    call TriggerAddAction(t, function CustomSpellAction)
    set t = null
endfunction
```

## Verify the event

Give your custom ability to a test unit, launch the correct map and cast it. The owner should see the message when the spell effect happens. Cast a different ability too: it should not show the message.

`GetSpellAbilityId()` and `GetTriggerUnit()` are event-context functions here. Read them in the action; if you defer work with a timer, capture the values you need before the original action ends. Keep gameplay changes synchronized across players; local display branches must not create different gameplay state.
