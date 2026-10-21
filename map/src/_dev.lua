-- Loaded before every other file in src/. Modules register a start
-- function with Dev.OnStart; they all run after the editor's own main().

Dev = { starters = {} }

function Dev.OnStart(fn)
    table.insert(Dev.starters, fn)
end
