-- ----------------------------------------------------------------------
-- Test: setup_schema.sql
-- ----------------------------------------------------------------------
CREATE SCHEMA dml_over_joins;

SET search_path = dml_over_joins;

-- ----------------------------------------------------------------------
-- Test: heap_motion1.sql
-- ----------------------------------------------------------------------
------------------------------------------------------------
-- Update with Motion:
--   r,s colocated on join attributes
--      delete: using clause, subquery, initplan
--      update: join and subsubquery
------------------------------------------------------------
DROP TABLE IF EXISTS r;

DROP TABLE IF EXISTS s;

CREATE TABLE r (
    a int4,
    b int4)
DISTRIBUTED BY (a);

CREATE TABLE s (
    a int4,
    b text)
DISTRIBUTED BY (a);

INSERT INTO r
SELECT
    generate_series(1, 10000),
    generate_series(1, 10000) * 3;

INSERT INTO s
SELECT
    generate_series(1, 100),
    generate_series(1, 100) * 4;

UPDATE
    r
SET
    b = r.b + 1
FROM
    s
WHERE
    r.a = s.a;

UPDATE
    r
SET
    b = r.b + 1
FROM
    s
WHERE
    r.a IN (
        SELECT
            a
        FROM
            s);

DELETE FROM r USING s
WHERE r.a = s.a;

DELETE FROM r;

INSERT INTO r
SELECT
    generate_series(1, 10000),
    generate_series(1, 10000) * 3;

DELETE FROM r
WHERE a IN (
        SELECT
            a
        FROM
            s);

DELETE FROM r;

INSERT INTO r
SELECT
    generate_series(1, 10000),
    generate_series(1, 10000) * 3;

DELETE FROM r
WHERE a = (
        SELECT
            max(a)
        FROM
            s);
