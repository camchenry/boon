-- This file ensures that required files will work

local test = function()
    local os = love.system.getOS()
    local fh = assert(io.open('OK', 'wb'))
    fh:write(os)
    fh:flush()
    fh:close()
end

return test
