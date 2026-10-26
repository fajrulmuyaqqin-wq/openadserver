-- wrk benchmark script for OpenAdServer /api/v1/ad/request
wrk.method = "POST"
wrk.headers["Content-Type"] = "application/json"

local counter = 0

request = function()
    counter = counter + 1
    local user_id = "bench_user_" .. (counter % 10000)
    local os = (counter % 2 == 0) and "android" or "ios"
    local country = (counter % 3 == 0) and "ID" or "US"
    local body = string.format([[
    {
        "slot_id": "slot_banner_01",
        "user_id": "%s",
        "num_ads": 2,
        "device": {
            "os": "%s",
            "model": "Pixel 8"
        },
        "geo": {
            "country": "%s",
            "city": "Jakarta"
        }
    }
    ]], user_id, os, country)

    return wrk.format(nil, nil, nil, body)
end
