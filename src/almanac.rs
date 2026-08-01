/// Chinese almanac: 天干地支 + 建除十二神 + 宜忌
/// Pure offline calculation, no network needed.

const HEAVENLY_STEMS: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
const EARTHLY_BRANCHES: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
const WEEKDAYS: [&str; 7] = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

/// 建除十二神
const DAY_OFFICERS: [(&str, &str, &str); 12] = [
    ("建", "出行 上任 祈福", "安葬 动土 开仓"),
    ("除", "就医 扫除 出行", "嫁娶 开业 安葬"),
    ("满", "祈福 开业 嫁娶", "动土 安葬 诉讼"),
    ("平", "修造 出行 会友", "嫁娶 祈福 动土"),
    ("定", "祈福 嫁娶 开业", "诉讼 出行 动土"),
    ("执", "修造 安葬 收财", "开业 出行 嫁娶"),
    ("破", "求医 破土 拆卸", "开业 嫁娶 出行"),
    ("危", "祈福 安葬 修造", "开业 出行 嫁娶"),
    ("成", "开业 出行 祈福 嫁娶", "诉讼 动土 安葬"),
    ("收", "收财 入库 出行", "开业 安葬 动土"),
    ("开", "开业 出行 祈福 嫁娶", "安葬 动土 破土"),
    ("闭", "安葬 修造 收财", "开业 出行 祈福"),
];

pub struct AlmanacInfo {
    pub date_str: String,
    pub weekday: String,
    pub time_str: String,
    pub year_ganzhi: String,
    pub month_ganzhi: String,
    pub day_ganzhi: String,
    pub officer_name: String,
    pub yi: String,
    pub ji: String,
}

pub fn get_almanac() -> AlmanacInfo {
    let now = chrono::Local::now();
    let date = now.date_naive();

    let weekday = WEEKDAYS[now.weekday().num_days_from_sunday() as usize].to_string();
    let date_str = now.format("%Y-%m-%d").to_string();
    let time_str = now.format("%H:%M:%S").to_string();

    let year = date.year();
    let month = date.month() as i32;
    let _day = date.day() as i64;

    // 年干支 (以立春为界简化为公历年)
    let year_stem = (year - 4) % 10;
    let year_branch = (year - 4) % 12;
    let year_ganzhi = format!("{}{}", HEAVENLY_STEMS[year_stem as usize], EARTHLY_BRANCHES[year_branch as usize]);

    // 月干支 (简化：以公历月份近似)
    let month_branch = (month + 1) % 12; // 寅月=1月 近似
    let month_stem = (year_stem * 2 + month) % 10;
    let month_ganzhi = format!("{}{}", HEAVENLY_STEMS[month_stem as usize], EARTHLY_BRANCHES[month_branch as usize]);

    // 日干支 (基准: 1900-01-01 = 甲子日, 序号0)
    let base = chrono::NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
    let days_since_base = (date - base).num_days();
    let day_stem = ((days_since_base % 10) + 10) % 10;
    let day_branch = ((days_since_base % 12) + 12) % 12;
    let day_ganzhi = format!("{}{}", HEAVENLY_STEMS[day_stem as usize], EARTHLY_BRANCHES[day_branch as usize]);

    // 建除十二神: (日支 - 月支) mod 12
    let officer_idx = (((day_branch - month_branch as i64) % 12) + 12) % 12;
    let (officer_name, yi, ji) = DAY_OFFICERS[officer_idx as usize];

    AlmanacInfo {
        date_str,
        weekday,
        time_str,
        year_ganzhi,
        month_ganzhi,
        day_ganzhi,
        officer_name: officer_name.to_string(),
        yi: yi.to_string(),
        ji: ji.to_string(),
    }
}

use chrono::Datelike;

/// Format almanac as a compact one-line string for TUI header
#[allow(dead_code)]
pub fn format_header_line(info: &AlmanacInfo, show_bazi: bool) -> String {
    let mut parts = vec![
        format!("{} {} {}", info.date_str, info.weekday, info.time_str),
    ];

    if show_bazi {
        parts.push(format!("{}年{}月{}日", info.year_ganzhi, info.month_ganzhi, info.day_ganzhi));
        parts.push(format!("{}日 宜:{} 忌:{}", info.officer_name, info.yi, info.ji));
    }

    parts.join(" │ ")
}
