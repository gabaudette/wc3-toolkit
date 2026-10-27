-- {{SPELL_NAME}} ({{SPELL_ID}}): made by Tinkerbench from the "{{TEMPLATE}}" template.
-- Hits the target, then bounces to the nearest enemy it has not hit yet,
-- weaker each time. How far it looks for the next one is the spell's Area of
-- Effect, set in the editor.
-- Change the values below to tune it; each list has one value per level.

do
    local SPELL = FourCC("{{SPELL_ID}}")

    local DAMAGE = {{damage}}
    -- Bounces after the first hit.
    local BOUNCES = {{bounces}}
    -- Percent of the damage lost at each bounce.
    local REDUCTION = {{reduction}}
    -- Seconds between two hits, and how long each bolt stays visible.
    local BOUNCE_DELAY = 0.25
    local BOLT_TIME = 0.3
    local BOLT = "CLPB"
    local HIT_EFFECT = "Abilities\\Weapons\\Bolt\\BoltImpact.mdl"
    local BOLT_HEIGHT = 60

    local function at(list, level)
        return list[math.min(level, #list)]
    end

    local function isAlive(unit)
        return GetWidgetLife(unit) > 0.405
    end

    local function after(seconds, fn)
        local timer = CreateTimer()
        TimerStart(timer, seconds, false, function()
            DestroyTimer(timer)
            fn()
        end)
    end

    local function bolt(from, to)
        local lightning = AddLightningEx(BOLT, true,
            GetUnitX(from), GetUnitY(from), GetUnitFlyHeight(from) + BOLT_HEIGHT,
            GetUnitX(to), GetUnitY(to), GetUnitFlyHeight(to) + BOLT_HEIGHT)
        DestroyEffect(AddSpecialEffectTarget(HIT_EFFECT, to, "chest"))
        after(BOLT_TIME, function()
            DestroyLightning(lightning)
        end)
    end

    -- The closest living enemy around `from` that was not hit yet.
    local function nextTarget(owner, from, range, hit)
        local group = CreateGroup()
        local best, bestDistance = nil, range * range
        local x, y = GetUnitX(from), GetUnitY(from)

        GroupEnumUnitsInRange(group, x, y, range, nil)
        ForGroup(group, function()
            local unit = GetEnumUnit()
            if not hit[unit] and isAlive(unit) and IsUnitEnemy(unit, owner)
                and not IsUnitType(unit, UNIT_TYPE_STRUCTURE)
                and not IsUnitType(unit, UNIT_TYPE_MAGIC_IMMUNE) then
                local dx, dy = GetUnitX(unit) - x, GetUnitY(unit) - y
                local distance = dx * dx + dy * dy
                if distance <= bestDistance then
                    best, bestDistance = unit, distance
                end
            end
        end)
        DestroyGroup(group)
        return best
    end

    local function onCast()
        local caster = GetTriggerUnit()
        local owner = GetOwningPlayer(caster)
        local level = GetUnitAbilityLevel(caster, SPELL)
        local range = BlzGetAbilityRealLevelField(BlzGetUnitAbility(caster, SPELL),
            ABILITY_RLF_AREA_OF_EFFECT, level - 1)
        local keep = 1 - at(REDUCTION, level) / 100
        local bouncesLeft = at(BOUNCES, level)
        local damage = at(DAMAGE, level)
        local hit = {}

        local function strike(from, target)
            hit[target] = true
            bolt(from, target)
            UnitDamageTarget(caster, target, damage, true, false,
                ATTACK_TYPE_NORMAL, DAMAGE_TYPE_MAGIC, WEAPON_TYPE_WHOKNOWS)
            if bouncesLeft <= 0 then
                return
            end

            local nextUnit = nextTarget(owner, target, range, hit)
            if nextUnit == nil then
                return
            end

            bouncesLeft = bouncesLeft - 1
            damage = damage * keep
            after(BOUNCE_DELAY, function()
                if isAlive(nextUnit) then
                    strike(target, nextUnit)
                end
            end)
        end

        strike(caster, GetSpellTargetUnit())
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
