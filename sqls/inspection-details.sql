SELECT i.devicecode       as device_code,
       i.devicecategory   as device_category,
       i.creator,
       u.userid           as user_id,
       u.name             as user_name,
       u.usertype         as user_user_type,
       u.enablestate      as user_enable_state,
       i.creationtime     as creation_time,
       i.billflag         as inspection_flag,
       i.spec             as product_spec,
       i.wirenum          as wire_num,
       i.breakspec        as break_spec,
       i.twbatchcode      as wire_batch_code,
       i.trbatchcode      as stick_batch_code,
       i.dlwarehouse      as warehouse,
       i.tgproducttime    as product_time,
       i.breakflag = '1'  as break_flag,
       i.breakpointb      as breakpoint_b,
       i.breakpointa      as breakpoint_a,
       bp_a.breakpointid  as bp_a_bp_id,
       bp_a.breakpoint    as bp_a_bp_name,
       i.breakreasona     as break_cause_a,
       br_a.breakreasonid as br_a_cause_id,
       br_a.reasontype    as br_a_cause_type,
       br_a.breakreason   as br_a_cause_name,
       br_a.enablestate   as br_a_cause_enable_state,
       br_b.breakreasonid as br_b_cause_id,
       br_b.reasontype    as br_b_cause_type,
       br_b.breakreason   as br_b_cause_name,
       br_b.enablestate   as br_b_cause_enable_state,
       i.breakreasonb     as break_cause_b,
       i.memo             as comments,
       -- Due to unexpected database changing demands, i.inspector is a string, thus we
       -- can't get its id here. The inspector id is not used for the App side, so just set
       -- a dummy value.
       0                  as inspector_user_id,
       '1'                as inspector_user_user_type,
       2                  as inspector_user_enable_state,
       i.inspector        as inspector_user_name,
       i.inspecttime      as inspection_time,
       i.id
FROM tt_inspect i
         LEFT JOIN tt_user u
                   ON i.creator = u.userid
         LEFT JOIN tt_breakpoint bp_a
                   ON i.breakpointa = bp_a.breakpointid
         LEFT JOIN tt_breakreason br_a
                   ON i.breakreasona = br_a.breakreasonid
         LEFT JOIN tt_breakreason br_b
                   ON i.breakreasonb = br_b.breakreasonid
WHERE i.id = ?;