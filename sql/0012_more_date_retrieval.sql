DROP FUNCTION first_occurence;
DROP FUNCTION is_repeated_on;
DROP FUNCTION repeats_on_date;

CREATE OR REPLACE FUNCTION first_occurence_weekly(start_day date, week_day SMALLINT) RETURNS date AS $$
    SELECT start_day + (week_day - dow_number_of(start_day) + 7) % 7
$$ LANGUAGE SQL IMMUTABLE;

CREATE OR REPLACE FUNCTION is_repeated_on_weekly(start_day DATE, week_day SMALLINT, every INTEGER, target_day DATE ) RETURNS BOOLEAN AS $$
    SELECT (target_day - first_occurence_weekly(start_day, week_day)) % every = 0
$$ LANGUAGE SQL IMMUTABLE;

CREATE OR REPLACE FUNCTION repeats_on_date_weekly(start_day DATE, end_day DATE, week_day SMALLINT, every INTEGER, target_day DATE) RETURNS BOOLEAN AS $$
    SELECT start_day <= target_day AND
        (end_day IS NULL OR end_day >= target_day) AND
        week_day = dow_number_of(target_day) AND
        is_repeated_on_weekly(start_day, week_day, every, target_day)
$$ LANGUAGE SQL IMMUTABLE;

CREATE OR REPLACE FUNCTION datediff_month(a DATE, b DATE) RETURNS INTEGER AS $$
    SELECT ((EXTRACT(year FROM a) - EXTRACT(year FROM b)) * 12 + (EXTRACT(month FROM a) - EXTRACT(month FROM b)))::INTEGER;
$$ LANGUAGE SQL IMMUTABLE;

CREATE OR REPLACE FUNCTION repeats_on_date_monthly(start_date DATE, end_date DATE, scheduled_time DATE, every INTEGER, target_date DATE) RETURNS BOOLEAN AS $$
    SELECT target_date >= start_date AND
        (end_date IS NULL OR end_date >= target_date) AND
        EXTRACT(day FROM target_date) = EXTRACT(day FROM scheduled_time) AND
        datediff_month(target_date, scheduled_time) % every = 0
$$ LANGUAGE SQL IMMUTABLE;

CREATE OR REPLACE FUNCTION repeats_on_date_daily(start_date DATE, end_date DATE, target_date DATE) RETURNS BOOLEAN AS $$
    SELECT target_date >= start_date AND (end_date IS NULL OR end_date >= target_date)
$$ LANGUAGE SQL IMMUTABLE;
