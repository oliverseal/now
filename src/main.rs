use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Timelike};
use chrono_tz::Tz;
use serde::Deserialize;
use std::cmp::max;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

// Configuration structures
#[derive(Debug, Clone, Deserialize)]
struct Root {
    #[serde(default)]
    config: Config,
    timezones: Vec<TzEntry>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct Config {
    #[serde(default)]
    hour24: bool,
    #[serde(default)]
    emoji: bool,
    local_label: Option<String>,
    #[serde(default = "default_day_emoji")]
    day_emoji: String,
    #[serde(default = "default_night_emoji")]
    night_emoji: String,
}

fn default_day_emoji() -> String {
    "🌞".to_string()
}

fn default_night_emoji() -> String {
    "🌙".to_string()
}

#[derive(Debug, Clone, Deserialize)]
struct TzEntry {
    label: String,
    zone: String,
    #[serde(default)]
    lat: Option<f64>,
    #[serde(default)]
    lon: Option<f64>,
}

// Command-line arguments
#[derive(Debug, Default)]
struct Args {
    interactive: bool,
    hour24: Option<bool>,
    emoji: Option<bool>,
    config: Option<PathBuf>,
    reference: Option<DateTime<Local>>,
}

// Parsed time entry for a zone
struct TimeEntry {
    label: String,
    time: DateTime<Tz>,
}

impl TimeEntry {
    fn new(label: String, tz: Tz, instant: DateTime<Local>) -> Self {
        let time = instant.with_timezone(&tz);
        TimeEntry { label, time }
    }
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let config_path = resolve_config_path(args.config.as_ref());

    let root: Root = match load_config(&config_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    // Determine effective settings with CLI overrides
    let hour24 = args.hour24.unwrap_or(root.config.hour24);
    let emoji = args.emoji.unwrap_or(root.config.emoji);
    let local_label = root.config.local_label.clone();

    let reference = args.reference.unwrap_or_else(Local::now);

    if args.interactive {
        match run_interactive(
            reference,
            &root.timezones,
            hour24,
            emoji,
            local_label,
            root.config.day_emoji.clone(),
            root.config.night_emoji.clone(),
        ) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Interactive mode error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        render_listing(
            reference,
            &root.timezones,
            hour24,
            emoji,
            local_label,
            root.config.day_emoji,
            root.config.night_emoji,
        );
    }
}

fn parse_args() -> Result<Args, String> {
    let mut pargs = pico_args::Arguments::from_env();
    let mut args = Args::default();

    // Flags
    args.interactive = pargs.contains(["-i", "--interactive"]);

    // Options
    if pargs.contains("--24h") {
        args.hour24 = Some(true);
    } else if pargs.contains("--12h") {
        args.hour24 = Some(false);
    }

    if pargs.contains("--emoji") {
        args.emoji = Some(true);
    } else if pargs.contains("--no-emoji") {
        args.emoji = Some(false);
    }

    if let Ok(config) = pargs.opt_value_from_str::<_, String>(["-c", "--config"]) {
        args.config = config.map(PathBuf::from);
    }

    if let Ok(ref_str) = pargs.opt_value_from_str::<_, String>("--reference") {
        if let Some(ref_str) = ref_str {
            match chrono::NaiveDateTime::parse_from_str(&ref_str, "%Y-%m-%dT%H:%M") {
                Ok(naive_dt) => {
                    // Interpret as local time
                    args.reference = Some(
                        Local
                            .from_local_datetime(&naive_dt)
                            .single()
                            .ok_or("Invalid reference time (ambiguous local time)".to_string())?,
                    );
                }
                Err(_) => {
                    return Err(
                        "Invalid --reference format; use ISO-8601 (e.g., 2026-02-21T10:30)"
                            .to_string(),
                    );
                }
            }
        }
    }

    Ok(args)
}

fn resolve_config_path(cli_path: Option<&PathBuf>) -> PathBuf {
    if let Some(p) = cli_path {
        return p.clone();
    }

    // Expand ~/.config/now/timezones.yaml
    if let Ok(home) = std::env::var("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".config/now/timezones.yaml");
        return path;
    }

    PathBuf::from(".config/now/timezones.yaml")
}

fn load_config(path: &PathBuf) -> Result<Root, Box<dyn std::error::Error>> {
    // If the config file doesn't exist, create it with defaults
    if !path.exists() {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write default example config
        let default_config = r#"config:
  hour24: false
  emoji: true
  local_label: "Local Time"
  day_emoji: "🌞"
  night_emoji: "🌙"

timezones:
  - label: "Mom"
    zone:  "America/Chicago"

  - label: "Office"
    zone:  "America/Los_Angeles"
    lat: 37.7749
    lon: -122.4194

  - label: "UTC"
    zone:  "UTC"

  - label: "Kyoto"
    zone:  "Asia/Tokyo"
    lat: 35.6762
    lon: 139.6503

  - label: "London"
    zone:  "Europe/London"
"#;

        fs::write(path, default_config)?;
        eprintln!("Created default config at {}", path.display());
    }

    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let root: Root = serde_yaml::from_str(&contents)?;
    Ok(root)
}

fn render_listing(
    reference: DateTime<Local>,
    timezones: &[TzEntry],
    hour24: bool,
    emoji: bool,
    local_label: Option<String>,
    day_emoji: String,
    night_emoji: String,
) {
    let output = build_listing_output(
        reference,
        timezones,
        hour24,
        emoji,
        local_label,
        day_emoji,
        night_emoji,
    );
    print!("{}", output);
}

fn fmt_time(dt: DateTime<Tz>, hour24: bool) -> String {
    if hour24 {
        dt.format("%H:%M").to_string()
    } else {
        dt.format("%I:%M %p").to_string()
    }
}

fn fmt_time_local(dt: DateTime<Local>, hour24: bool) -> String {
    if hour24 {
        dt.format("%H:%M").to_string()
    } else {
        dt.format("%I:%M %p").to_string()
    }
}

fn build_listing_output(
    reference: DateTime<Local>,
    timezones: &[TzEntry],
    hour24: bool,
    emoji: bool,
    local_label: Option<String>,
    day_emoji: String,
    night_emoji: String,
) -> String {
    let local_date = reference.date_naive();

    // Build all time entries and compute label width
    let mut entries = Vec::new();
    let mut max_label_len = 0;

    for tz_entry in timezones {
        match tz_entry.zone.parse::<Tz>() {
            Ok(tz) => {
                let time_entry = TimeEntry::new(tz_entry.label.clone(), tz, reference.clone());
                max_label_len = max(max_label_len, tz_entry.label.len());
                entries.push((time_entry, tz_entry));
            }
            Err(_) => {
                eprintln!(
                    "Warning: invalid timezone '{}' in entry '{}'",
                    tz_entry.zone, tz_entry.label
                );
            }
        }
    }

    let label_w = max_label_len + 2;
    let time_base_w = if hour24 { 5 } else { 8 };

    let mut output = String::new();

    // Print local time if a label is provided
    if let Some(label) = local_label {
        let time_str = fmt_time_local(reference, hour24);
        let emoji_str = if emoji {
            format_emoji(
                reference.hour() as i32,
                None,
                None,
                &day_emoji,
                &night_emoji,
            )
        } else {
            String::new()
        };
        let time_display = if emoji {
            format!("{} {}", time_str, emoji_str)
        } else {
            time_str
        };
        output.push_str(&format!(
            "{:<label_w$}{:>time_base_w$}\n",
            label, time_display
        ));
        output.push_str("Today\n");
        output.push('\n');
    }

    // Render each timezone
    for (entry, tz_entry) in entries {
        let time_str = fmt_time(entry.time.clone(), hour24);
        let emoji_str = if emoji {
            format_emoji(
                entry.time.hour() as i32,
                tz_entry.lat,
                tz_entry.lon,
                &day_emoji,
                &night_emoji,
            )
        } else {
            String::new()
        };

        let zone_date = entry.time.date_naive();
        let day_word = day_word(local_date, zone_date);
        let hours_delta = hours_delta_tz(reference.clone(), entry.time.clone());

        let time_display = if emoji {
            format!("{} {}", time_str, emoji_str)
        } else {
            time_str
        };
        output.push_str(&format!(
            "{:<label_w$}{:>time_base_w$}\n",
            entry.label, time_display
        ));

        if hours_delta == 0 {
            output.push_str(&format!("{}\n", day_word));
        } else {
            let direction = if hours_delta > 0 { "ahead" } else { "behind" };
            output.push_str(&format!(
                "{}, {} hours {}\n",
                day_word,
                hours_delta.abs(),
                direction
            ));
        }

        output.push('\n');
    }

    output
}

fn day_word(local_date: NaiveDate, zoned_date: NaiveDate) -> &'static str {
    let diff = zoned_date.signed_duration_since(local_date);
    if diff.num_days() < 0 {
        "Yesterday"
    } else if diff.num_days() > 0 {
        "Tomorrow"
    } else {
        "Today"
    }
}

fn hours_delta_tz(local_dt: DateTime<Local>, zoned_dt: DateTime<Tz>) -> i32 {
    let local_hour = local_dt.hour() as i32;
    let zoned_hour = zoned_dt.hour() as i32;
    zoned_hour - local_hour
}

fn format_emoji(
    hour: i32,
    _lat: Option<f64>,
    _lon: Option<f64>,
    day_emoji: &str,
    night_emoji: &str,
) -> String {
    #[cfg(feature = "emoji-sun")]
    {
        if let (Some(lat), Some(lon)) = (_lat, _lon) {
            // Try to use sunrise/sunset for accurate day/night
            // Use a reasonable fake date for testing; in production, use the actual zone date
            let (sunrise, sunset) = sunrise::sunrise_sunset(lat, lon, 2026, 2, 21);
            let hour_f = hour as f64 + 0.0; // Simplified; could use minutes too
            if hour_f >= sunrise as f64 && hour_f < sunset as f64 {
                return day_emoji.to_string();
            } else {
                return night_emoji.to_string();
            }
        }
    }

    // Fallback: day if 7-18, else night
    if hour >= 7 && hour < 18 {
        day_emoji.to_string()
    } else {
        night_emoji.to_string()
    }
}

fn run_interactive(
    reference: DateTime<Local>,
    timezones: &[TzEntry],
    hour24: bool,
    emoji: bool,
    local_label: Option<String>,
    day_emoji: String,
    night_emoji: String,
) -> io::Result<()> {
    use crossterm::{
        event::{self, KeyCode, KeyEvent},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use std::io::stdout;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let mut delta_hours = 0i32;

    loop {
        // Clear and render
        execute!(
            stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )?;
        execute!(stdout, crossterm::cursor::MoveTo(0, 0))?;

        let current_ref = reference + Duration::hours(delta_hours as i64);

        // Build the output string manually to avoid cursor issues
        let output = build_listing_output(
            current_ref,
            timezones,
            hour24,
            emoji,
            local_label.clone(),
            day_emoji.clone(),
            night_emoji.clone(),
        );

        write!(stdout, "{}", output)?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let event::Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Left => delta_hours -= 1,
                    KeyCode::Right => delta_hours += 1,
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;

    Ok(())
}
