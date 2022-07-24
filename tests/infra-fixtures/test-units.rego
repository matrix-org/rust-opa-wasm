package test

v_bytes := units.parse_bytes("1K")

v_bytes_nounit := units.parse_bytes("100")

v_bytes_mib := units.parse_bytes("1MiB")

v_bytes_lower := units.parse_bytes("200m")

v_bytes_frac := units.parse_bytes("1.5mib")

v_bytes_zero := units.parse_bytes("0M")

v_decimal := units.parse("1K")

v_decimal_nounit := units.parse("100")

v_decimal_lower := units.parse("1mb")

v_decimal_mili := units.parse("200m")

v_decimal_frac := units.parse("1.5M")

v_decimal_zero := units.parse("0M")
