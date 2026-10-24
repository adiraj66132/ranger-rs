-- Copy to ~/.config/ranger-rs/plugins/example.lua

ranger.register_command("hello", function(args)
    local name = args ~= "" and args or "world"
    ranger.notify("hello " .. name .. " from " .. ranger.cwd())
end)
ranger.register_key("H", "hello")

ranger.register_command("newdir", function(args)
    if args == "" then
        ranger.notify("usage: :newdir <name>")
        return
    end
    ranger.mkdir(args)
end)
ranger.register_key("N", "newdir")
