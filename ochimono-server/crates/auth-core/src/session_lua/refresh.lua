-- A renewal must never recreate a revoked or expired session.
if redis.call('GET', KEYS[1]) ~= ARGV[1] then return 0 end
if (redis.call('GET', KEYS[2]) or '1') ~= ARGV[2] then return 0 end
if redis.call('GET', KEYS[3]) ~= ARGV[3] then return 0 end
local index_type = redis.call('TYPE', KEYS[4])['ok']
if index_type ~= 'zset' then return 0 end
if not redis.call('ZSCORE', KEYS[4], ARGV[4]) then return 0 end
redis.call('SET', KEYS[1], ARGV[5], 'EX', ARGV[6])
redis.call('EXPIRE', KEYS[3], ARGV[6])
redis.call('ZADD', KEYS[4], ARGV[7], ARGV[4])
redis.call('EXPIRE', KEYS[4], ARGV[8])
return 1
