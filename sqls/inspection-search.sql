SELECT i.breakreasona     as br_a,
       i.breakreasonb     as br_b,
       br_a.breakreasonid as br_a_cause_id,
       br_a.reasontype    as br_a_cause_type,
       br_a.breakreason   as br_a_cause_name,
       br_b.breakreasonid as br_b_id,
       br_b.reasontype    as br_b_type,
       br_b.breakreason   as br_b_name,
       i.spec,
       i.breakspec,
       i.creator,
       u.userid           as user_id,
       u.name             as user_name,
       i.creationtime,
       i.billflag,
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
    i.creator LIKE CONCAT('%', ?, '%')
        OR i.breakreasona LIKE CONCAT('%', ?, '%')
        OR i.breakreasonb LIKE CONCAT('%', ?, '%')
        OR i.spec LIKE CONCAT('%', ?, '%')
        OR i.creationtime LIKE CONCAT('%', ?, '%')
        OR CONCAT(devicecode, '号机台') LIKE CONCAT('%', ?, '%')
        OR i.memo LIKE CONCAT('%', ?, '%')
    )
