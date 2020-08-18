CREATE OR REPLACE FUNCTION dow_number_of(date) RETURNS SMALLINT AS $$
    SELECT (CAST(EXTRACT(dow FROM $1) AS SMALLINT) + 6::SMALLINT) % 7::SMALLINT + 1::SMALLINT
$$ LANGUAGE SQL IMMUTABLE;

CREATE OR REPLACE FUNCTION weekday_number(anyenum) RETURNS integer AS $$
SELECT enumpos::integer FROM (
        SELECT row_number() OVER (order by enumsortorder) AS enumpos,
               enumsortorder,
               enumlabel
        FROM pg_catalog.pg_enum
        WHERE enumtypid = pg_typeof($1)
    ) enum_ordering
    WHERE enumlabel = ($1::text);
$$ LANGUAGE SQL STABLE STRICT;

CREATE OR REPLACE FUNCTION first_occurence(start_day date, week_day SMALLINT) RETURNS date AS $$
    SELECT start_day + (week_day - dow_number_of(start_day) + 7) % 7
$$ LANGUAGE SQL IMMUTABLE;

CREATE OR REPLACE FUNCTION is_repeated_on(start_day DATE, week_day SMALLINT, every INTEGER, target_day DATE ) RETURNS BOOLEAN AS $$
    SELECT (target_day - first_occurence(start_day, week_day)) % every = 0
$$ LANGUAGE SQL IMMUTABLE;

CREATE OR REPLACE FUNCTION repeats_on_date(start_day DATE, end_day DATE, week_day SMALLINT, every INTEGER, target_day DATE) RETURNS BOOLEAN AS $$
    SELECT start_day < target_day AND
        (end_day IS NULL OR end_day > target_day) AND
        week_day = dow_number_of(target_day) AND
        is_repeated_on(start_day, week_day, every, target_day)
$$ LANGUAGE SQL IMMUTABLE;

CREATE OR REPLACE FUNCTION lesson_permission_for(lesson_id UUID, account_id UUID) RETURNS PermissionType AS $$
    SELECT type FROM LessonPermission WHERE lesson_id = lesson_id AND account_id = account_id
$$ LANGUAGE SQL STABLE;

CREATE OR REPLACE FUNCTION is_read_permission(permission PermissionType) RETURNS BOOLEAN AS $$
    SELECT permission = 'r'::PermissionType OR permission = 'rw'::PermissionType
$$ LANGUAGE SQL STABLE;

ALTER TABLE Repeats
    ALTER COLUMN week_day TYPE SMALLINT USING weekday_number(week_day);

ALTER TABLE Repeats
    ADD CONSTRAINT week_day_is_in_range CHECK (week_day BETWEEN 1 AND 7);
