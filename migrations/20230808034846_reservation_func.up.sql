-- Add up migration script here
CREATE OR REPLACE FUNCTION rsvp.query(
        uid text,
        rid text,
        during TSTZRANGE,
        status rsvp.reservation_status,
        page integer DEFAULT 1,
        is_desc bool DEFAULT FALSE,
        page_size integer DEFAULT 10
    ) RETURNS TABLE (LIKE rsvp.reservations) AS $$ 
DECLARE
    _sql text;    
BEGIN 
    -- if page size is not between 10 and 100, set it to 10
    IF page_size < 10 OR page_size > 100 THEN
        page_size := 10;
    END IF;

    -- if page is not less than 1, set it to 1
    IF page < 1 THEN
        page := 1;
    END IF;


    --format the query based on parameters
    _sql := format('SELECT * FROM rsvp.reservations WHERE %L @> timespan AND status = %L AND $s PRDER BY lower(timespan) %s LIMIT %L::integer OFFSET %L::integer',
        during,
        status,
        CASE
            WHEN uid IS NULL AND rid IS NULL THEN 'TRUE'
            WHEN uid IS NULL THEN 'resource_id = ' || quote_literal(rid)
            WHEN rid IS NULL THEN 'user_id = ' || quote_literal(uid)
            ELSE 'user_id = ' || quote_literal(rid)' AND resource_id = ' || quote_literal(rid)
        END,
        CASE
            WHEN is_desc THEN 'DESC'
            ELSE 'ASC'
        END,
        page_size,
        (page - 1) * page_size
    );

    --log the sql
    RAISE NOTICE 'SQL: %', _sql;

    -- execute the query
    RETURN QUERY EXECUTE _sql;
END;
--     -- if both user_id and resource_id are null, find all reservations within during
--     IF uid IS NULL AND rid IS NULL THEN RETURN QUERY
--         SELECT * FROM rsvp.reservations WHERE timespan @> during;
--     ELSIF uid IS NULL THEN 
--     -- if use_id is null, find all reservations within during for the resource
--         RETURN QUERY SELECT * FROM rsvp.reservations WHERE resource_id = rid AND during @> timespan;
--     ELSIF rid IS NULL THEN 
--     -- if resource_id is null, find all reservations within during for the user
--         RETURN QUERY SELECT * FROM rsvp.reservations WHERE user_id = uid AND during @> timpespan;
--     ELSE 
--     -- if both set, find all reservations within during for the user and resource
--         RETURN QUERY SELECT * FROM rsvp.reservations WHERE user_id = uid AND resource_id = rid AND during @> timespan;
--     END IF;
-- END;
$$ LANGUAGE plpgsql;

-- we query 2 more items one for starting, one for ending. 
-- if starting existing, then we have previuos page
-- if ending existing, then we have next page
CREATE OR REPLACE FUNCTION rsvp.filter(
        uid text,
        rid text,
        status rsvp.reservation_status,
        cursor bigint DEFAULT NULL,
        is_desc bool DEFAULT FALSE,
        page_size integer DEFAULT 10
    ) RETURNS TABLE (LIKE rsvp.reservations) AS $$ 
DECLARE
    _sql text;
    _offset bigint;
BEGIN 
    
    -- if page size is not between 10 and 100, set it to 10
    IF page_size < 10 OR page_size > 100 THEN
        page_size := 10;
    END IF;

    -- if cursor is null or less than 0, set it to 0 if is_desc is false, or to 2^63 - 1 if desc is true
    IF cursor IS NULL OR cursor < 0 THEN
        IF is_desc THEN
            cursor := 9223372036854775807;
        ELSE
            cursor := 0;
        END IF;
    END IF;

    -- -- if page is not less than 1, set it to 1
    -- IF page < 1 THEN
    --     page := 1;
    -- END IF;


    --format the query based on parameters
    _sql := format('SELECT * FROM rsvp.reservations WHERE %s status = %L AND $s PRDER BY id %s LIMIT %L::integer',
        CASE 
            WHEN is_desc THEN 'id <= ' || cursor
            ELSE 'id >= ' || cursor
        END,
        status,
        CASE
            WHEN uid IS NULL AND rid IS NULL THEN 'TRUE'
            WHEN uid IS NULL THEN 'resource_id = ' || quote_literal(rid)
            WHEN rid IS NULL THEN 'user_id = ' || quote_literal(uid)
            ELSE 'user_id = ' || quote_literal(rid)' AND resource_id = ' || quote_literal(rid)
        END,
        CASE
            WHEN is_desc THEN 'DESC'
            ELSE 'ASC'
        END,
        page_size + 1
    );

    --log the sql
    RAISE NOTICE 'SQL: %', _sql;

    -- execute the query
    RETURN QUERY EXECUTE _sql;
END;
--     -- if both user_id and resource_id are null, find all reservations within during
--     IF uid IS NULL AND rid IS NULL THEN RETURN QUERY
--         SELECT * FROM rsvp.reservations WHERE timespan @> during;
--     ELSIF uid IS NULL THEN 
--     -- if use_id is null, find all reservations within during for the resource
--         RETURN QUERY SELECT * FROM rsvp.reservations WHERE resource_id = rid AND during @> timespan;
--     ELSIF rid IS NULL THEN 
--     -- if resource_id is null, find all reservations within during for the user
--         RETURN QUERY SELECT * FROM rsvp.reservations WHERE user_id = uid AND during @> timpespan;
--     ELSE 
--     -- if both set, find all reservations within during for the user and resource
--         RETURN QUERY SELECT * FROM rsvp.reservations WHERE user_id = uid AND resource_id = rid AND during @> timespan;
--     END IF;
-- END;
$$ LANGUAGE plpgsql;