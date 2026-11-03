local starters = {}
local dialogs = {}
local displays = {}
local messages = 0
local clicked, clickingPlayer
local players = {
    { playing = true, human = true },
    { playing = true, human = false },
    { playing = false, human = true },
    { playing = true, human = true },
}

Dev = { OnStart = function(fn) table.insert(starters, fn) end }
bj_MAX_PLAYERS = #players
PLAYER_SLOT_STATE_PLAYING = true
MAP_CONTROL_USER = true

function Player(index) return players[index + 1] end
function GetPlayerSlotState(player) return player.playing end
function GetPlayerController(player) return player.human end
function DialogCreate()
    local dialog = { buttons = {} }
    table.insert(dialogs, dialog)
    return dialog
end
function DialogSetMessage(dialog, text)
    dialog.message = text
    messages = messages + 1
end
function DialogAddButton(dialog, text, hotkey)
    local button = { text = text, hotkey = hotkey }
    table.insert(dialog.buttons, button)
    return button
end
function DialogDisplay(player, dialog, show)
    if show then
        assert(dialog.message == dialog.expected, "show must restore the designed message")
        table.insert(displays, { player = player, dialog = dialog })
        -- Warcraft III does not retain the message for the next display.
        dialog.message = ""
    end
end
function CreateTrigger() return {} end
function TriggerRegisterDialogEvent(trigger, dialog) dialog.trigger = trigger end
function TriggerAddAction(trigger, action) trigger.action = action end
function GetClickedButton() return clicked end
function GetTriggerPlayer() return clickingPlayer end

local function click(dialog, index, player)
    clicked = dialog.buttons[index]
    clickingPlayer = player
    dialog.message = ""
    dialog.trigger.action()
end

function verify_dialogs()
    UI.Difficulty.Show(Player(0))
    UI.Difficulty.Hide(Player(0))
    assert(#displays == 0, "calls before initialization must remain safe")
    for _, fn in ipairs(starters) do fn() end
    assert(#dialogs == 2)
    for index, dialog in ipairs(dialogs) do dialog.expected = expected[index] end
    local main, other = dialogs[1], dialogs[2]
    local callbacks = 0
    function OnHardMode(player)
        assert(player == Player(3) and player == GetTriggerPlayer())
        callbacks = callbacks + 1
    end

    for _ = 1, 3 do
        UI.Difficulty.Show(Player(0))
        local beforeHide = messages
        UI.Difficulty.Hide(Player(0))
        assert(messages == beforeHide, "hiding must not change the message")
        UI.Difficulty.Show(Player(0))
        click(main, 2, Player(0))
        assert(displays[#displays].dialog == other)
        click(other, 1, Player(0))
        assert(displays[#displays].dialog == main)
        click(main, 1, Player(0))
        UI.Difficulty.Show(Player(3))
        click(main, 3, Player(3))
        assert(displays[#displays].player == Player(3))
    end
    assert(callbacks == 3)

    for _ = 1, 2 do
        local beforeAll = #displays
        UI.Difficulty.ShowAll()
        assert(#displays == beforeAll + 2, "show only playing human slots")
        assert(displays[beforeAll + 1].player == Player(0))
        assert(displays[beforeAll + 2].player == Player(3))
    end
    assert(#dialogs == 2 and #main.buttons == 3 and #other.buttons == 1)
end
