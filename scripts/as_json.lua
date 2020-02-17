json = require "scripts/json"

function setup(thread)
   thread0 = thread0 or thread
end
                   
function init(args)
   file = args[1] or "/dev/null"
end

function done(summary, latency, requests)
   percentiles = {}
   
   for _, p in pairs({ 50, 90, 99, 99.999 }) do
      k = string.format("%g%%", p)
      v = latency:percentile(p)
      percentiles[k] = v
   end
   
   print(json.encode({
       duration = summary.duration,
       requests = summary.requests,
       bytes    = summary.bytes,
       errors   = summary.errors,
       latency  = {
          min         = latency.min,
          max         = latency.max,
          mean        = latency.mean,
          stdev       = latency.stdev,
          percentiles = percentiles,
       },
   }))
end
