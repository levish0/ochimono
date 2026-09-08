local generation = redis.call('GET', KEYS[1]) or '1'
if generation ~= ARGV[1] then
    return 0
end

if redis.call('EXISTS', KEYS[2]) ~= 0 or redis.call('EXISTS', KEYS[3]) ~= 0 then
    return redis.error_reply('SESSION_INVARIANT id collision')
end

-- Redis scripts do not roll back commands already executed when a later command raises a runtime
-- error. Preflight the only type-sensitive write before creating any part of the session tuple.
local index_type = redis.call('TYPE', KEYS[4])['ok']
if index_type ~= 'none' and index_type ~= 'zset' then
    return redis.error_reply('SESSION_INVARIANT user index wrong type')
end

redis.call('SET', KEYS[1], generation, 'NX')
redis.call('PERSIST', KEYS[1])
redis.call('SET', KEYS[2], ARGV[2], 'EX', ARGV[6])
redis.call('SET', KEYS[3], ARGV[3], 'EX', ARGV[6])
redis.call('ZADD', KEYS[4], ARGV[5], ARGV[4])
redis.call('EXPIRE', KEYS[4], ARGV[7])
return 1
