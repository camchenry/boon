function love.load()
    local os = love.system.getOS()
    local fh = assert(io.open('OK', 'wb'))
    fh:write(os)
    fh:flush()
    fh:close()
    love.event.quit(0)
end