SELECT i.breakreasona     as br_a,
       i.breakreasonb     as br_b,
       br_a.breakreasonid as br_a_cause_id,
       br_a.reasontype    as br_a_cause_type,
       br_a.breakreason   as br_a_cause_name,
       br_b.breakreasonid as br_b_cause_id,
       br_b.reasontype    as br_b_cause_type,
       br_b.breakreason   as br_b_cause_name,
       i.spec             as product_spec,
       i.breakflag = '1'  as break_flag,
       i.breakspec        as break_spec,
       i.creator,
       u.userid           as user_id,
       u.name             as user_name,
       u.usertype         as user_user_type,
       u.enablestate      as user_enable_state,
       i.creationtime     as creation_time,
       i.billflag         as inspection_flag,
       i.devicecode       as device_code,
       i.id
FROM tt_inspect i
         INNER JOIN tt_machine m ON i.devicecode = m.machinenumber
         LEFT JOIN tt_user u ON i.creator = u.userid
         LEFT JOIN tt_breakreason br_a ON i.breakreasona = br_a.breakreasonid
         LEFT JOIN tt_breakreason br_b ON i.breakreasona = br_b.breakreasonid
WHERE m.stage = ?
  AND i.deleteflag = 0
  AND (
    u.name LIKE CONCAT('%', ?, '%')
        OR br_a.breakreason LIKE CONCAT('%', ?, '%')
        OR br_b.breakreason LIKE CONCAT('%', ?, '%')
        OR i.spec LIKE CONCAT('%', ?, '%')
        OR i.creationtime LIKE CONCAT('%', DATE_FORMAT(CONVERT(?, DATE), '%Y-%m-%d'), '%')
        OR i.creationtime LIKE CONCAT('%', ?, '%')
        OR REPLACE(i.creationtime, '-', '.') LIKE CONCAT('%', ?, '%')
        OR REPLACE(i.creationtime, '-0', '.') LIKE CONCAT('%', ?, '%')
        OR CONCAT(devicecode, '号机台') LIKE CONCAT('%', ?, '%')
        OR i.memo LIKE CONCAT('%', ?, '%')
    )
LIMIT ? OFFSET ?