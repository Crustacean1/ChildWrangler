CREATE TABLE guardians(
	id uuid primary key default gen_random_uuid(),
	removed bool default false,
	fullname text not null,
	phone text,
	unique (fullname)
);

CREATE TABLE groups(
	id uuid primary key not null default gen_random_uuid(),
	removed bool default false,
	name text not null
);

CREATE TABLE caterings(
	id uuid primary key default gen_random_uuid(),
	group_id uuid references groups(id) not null,
	grace_period time not null,
	dow smallint not null,
	since date not null,
	until date not null
);

CREATE TABLE group_relations(
	child uuid not null,
	parent uuid not null,
	level smallint not null,
	unique (child,level),
	primary key (child,parent)
);

CREATE INDEX gr_parent_index ON group_relations(parent);
CREATE INDEX gr_child_index ON group_relations(child);

CREATE TABLE allergies(
	id uuid primary key default gen_random_uuid(),
	name text unique not null
);

CREATE TABLE allergy_combinations(
	id uuid,
	allergy_id uuid,
	primary key(id,allergy_id)
);

CREATE TABLE students (
	id uuid primary key default gen_random_uuid(),
	removed bool default false,
	name text not null,
	surname text not null,
	allergy_combination_id uuid
);

CREATE TABLE student_guardians (
	guardian_id uuid references guardians(id) not null,
	student_id uuid references students(id) not null
);

CREATE TABLE meals(
	id uuid primary key default gen_random_uuid(),
	name text unique not null
);

CREATE TABLE catering_meals(
	catering_id uuid references caterings(id),
	meal_id uuid references meals(id),
	meal_order integer not null
);

CREATE TABLE cancellations (
	id uuid primary key default gen_random_uuid(),
	accepted bool,
	details jsonb,
	inbox_id integer,
	outbox_id integer
);

CREATE TABLE processing_info (
	id uuid primary key default gen_random_uuid(),
	cause_id uuid not null,
	reason text,
	value jsonb not null
);

CREATE TABLE processing_trigger (
	message_id integer,
	processing_id uuid
);

CREATE TABLE attendance (
	originated timestamptz not null default NOW(),
	cause_id uuid,
	target uuid not null,
	day date,
	meal_id uuid not null,
	value bool not null
);

CREATE INDEX attendance_composite ON attendance (day, meal_id, target, originated) INCLUDE (value);
CREATE INDEX attendance_day ON attendance (day);
CREATE INDEX attendance_target ON attendance (target);

CREATE TABLE messages (
	id uuid primary key default gen_random_uuid(),
	received timestamptz not null,
	sender text not null,
	content text not null
);


CREATE VIEW effective_attendance AS SELECT DISTINCT ON (day, meal_id, target) day, meal_id, target, value FROM attendance ORDER BY day, meal_id, target, originated, cause_id;
CREATE VIEW rooted_attendance AS SELECT bool_and(effective_attendance.value) AS present, effective_attendance.day, effective_attendance.meal_id, effective_attendance.target as student_id, group_relations.parent AS root FROM group_relations
                        INNER JOIN group_relations AS student_relation ON student_relation.parent = group_relations.child
                        INNER JOIN students ON students.id = student_relation.child
                        INNER JOIN effective_attendance ON effective_attendance.target = student_relation.child
                        GROUP BY effective_attendance.day, effective_attendance.meal_id, effective_attendance.target, group_relations.parent;
