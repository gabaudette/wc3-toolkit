-- Welcome message shown to every player shortly after the game starts.

Welcome = {}

-- Text sent during map init is lost behind the loading screen,
-- so wait this long (game seconds) before showing it.
local SHOW_DELAY = 1.0
local MESSAGE_DURATION = 15

local function showToPlayingPlayers()
    for i = 0, bj_MAX_PLAYERS - 1 do
        local p = Player(i)

        if GetPlayerSlotState(p) == PLAYER_SLOT_STATE_PLAYING then
            DisplayTimedTextToPlayer(p, 0, 0, MESSAGE_DURATION,
                "|cffffcc00Welcome, " .. GetPlayerName(p) .. "!|r\n" ..
                "This map is now running custom Lua code.")
        end
    end
end

function Welcome.Init()
    local t = CreateTimer()

    TimerStart(t, SHOW_DELAY, false, function()
        DestroyTimer(t)
        showToPlayingPlayers()
    end)
end

Dev.OnStart(Welcome.Init)
