CREATE TABLE guardians(
	id uuid primary key default gen_random_uuid(),
	created timestamp DEFAULT LOCALTIMESTAMP(0),
	removed bool default false,
	fullname text not null,
	phone text,
	unique (fullname),
	unique (phone)
);

CREATE TABLE groups(
	id uuid primary key not null default gen_random_uuid(),
	removed bool default false not null,
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

CREATE TABLE messages (
	id uuid primary key default gen_random_uuid() NOT NULL,
	content text NOT NULL,
	phone text NOT NULL,
	outgoing bool NOT NULL,
	inserted timestamp default NOW() NOT NULL,
	processed bool default false NOT NULL,
	cause_id uuid references messages(id),
	sent timestamp
);

CREATE TABLE processing_step (
	id int primary key generated always as identity,
	completed timestamp,
	cause_id uuid not null references messages(id),
	value jsonb not null
);

CREATE TABLE attendance_override (
	id uuid primary key default gen_random_uuid(),
	created timestamp not null default LOCALTIMESTAMP(0),
	note text
);

CREATE TABLE attendance (
	originated timestamp not null default LOCALTIMESTAMP(0),
	cause_id uuid not null,
	target uuid not null,
	day date not null ,
	meal_id uuid not null,
	value bool not null
);

CREATE INDEX attendance_composite ON attendance (day, meal_id, target, originated) INCLUDE (value);
CREATE INDEX attendance_day ON attendance (day);
CREATE INDEX attendance_target ON attendance (target);

CREATE VIEW effective_attendance AS SELECT DISTINCT ON (day, meal_id, target) day, meal_id, target, value, cause_id FROM attendance ORDER BY day, meal_id, target, originated DESC, cause_id;
CREATE VIEW rooted_attendance AS SELECT bool_and(effective_attendance.value) AS present, effective_attendance.day, effective_attendance.meal_id, student_relation.child as student_id, group_relations.parent AS root FROM group_relations
                        INNER JOIN group_relations AS student_relation ON student_relation.parent = group_relations.child
                        INNER JOIN students ON students.id = student_relation.child AND students.removed = false
                        INNER JOIN effective_attendance ON effective_attendance.target = student_relation.parent
                        GROUP BY effective_attendance.day, effective_attendance.meal_id, student_relation.child, group_relations.parent;
