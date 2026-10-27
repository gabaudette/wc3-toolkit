-- {{SPELL_NAME}} ({{SPELL_ID}}): made by Tinkerbench from the "{{TEMPLATE}}" template.
-- Damages enemies around the caster and pushes them away from it. The radius
-- is the spell's Area of Effect, set in the editor.
-- Change the values below to tune it; each list has one value per level.

do
    local SPELL = FourCC("{{SPELL_ID}}")

    local DAMAGE = {{damage}}
    -- How far each enemy is pushed, and over how many seconds.
    local DISTANCE = {{distance}}
    local DURATION = {{duration}}
    local CAST_EFFECT = "Abilities\\Spells\\Human\\ThunderClap\\ThunderClapCaster.mdl"
    -- Seconds between two steps of the push.
    local STEP_TIME = 0.03

    local function at(list, level)
        return list[math.min(level, #list)]
    end

    local function isAlive(unit)
        return GetWidgetLife(unit) > 0.405
    end

    local function canMoveTo(x, y)
        -- IsTerrainPathable is true where ground units cannot walk.
        return RectContainsCoords(bj_mapInitialPlayableArea, x, y)
            and not IsTerrainPathable(x, y, PATHING_TYPE_WALKABILITY)
    end

    local function push(caster, target, level)
        local angle = Atan2(GetUnitY(target) - GetUnitY(caster), GetUnitX(target) - GetUnitX(caster))
        local steps = math.max(1, math.floor(at(DURATION, level) / STEP_TIME + 0.5))
        local stepLength = at(DISTANCE, level) / steps
        local dx, dy = stepLength * Cos(angle), stepLength * Sin(angle)
        local done = 0
        local timer = CreateTimer()

        TimerStart(timer, STEP_TIME, true, function()
            done = done + 1
            local x, y = GetUnitX(target) + dx, GetUnitY(target) + dy
            if isAlive(target) and canMoveTo(x, y) then
                SetUnitX(target, x)
                SetUnitY(target, y)
            else
                done = steps
            end

            if done >= steps then
                PauseTimer(timer)
                DestroyTimer(timer)
            end
        end)
    end

    local function onCast()
        local caster = GetTriggerUnit()
        local owner = GetOwningPlayer(caster)
        local level = GetUnitAbilityLevel(caster, SPELL)
        local radius = BlzGetAbilityRealLevelField(BlzGetUnitAbility(caster, SPELL),
            ABILITY_RLF_AREA_OF_EFFECT, level - 1)
        local x, y = GetUnitX(caster), GetUnitY(caster)
        local group = CreateGroup()

        DestroyEffect(AddSpecialEffect(CAST_EFFECT, x, y))
        GroupEnumUnitsInRange(group, x, y, radius, nil)
        ForGroup(group, function()
            local unit = GetEnumUnit()
            if IsUnitEnemy(unit, owner) and isAlive(unit)
                and not IsUnitType(unit, UNIT_TYPE_STRUCTURE)
                and not IsUnitType(unit, UNIT_TYPE_FLYING)
                and not IsUnitType(unit, UNIT_TYPE_MAGIC_IMMUNE) then
                UnitDamageTarget(caster, unit, at(DAMAGE, level), true, false,
                    ATTACK_TYPE_NORMAL, DAMAGE_TYPE_MAGIC, WEAPON_TYPE_WHOKNOWS)
                push(caster, unit, level)
            end
        end)
        DestroyGroup(group)
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
