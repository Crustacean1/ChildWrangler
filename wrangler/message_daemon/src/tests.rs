#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn cancellation_respects_grace_period() {
        let student_id = Uuid::new_v4();
        let meal_1_id = Uuid::new_v4();

        let request = CancellationRequest {
            since: NaiveDate::from_ymd_opt(2025, 01, 01).unwrap(),
            until: NaiveDate::from_ymd_opt(2025, 01, 03).unwrap(),
            students: vec![student_id],
            meals: vec![meal_1_id],
        };
        let students = vec![Student {
            id: student_id,
            name: String::new(),
            surname: String::new(),
            grace_period: NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            meals: vec![Meal {
                id: meal_1_id,
                name: String::new(),
            }],
            starts: NaiveDate::from_ymd_opt(2024, 12, 01).unwrap(),
            ends: NaiveDate::from_ymd_opt(2025, 12, 01).unwrap(),
        }];
        let message = Message {
            id: 0,
            sender: String::new(),
            content: String::new(),
            arrived: NaiveDateTime::parse_from_str("2025-01-01 07:01:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
        };

        let cancellation = into_cancellations(&request, &students, &message);

        assert!(cancellation.students.len() == 1);
        assert!(cancellation.students[0].since == NaiveDate::from_ymd_opt(2025, 01, 02).unwrap());
        assert!(cancellation.students[0].until == NaiveDate::from_ymd_opt(2025, 01, 03).unwrap());
    }
}
