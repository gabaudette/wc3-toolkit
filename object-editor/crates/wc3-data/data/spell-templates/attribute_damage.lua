-- {{SPELL_NAME}} ({{SPELL_ID}}): made by Tinkerbench from the "{{TEMPLATE}}" template.
-- Damages the target based on the caster's strength, agility or intelligence.
-- Change the values below to tune it; each list has one value per level.

do
    local SPELL = FourCC("{{SPELL_ID}}")

    -- 0 = strength, 1 = agility, 2 = intelligence (bonuses from items count).
    local ATTRIBUTE = {{attribute}}
    local BASE_DAMAGE = {{base_damage}}
    local DAMAGE_PER_POINT = {{per_point}}
    local HIT_EFFECT = "Abilities\\Spells\\Other\\Incinerate\\FireLordDeathExplode.mdl"

    -- The value for a level; levels past the end of a list use its last value.
    local function at(list, level)
        return list[math.min(level, #list)]
    end

    local function attribute(unit)
        if ATTRIBUTE == 1 then
            return GetHeroAgi(unit, true)
        elseif ATTRIBUTE == 2 then
            return GetHeroInt(unit, true)
        end
        return GetHeroStr(unit, true)
    end

    local function onCast()
        local caster = GetTriggerUnit()
        local target = GetSpellTargetUnit()
        local level = GetUnitAbilityLevel(caster, SPELL)
        local amount = at(BASE_DAMAGE, level) + at(DAMAGE_PER_POINT, level) * attribute(caster)

        DestroyEffect(AddSpecialEffectTarget(HIT_EFFECT, target, "origin"))
        UnitDamageTarget(caster, target, amount, true, false,
            ATTACK_TYPE_NORMAL, DAMAGE_TYPE_MAGIC, WEAPON_TYPE_WHOKNOWS)
    end

    Dev.OnStart(function()
        local trigger = CreateTrigger()
        TriggerRegisterAnyUnitEventBJ(trigger, EVENT_PLAYER_UNIT_SPELL_EFFECT)
        TriggerAddAction(trigger, function()
            if GetSpellAbilityId() == SPELL then
                onCast()
            end
        end)
    end)
end
