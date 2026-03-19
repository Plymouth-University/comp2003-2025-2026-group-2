use back_end::logs_db::{Frequency, Schedule};
use chrono::Datelike;

fn daily_schedule(days_of_week: Option<Vec<u8>>) -> Schedule {
    Schedule {
        frequency: Frequency::Daily,
        days_of_week,
        day_of_week: None,
        day_of_month: None,
        month_of_year: None,
        available_from_time: None,
        due_at_time: None,
    }
}

fn weekly_schedule(day_of_week: u8) -> Schedule {
    Schedule {
        frequency: Frequency::Weekly,
        days_of_week: None,
        day_of_week: Some(day_of_week),
        day_of_month: None,
        month_of_year: None,
        available_from_time: None,
        due_at_time: None,
    }
}

fn monthly_schedule(day_of_month: u8) -> Schedule {
    Schedule {
        frequency: Frequency::Monthly,
        days_of_week: None,
        day_of_week: None,
        day_of_month: Some(day_of_month),
        month_of_year: None,
        available_from_time: None,
        due_at_time: None,
    }
}

fn yearly_schedule(month_of_year: u8, day_of_month: u8) -> Schedule {
    Schedule {
        frequency: Frequency::Yearly,
        days_of_week: None,
        day_of_week: None,
        day_of_month: Some(day_of_month),
        month_of_year: Some(month_of_year),
        available_from_time: None,
        due_at_time: None,
    }
}

/// Sunday=0 convention should match the frontend:
///   0=Sunday, 1=Monday, 2=Tuesday, 3=Wednesday,
///   4=Thursday, 5=Friday, 6=Saturday
#[test]
fn test_is_form_due_today_daily_weekdays_only() {
    // Schedule for Monday(1) through Friday(5) only
    let schedule = daily_schedule(Some(vec![1, 2, 3, 4, 5]));

    let today = chrono::Utc::now().weekday();
    let today_num: u8 = match today {
        chrono::Weekday::Sun => 0,
        chrono::Weekday::Mon => 1,
        chrono::Weekday::Tue => 2,
        chrono::Weekday::Wed => 3,
        chrono::Weekday::Thu => 4,
        chrono::Weekday::Fri => 5,
        chrono::Weekday::Sat => 6,
    };

    let result = back_end::logs_db::is_form_due_today(&schedule);
    // Should be due on Mon-Fri (1-5), not on Sun(0) or Sat(6)
    let expected = (1..=5).contains(&today_num);
    assert_eq!(
        result, expected,
        "is_form_due_today mismatch for weekday {today} (num={today_num}): expected {expected}, got {result}"
    );
}

#[test]
fn test_is_form_due_today_daily_every_day() {
    // No days_of_week restriction means due every day
    let schedule = daily_schedule(None);
    assert!(back_end::logs_db::is_form_due_today(&schedule));
}

#[test]
fn test_is_form_due_today_daily_sunday_only() {
    // Schedule for Sunday(0) only
    let schedule = daily_schedule(Some(vec![0]));

    let today = chrono::Utc::now().weekday();
    let result = back_end::logs_db::is_form_due_today(&schedule);

    match today {
        chrono::Weekday::Sun => assert!(result, "Should be due on Sunday"),
        _ => assert!(!result, "Should NOT be due on {today}"),
    }
}

#[test]
fn test_is_form_due_today_daily_saturday_only() {
    // Schedule for Saturday(6) only
    let schedule = daily_schedule(Some(vec![6]));

    let today = chrono::Utc::now().weekday();
    let result = back_end::logs_db::is_form_due_today(&schedule);

    match today {
        chrono::Weekday::Sat => assert!(result, "Should be due on Saturday"),
        _ => assert!(!result, "Should NOT be due on {today}"),
    }
}

#[test]
fn test_is_form_due_today_weekly_monday() {
    // Weekly on Monday (1 in Sunday=0 convention)
    let schedule = weekly_schedule(1);

    let today = chrono::Utc::now().weekday();
    let result = back_end::logs_db::is_form_due_today(&schedule);

    match today {
        chrono::Weekday::Mon => assert!(result, "Should be due on Monday"),
        _ => assert!(!result, "Should NOT be due on {today}"),
    }
}

#[test]
fn test_is_form_due_today_weekly_sunday() {
    // Weekly on Sunday (0 in Sunday=0 convention)
    let schedule = weekly_schedule(0);

    let today = chrono::Utc::now().weekday();
    let result = back_end::logs_db::is_form_due_today(&schedule);

    match today {
        chrono::Weekday::Sun => assert!(result, "Should be due on Sunday"),
        _ => assert!(!result, "Should NOT be due on {today}"),
    }
}

#[test]
fn test_is_form_due_today_weekly_friday() {
    // Weekly on Friday (5 in Sunday=0 convention)
    let schedule = weekly_schedule(5);

    let today = chrono::Utc::now().weekday();
    let result = back_end::logs_db::is_form_due_today(&schedule);

    match today {
        chrono::Weekday::Fri => assert!(result, "Should be due on Friday"),
        _ => assert!(!result, "Should NOT be due on {today}"),
    }
}

#[test]
fn test_get_missed_periods_daily_weekdays() {
    // Daily schedule for Mon-Fri only (1-5 in Sunday=0)
    let schedule = daily_schedule(Some(vec![1, 2, 3, 4, 5]));

    // Use a known 3-day window: set created_at to 2 days ago
    let today = chrono::Utc::now().date_naive();
    let two_days_ago = today - chrono::Duration::days(2);
    let created_at = two_days_ago
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .to_rfc3339();

    let missed = back_end::logs_db::get_missed_periods(&schedule, None, Some(&created_at));

    // Every day between two_days_ago and today (inclusive) that is Mon-Fri
    // should appear in missed periods
    let mut current = two_days_ago;
    while current <= today {
        let day_num: u8 = match current.weekday() {
            chrono::Weekday::Sun => 0,
            chrono::Weekday::Mon => 1,
            chrono::Weekday::Tue => 2,
            chrono::Weekday::Wed => 3,
            chrono::Weekday::Thu => 4,
            chrono::Weekday::Fri => 5,
            chrono::Weekday::Sat => 6,
        };
        let date_str = format!(
            "{:02}/{:02}/{:04}",
            current.day(),
            current.month(),
            current.year()
        );
        if (1..=5).contains(&day_num) {
            assert!(
                missed.contains(&date_str),
                "Expected missed period {date_str} (weekday num {day_num}) for {current}"
            );
        } else {
            assert!(
                !missed.contains(&date_str),
                "Did NOT expect missed period {date_str} (weekday num {day_num}) for {current}"
            );
        }
        current += chrono::Duration::days(1);
    }
}

#[test]
fn test_get_missed_periods_daily_sunday_only() {
    // Daily schedule for Sunday(0) only
    let schedule = daily_schedule(Some(vec![0]));

    let today = chrono::Utc::now().date_naive();
    let two_days_ago = today - chrono::Duration::days(2);
    let created_at = two_days_ago
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .to_rfc3339();

    let missed = back_end::logs_db::get_missed_periods(&schedule, None, Some(&created_at));

    let mut current = two_days_ago;
    while current <= today {
        let date_str = format!(
            "{:02}/{:02}/{:04}",
            current.day(),
            current.month(),
            current.year()
        );
        if current.weekday() == chrono::Weekday::Sun {
            assert!(
                missed.contains(&date_str),
                "Expected missed period {date_str} for Sunday"
            );
        } else {
            assert!(
                !missed.contains(&date_str),
                "Did NOT expect missed period {date_str} for {}",
                current.weekday()
            );
        }
        current += chrono::Duration::days(1);
    }
}

#[test]
fn test_format_period_for_frequency_weekly_starts_sunday() {
    // format_period_for_frequency for Weekly should produce a week period
    // where the week starts on Sunday
    let period = back_end::logs_db::format_period_for_frequency(&Frequency::Weekly);

    // The period format is "start_day-end_day/month/year"
    let parts: Vec<&str> = period.split('/').collect();
    assert_eq!(
        parts.len(),
        3,
        "Weekly period should have 3 parts: day_range/month/year"
    );

    let day_range: Vec<&str> = parts[0].split('-').collect();
    assert_eq!(day_range.len(), 2, "Day range should have start and end");

    let start_day: u32 = day_range[0].parse().expect("start day should be a number");
    let end_day: u32 = day_range[1].parse().expect("end day should be a number");

    // Week span is 7 days, so end_day - start_day should be 6
    // (unless crossing month boundary, which we won't test here)
    // Also verify the period is consistent: parsing it back should give a Sunday start
    let month: u32 = parts[1].parse().expect("month should be a number");
    let year: i32 = parts[2].parse().expect("year should be a number");

    if let Some(week_start) = chrono::NaiveDate::from_ymd_opt(year, month, start_day) {
        assert_eq!(
            week_start.weekday(),
            chrono::Weekday::Sun,
            "Weekly period start day ({start_day}/{month}/{year}) should be Sunday, got {}",
            week_start.weekday()
        );

        if let Some(week_end) = chrono::NaiveDate::from_ymd_opt(year, month, end_day) {
            assert_eq!(
                week_end.weekday(),
                chrono::Weekday::Sat,
                "Weekly period end day ({end_day}/{month}/{year}) should be Saturday, got {}",
                week_end.weekday()
            );
        }
    }
}

#[test]
fn test_get_missed_periods_weekly_no_period_before_created_at() {
    use back_end::logs_db::compute_due_date_for_period;

    let schedule = weekly_schedule(1);

    let created_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
    let created_at = created_date
        .and_hms_opt(14, 30, 0)
        .unwrap()
        .and_utc()
        .to_rfc3339();

    let missed = back_end::logs_db::get_missed_periods(&schedule, None, Some(&created_at));

    assert!(
        !missed.is_empty(),
        "Should have at least one missed period for weekly Monday schedule"
    );

    for period in &missed {
        let due_date = compute_due_date_for_period(&schedule, period);
        assert!(
            due_date.is_some() && due_date.unwrap() >= created_date,
            "Period {period} has due date {:?}, which is before created_at {created_date}",
            due_date
        );
    }
}

#[test]
fn test_get_missed_periods_weekly_created_on_target_day() {
    use back_end::logs_db::compute_due_date_for_period;

    let schedule = weekly_schedule(1);

    let created_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 13).unwrap();
    let created_at = created_date
        .and_hms_opt(14, 30, 0)
        .unwrap()
        .and_utc()
        .to_rfc3339();

    let missed = back_end::logs_db::get_missed_periods(&schedule, None, Some(&created_at));

    assert!(
        !missed.is_empty(),
        "Should have at least one missed period when created on target weekday"
    );

    for period in &missed {
        let due_date = compute_due_date_for_period(&schedule, period);
        assert!(
            due_date.is_some() && due_date.unwrap() >= created_date,
            "Period {period} has due date {:?}, which is before created_at {created_date}",
            due_date
        );
    }
}

#[test]
fn test_get_missed_periods_weekly_with_last_submission() {
    use back_end::logs_db::compute_due_date_for_period;

    let schedule = weekly_schedule(1);

    let last_period = "12-18/1/2025";
    let last_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 18).unwrap();

    let missed = back_end::logs_db::get_missed_periods(&schedule, Some(last_period), None);

    assert!(
        !missed.is_empty(),
        "Should have missed periods after last submission"
    );

    for period in &missed {
        let due_date = compute_due_date_for_period(&schedule, period);
        assert!(
            due_date.is_some() && due_date.unwrap() > last_date,
            "Period {period} has due date {:?}, which is not after last submission {last_date}",
            due_date
        );
    }
}

#[test]
fn test_validate_and_normalize_period_daily_rejects_non_daily_shapes() {
    let schedule = daily_schedule(None);

    assert_eq!(
        back_end::logs_db::validate_and_normalize_period(&schedule, "2026"),
        None,
        "Daily period must not accept yearly format"
    );
    assert_eq!(
        back_end::logs_db::validate_and_normalize_period(&schedule, "29-4/4/2026"),
        None,
        "Daily period must not accept weekly format"
    );
    assert_eq!(
        back_end::logs_db::validate_and_normalize_period(&schedule, "4/4/2026"),
        Some("04/04/2026".to_string()),
        "Daily period should accept DD/MM/YYYY format"
    );
}

#[test]
fn test_get_missed_periods_monthly_no_period_before_created_at() {
    use back_end::logs_db::compute_due_date_for_period;

    let schedule = monthly_schedule(5);

    let created_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();
    let created_at = created_date
        .and_hms_opt(14, 30, 0)
        .unwrap()
        .and_utc()
        .to_rfc3339();

    let missed = back_end::logs_db::get_missed_periods(&schedule, None, Some(&created_at));

    assert!(
        !missed.is_empty(),
        "Should have at least one missed period for monthly schedule"
    );

    for period in &missed {
        let due_date = compute_due_date_for_period(&schedule, period);
        assert!(
            due_date.is_some() && due_date.unwrap() >= created_date,
            "Period {period} has due date {:?}, which is before created_at {created_date}",
            due_date
        );
    }
}

#[test]
fn test_get_missed_periods_yearly_no_period_before_created_at() {
    use back_end::logs_db::compute_due_date_for_period;

    let schedule = yearly_schedule(1, 5);

    let created_date = chrono::NaiveDate::from_ymd_opt(2025, 2, 20).unwrap();
    let created_at = created_date
        .and_hms_opt(14, 30, 0)
        .unwrap()
        .and_utc()
        .to_rfc3339();

    let missed = back_end::logs_db::get_missed_periods(&schedule, None, Some(&created_at));

    assert!(
        !missed.is_empty(),
        "Should have at least one missed period for yearly schedule"
    );

    for period in &missed {
        let due_date = compute_due_date_for_period(&schedule, period);
        assert!(
            due_date.is_some() && due_date.unwrap() >= created_date,
            "Period {period} has due date {:?}, which is before created_at {created_date}",
            due_date
        );
    }
}
