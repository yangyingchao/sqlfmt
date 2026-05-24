-- -*- coding: utf-8 -*-
-- ----------------------------------------------------------------------
-- Test: setup_schema.sql
-- ----------------------------------------------------------------------
create schema dml_over_joins;

set search_path = dml_over_joins;

-- ----------------------------------------------------------------------
-- Test: heap_motion1.sql
-- ----------------------------------------------------------------------
------------------------------------------------------------
-- Update with Motion:
--   r,s colocated on join attributes
--      delete: using clause, subquery, initplan
--      update: join and subsubquery
------------------------------------------------------------
drop table if exists r;

drop table if exists s;

create table r (
    a int4,
    b int4)
with (
    appendonly = true,
    compresslevel = 3
)
distributed by (a);

create table s (
    a int4,
    b text)
with (
    appendonly = true,
    orientation = column,
    compresstype = multiple,
    compresslevel = 3
)
    distributed by (a)
    partition by list(a);

insert into r
select
    generate_series(1, 10000),
    generate_series(1, 10000) * 3;

insert into s
select
    generate_series(1, 100),
    generate_series(1, 100) * 4;

update
    r
set
    b = r.b + 1
from
    s
where
    r.a = s.a;

update
    r
set
    b = r.b + 1
from
    s
where
    r.a in (
        select
            a
        from
            s);

delete from r using s
where r.a = s.a;

delete from r;

insert into r
select
    generate_series(1, 10000),
    generate_series(1, 10000) * 3;

delete from r
where a in (
        select
            a
        from
            s);

delete from r;

insert into r
select
    generate_series(1, 10000),
    generate_series(1, 10000) * 3;

delete from r
where a = (
        select
            max(a)
        from
            s);
