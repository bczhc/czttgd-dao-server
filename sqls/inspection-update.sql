UPDATE tt_inspect
SET creator        = ?,
    devicecode     = ?,
    creationtime   = ?,
    spec           = ?,
    wirespeed      = ?,
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
    billflag       = 0
WHERE id = ?