package test

now_ns := time.now_ns()

# still not implemented
#parse_ns = time.parse_ns("2006-01-01", "2022-01-01")
#parse_ns1 = time.parse_ns("2006-01-01", "2022-01-08")

parse_rfc3339_ns := time.parse_rfc3339_ns("2022-07-31T12:22:40.727411+00:00")

parse_duration_ns_us := time.parse_duration_ns("1us")

parse_duration_ns_us1 := time.parse_duration_ns("1ns")

parse_duration_ns_us2 := time.parse_duration_ns("1µs")

parse_duration_ns_ms := time.parse_duration_ns("1ms")

parse_duration_ns_s := time.parse_duration_ns("1s")

parse_duration_ns_m := time.parse_duration_ns("1m")

parse_duration_ns_h := time.parse_duration_ns("1h")

date := time.date(1659996459131330000)

date_by_tz := time.date([1659996459131330000, "Europe/Paris"])

clock := time.clock(1659996459131330000)

clock_tz := time.clock([1659996459131330000, "Europe/Paris"])

weekday := time.weekday(1659996459131330000)

weekday2 := time.weekday([1659996459131330000, "Europe/Paris"])

add_date := time.add_date(1659996459131330000, 1, 1, 1)

# still not implemented
#diff := time.diff(1659996459131330000, 1659017824635051000)
#diff2 := time.diff([1659996459131330000, "Europe/Paris"], [1658997582413084200, "Europe/Paris"])
