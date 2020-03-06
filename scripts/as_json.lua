json = require "scripts/json"

function setup(thread)
   thread0 = thread0 or thread
end
                   
function init(args)
   file = args[1] or "/dev/null"
end

function done(summary, latency, requests)
   print(json.encode({
       duration      = summary.duration,
       requests      = summary.requests,
       latency_min   = latency.min,
       latency_max   = latency.max,
       latency_mean  = latency.mean,
       latency_stdev = latency.stdev,
   }))
end
