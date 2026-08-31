use anyhow::Context;
use clap::ArgGroup;

#[derive(clap::Args, Debug)]
#[command(group(
    ArgGroup::new("duration")
        .args(["from_date", "to_date"])
        .multiple(true)
))]
pub struct UpdateArgs {
    #[arg(
        long = "from",
        short = 'f',
        value_name = "DATE",
        help = "取り込み期間の開始日を指定します（YYYY-MM-DD、`--to` と併用）"
    )]
    pub from_date: Option<String>,
    #[arg(
        long = "to",
        short = 't',
        value_name = "DATE",
        help = "取り込み期間の終了日を指定します（YYYY-MM-DD、`--from` と併用）"
    )]
    pub to_date: Option<String>,
    #[arg(
        long,
        short = 'y',
        value_name = "N",
        conflicts_with_all = ["duration", "today"],
        help = "今日を含む直近 N 年の未更新日を取り込みます（N は 1 以上）"
    )]
    pub years: Option<std::num::NonZeroU32>,
    #[arg(
        long,
        short = 'd',
        conflicts_with = "duration",
        help = "今日の書類情報だけを取り込みます"
    )]
    pub today: bool,
    #[arg(
        long,
        short = 's',
        conflicts_with = "concurrency",
        help = "日次 API 取得を直列実行します"
    )]
    pub sequential: bool,
    #[arg(
        long,
        short = 'F',
        help = "更新済み日付を無視して対象期間をすべて再取得します"
    )]
    pub force: bool,
    #[arg(
        long,
        short = 'p',
        value_name = "N",
        default_value = "4",
        help = "日次 API 取得の最大並列数を指定します（N は 1 以上）"
    )]
    pub concurrency: std::num::NonZeroUsize,
    #[arg(
        long,
        short = 'k',
        value_name = "API_KEY",
        help = "この実行で使う EDINET API キー（登録済みキーより優先）"
    )]
    pub key: Option<String>,
}

pub async fn run(args: UpdateArgs) -> anyhow::Result<()> {
    let concurrency = if args.sequential {
        1
    } else {
        args.concurrency.get()
    };
    let years_range = args
        .years
        .map(|years| {
            let today = chrono::Local::now().date_naive();
            calculate_year_window_start(&today, years).map(|start_date| (start_date, today))
        })
        .transpose()?;
    let db = crate::store::open_db::open_db_connection().await?;
    let api_key = crate::app_config::resolve_api_key(args.key.as_deref())?;
    if args.today {
        let today = chrono::Local::now().date_naive();
        crate::updater::update_documents::update_one_day(&db, &today, &api_key).await?;
        return Ok(());
    }
    if let Some((start_date, today)) = years_range {
        crate::updater::update_documents::update_days(
            &db,
            &start_date,
            &today,
            &api_key,
            concurrency,
            args.force,
        )
        .await?;
        return Ok(());
    }
    if args.from_date.is_none() && args.to_date.is_none() {
        let today = chrono::Local::now().date_naive();
        crate::updater::update_documents::update_recent_days(
            &db,
            &today,
            &api_key,
            concurrency,
            args.force,
        )
        .await?;
        return Ok(());
    }
    if args.from_date.is_none() || args.to_date.is_none() {
        return Err(anyhow::anyhow!("--from and --to must be used together"));
    }
    let from_date = parse_date(
        args.from_date
            .as_deref()
            .expect("--from should be present after validation"),
    )?;
    let to_date = parse_date(
        args.to_date
            .as_deref()
            .expect("--to should be present after validation"),
    )?;
    crate::updater::update_documents::update_range(
        &db,
        &from_date,
        &to_date,
        &api_key,
        concurrency,
    )
    .await?;

    Ok(())
}

fn parse_date(date: &str) -> anyhow::Result<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .with_context(|| format!("failed to parse date: {}", date))
}

fn calculate_year_window_start(
    today: &chrono::NaiveDate,
    years: std::num::NonZeroU32,
) -> anyhow::Result<chrono::NaiveDate> {
    let months = years
        .get()
        .checked_mul(12)
        .context("year count is too large")?;
    today
        .checked_sub_months(chrono::Months::new(months))
        .and_then(|date| date.succ_opt())
        .context("failed to calculate update start date")
}

#[cfg(test)]
mod tests {
    use super::calculate_year_window_start;

    #[test]
    fn calculates_inclusive_calendar_year_window() {
        let today = chrono::NaiveDate::from_ymd_opt(2026, 8, 11).unwrap();
        let years = std::num::NonZeroU32::new(2).unwrap();

        assert_eq!(
            calculate_year_window_start(&today, years).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2024, 8, 12).unwrap()
        );
    }

    #[test]
    fn calculates_calendar_year_window_across_leap_day() {
        let today = chrono::NaiveDate::from_ymd_opt(2025, 2, 28).unwrap();
        let years = std::num::NonZeroU32::new(1).unwrap();

        assert_eq!(
            calculate_year_window_start(&today, years).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        );
    }
}
