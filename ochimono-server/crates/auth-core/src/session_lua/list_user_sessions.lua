local generation_raw = redis.call('GET', KEYS[1]) or '1'
if not string.match(generation_raw, '^[1-9][0-9]*$') then
    return redis.error_reply('SESSION_GENERATION invalid decimal')
end
local generation = tonumber(generation_raw)
local maximum = tonumber(ARGV[2])
if not generation or generation > maximum then
    return redis.error_reply('SESSION_GENERATION outside Lua-safe range')
end
local now = tonumber(ARGV[5])

-- Listing and stale-entry cleanup must be one Redis linearization point. In particular,
-- revoke_others may rebind its kept session to the next generation; a stale list must never
-- subsequently delete that newly valid management/index tuple.
local management_ids = redis.call('ZRANGE', KEYS[2], 0, -1)
local decisions = {}
for index, management_id in ipairs(management_ids) do
    local management_key = ARGV[3] .. management_id
    local session_id = redis.call('GET', management_key)
    local raw = nil
    local score = redis.call('ZSCORE', KEYS[2], management_id)
    local index_is_current = score and (not now or tonumber(score) > now)
    local valid = false
    local safe_to_delete_management = false

    if session_id then
        raw = redis.call('GET', ARGV[4] .. session_id)
        if raw then
            local decoded, stored = pcall(cjson.decode, raw)
            if decoded and type(stored) == 'table' and stored.session_id == session_id and
               stored.user_id == ARGV[1] and stored.management_id == management_id then
                -- Only a payload that proves this exact tuple belongs to this user authorizes
                -- deleting the global management key. A corrupt index must not evict another
                -- user's valid session merely because it names that session's management id.
                safe_to_delete_management = true
                valid = index_is_current and stored.generation == generation
            end
        else
            -- The mapping names no bearer payload at all, so it cannot be another valid session.
            safe_to_delete_management = true
        end
    end

    -- Redis Lua does not roll back earlier writes when a later GET/ZSCORE raises WRONGTYPE. Read
    -- and validate the complete index first; only then apply the safe cleanup decisions.
    decisions[index] = {
        management_id = management_id,
        management_key = management_key,
        raw = raw,
        valid = valid,
        safe_to_delete_management = safe_to_delete_management
    }
end

local sessions = {}
for _, decision in ipairs(decisions) do
    if decision.valid then
        table.insert(sessions, decision.raw)
    else
        if decision.safe_to_delete_management then
            redis.call('DEL', decision.management_key)
        end
        -- This index belongs to the requested user, so its removal is safe even when the global
        -- management mapping/payload was malformed or belongs to someone else.
        redis.call('ZREM', KEYS[2], decision.management_id)
    end
end

return sessions
