DELETE VIEW total_attendance;
-- Add migration script here
CREATE VIEW total_attendance AS SELECT DISTINCT ON (day, meal_id, students.id) effective_attendance.value, effective_attendance.day, effective_attendance.meal_id, students.id AS student_id, effective_attendance.cause_id FROM students
						INNER JOIN group_relations ON group_relations.child = students.id
						INNER JOIN effective_attendance ON effective_attendance.target = group_relations.parent
						WHERE student.removed = false
ORDER BY day, meal_id, students.id, value, level;
