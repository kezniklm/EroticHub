use chrono::{Local, NaiveDateTime, TimeZone};

pub fn format_datetime(naive: NaiveDateTime) -> String {
    let local_datetime = Local.from_utc_datetime(&naive);
    local_datetime.format("%d.%m.%Y %H:%M").to_string()
}
