-- -*- coding: gb18030 -*-
-- 查询用户订单
-- 关联用户表和订单表
select
    u.name,
    o.order_id,
    o.total
from
    users u
    join orders o on
        u.id = o.user_id
where
    o.status = 'ACTIVE'
order by
    o.created_at desc;
