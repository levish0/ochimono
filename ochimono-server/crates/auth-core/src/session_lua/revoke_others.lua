local generation_raw = redis.call('GET', KEYS[1]) or '1'
if not string.match(generation_raw, '^[1-9][0-9]*$') then
    return redis.error_reply('SESSION_GENERATION invalid decimal')
end
local generation = tonumber(generation_raw)
local maximum = tonumber(ARGV[3])
if not generation or generation > maximum then
    return redis.error_reply('SESSION_GENERATION outside Lua-safe range')
end
if generation >= maximum then
    return {-1, generation, 0}
end
local security_generation_raw = ARGV[6]
if not string.match(security_generation_raw, '^[1-9][0-9]*$') then
    return redis.error_reply('SECURITY_GENERATION invalid decimal')
end
local security_generation = tonumber(security_generation_raw)
if not security_generation or security_generation > 9007199254740991 then
    return redis.error_reply('SECURITY_GENERATION outside Lua-safe range')
end

local current_raw = redis.call('GET', KEYS[3])
if not current_raw then
    return {0, generation, 0}
end
local current_ok, current = pcall(cjson.decode, current_raw)
if not current_ok or current.session_id ~= ARGV[2] or current.user_id ~= ARGV[1] or
   type(current.management_id) ~= 'string' or current.generation ~= generation then
    return {0, generation, 0}
end
local current_management_key = ARGV[4] .. current.management_id
local current_score = redis.call('ZSCORE', KEYS[2], current.management_id)
local current_ttl = redis.call('PTTL', KEYS[3])
if redis.call('GET', current_management_key) ~= ARGV[2] or not current_score or
   current_ttl <= 0 then
    return {0, generation, 0}
end

local management_ids = redis.call('ZRANGE', KEYS[2], 0, -1)
local session_keys = {}
local management_keys = {}
local deleted_count = 0
for index, management_id in ipairs(management_ids) do
    if management_id ~= current.management_id then
        local management_key = ARGV[4] .. management_id
        local session_id = redis.call('GET', management_key)
        management_keys[index] = management_key
        if session_id then
            local session_key = ARGV[5] .. session_id
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
end

-- Redis Lua does not roll back earlier DELs when a later cjson.encode raises. Prepare the exact
-- rebound payload before applying any mutation.
current.generation = generation + 1
current.security_generation = security_generation
local encode_ok, current_encoded = pcall(cjson.encode, current)
if not encode_ok then
    return redis.error_reply('SESSION_PAYLOAD cannot encode rebound session')
end

for index, management_key in pairs(management_keys) do
    if session_keys[index] then
        redis.call('DEL', session_keys[index])
    end
    redis.call('DEL', management_key)
end

redis.call('SET', KEYS[3], current_encoded, 'PX', current_ttl)
redis.call('SET', current_management_key, ARGV[2], 'PX', current_ttl)
redis.call('DEL', KEYS[2])
redis.call('ZADD', KEYS[2], current_score, current.management_id)
redis.call('PEXPIRE', KEYS[2], current_ttl)
redis.call('SET', KEYS[1], tostring(generation + 1))
return {1, generation + 1, deleted_count}
