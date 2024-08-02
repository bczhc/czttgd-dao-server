UPDATE tt_inspect
SET creator        = ?,
    devicecode     = ?,
    creationtime   = ?,
    spec           = ?,
    wirenum        = ?,
    breakspec      = ?,
    twbatchcode    = ?,
    trbatchcode    = ?,
    dlwarehouse    = ?,
    breakflag      = ?,
    breakpointa    = ?,
    breakpointb    = ?,
    memo           = ?,
    devicecategory = ?,
    breakreasona   = ?,
    billflag       = 0
WHERE id = ?