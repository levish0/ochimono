local raw = redis.call('GET', KEYS[1])
if not raw then
    return 0
end
local ok, stored = pcall(cjson.decode, raw)
if not ok or type(stored) ~= 'table' or stored.session_id ~= ARGV[1] or
   type(stored.user_id) ~= 'string' or
   type(stored.management_id) ~= 'string' then
    redis.call('DEL', KEYS[1])
    return 2
end

local management_key = ARGV[2] .. stored.management_id
local index_key = ARGV[3] .. stored.user_id
if redis.call('GET', management_key) ~= ARGV[1] or
   not redis.call('ZSCORE', index_key, stored.management_id) then
    redis.call('DEL', KEYS[1])
    return 2
end

redis.call('DEL', KEYS[1])
redis.call('DEL', management_key)
redis.call('ZREM', index_key, stored.management_id)
return 1
