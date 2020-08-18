SELECT * FROM repeats WHERE start_day < date '2020-08-20' AND (end_day IS NULL OR end_day > date '2020-08-20') AND week_day = 'FRI';
SELECT ((date '2020-08-10' - date '2020-08-03') % 7) = 0;
SELECT pg_typeof(CAST(EXTRACT(dow FROM date '2020-08-23') AS INTEGER) + 6);
SELECT (CAST(EXTRACT(dow FROM date '2020-08-17') AS INTEGER) + 6) % 7 + 1;
SELECT pg_typeof(WeekDay 'MON');
SELECT (
    SELECT enumsortorder FROM pg_enum WHERE enumlabel = 'MON' AND enumtypid = (SELECT oid FROM pg_type WHERE typname = 'weekday')
);

CREATE OR REPLACE FUNCTION dow_number_of(date) RETURNS SMALLINT AS $$
    SELECT (CAST(EXTRACT(dow FROM $1) AS SMALLINT) + 6::SMALLINT) % 7::SMALLINT + 1::SMALLINT
$$ LANGUAGE SQL IMMUTABLE;

SELECT dow_number_of('2020-08-17');

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

SELECT weekday_number(WeekDay 'MON');

CREATE OR REPLACE FUNCTION first_occurence(start_day date, week_day SMALLINT) RETURNS date AS $$
    SELECT start_day + (week_day - dow_number_of(start_day) + 7) % 7
$$ LANGUAGE SQL IMMUTABLE;

SELECT first_occurence('2020-08-20'::DATE, 1::SMALLINT);

CREATE OR REPLACE FUNCTION is_repeated_on(start_day DATE, week_day SMALLINT, every INTEGER, target_day DATE ) RETURNS BOOLEAN AS $$
    SELECT (target_day - first_occurence(start_day, week_day)) % every = 0
$$ LANGUAGE SQL IMMUTABLE;

SELECT is_repeated_on('2020-08-18'::DATE, 5::SMALLINT, 7, '2020-08-20');

SELECT * FROM repeats 
WHERE 
    start_day < date '2020-08-20' AND
    (end_day IS NULL OR end_day > date '2020-08-20') AND
    weekday_number(week_day) = dow_number_of('2020-08-20') AND
    is_repeated_on(start_day, weekday_number(week_day)::SMALLINT, every, '2020-08-20')

CREATE OR REPLACE FUNCTION repeats_on_date(start_day DATE, end_day DATE, week_day SMALLINT, every INTEGER, target_day DATE) RETURNS BOOLEAN AS $$
    SELECT start_day < target_day AND
        (end_day IS NULL OR end_day > target_day) AND
        week_day = dow_number_of(target_day) AND
        is_repeated_on(start_day, week_day, every, target_day)
$$ LANGUAGE SQL IMMUTABLE;

CREATE OR REPLACE FUNCTION lesson_permission_for(lesson_id UUID, account_id UUID) RETURNS PermissionType AS $$
    SELECT type FROM LessonPermission WHERE lesson_id = lesson_id AND account_id = account_id
$$ LANGUAGE SQL STABLE;

SELECT '2020-08-18'::date IN '2020-08-18T15:00:00'::timestamp 

-- SELECT to_number('MON');

-- SELECT to_number(WeekDay 'MON');

-- SELECT WeekDay_to_number('MON');



    


    

          