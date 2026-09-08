local generation_raw = redis.call('GET', KEYS[1]) or '1'
if not string.match(generation_raw, '^[1-9][0-9]*$') then
    return redis.error_reply('SESSION_GENERATION invalid decimal')
end
local generation = tonumber(generation_raw)
local maximum = tonumber(ARGV[2])
if not generation or generation > maximum then
    return redis.error_reply('SESSION_GENERATION outside Lua-safe range')
end
if generation >= maximum then
    return {-1, generation, 0}
end

local management_ids = redis.call('ZRANGE', KEYS[2], 0, -1)
local session_keys = {}
local management_keys = {}
local deleted_count = 0

for index, management_id in ipairs(management_ids) do
    local management_key = ARGV[3] .. management_id
    local session_id = redis.call('GET', management_key)
    management_keys[index] = management_key
    if session_id then
        local session_key = ARGV[4] .. session_id
        local raw = redis.call('GET', session_key)
        if raw then
            local ok, stored = pcall(cjson.decode, raw)
            if not ok or stored.session_id ~= session_id or stored.user_id ~= ARGV[1] or
               stored.management_id ~= management_id then
                return redis.error_reply('SESSION_INVARIANT indexed session mismatch')
            end
            session_keys[index] = session_key
            deleted_count = deleted_count + 1
        end
    end
end

for index, management_key in ipairs(management_keys) do
    if session_keys[index] then
        redis.call('DEL', session_keys[index])
    end
    redis.call('DEL', management_key)
end
redis.call('DEL', KEYS[2])
redis.call('SET', KEYS[1], tostring(generation + 1))
return {1, generation + 1, deleted_count}
