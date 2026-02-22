# `now` - Multi-timezone Clock

A lean, fast Rust CLI for displaying current times across multiple timezones with a clean, compact format.

## Building

```bash
cargo build --release
```

The binary will be at `target/release/now`.

## Usage

```bash
# Basic usage - shows current time in all configured zones
now

# Use 24-hour format
now --24h

# Use 12-hour format
now --12h

# Enable emoji indicators (day/night)
now --emoji

# Disable emoji indicators
now --no-emoji

# Specify a custom reference time (ISO-8601 local time)
now --reference 2026-02-21T10:30

# Use a custom config file
now --config /path/to/config.yaml

# Interactive mode - use arrow keys to shift time, 'q' to quit
now -i
# or
now --interactive
```

## Configuration

On first run, `now` creates a default configuration at `~/.config/now/timezones.yaml`. Edit this file to customize your timezones.

### Config Schema

```yaml
config:
  hour24: bool           # Default: false (use 12-hour format)
  emoji: bool            # Default: false (show day/night emoji)
  local_label: string    # Optional label for the local timezone
  day_emoji: string      # Default: "🌞" (shown for hours 7-18)
  night_emoji: string    # Default: "🌙" (shown for other hours)

timezones:
  - label: "Label Name"
    zone: "IANA/Timezone"
    lat: 37.7749         # Optional (used with emoji-sun feature)
    lon: -122.4194       # Optional (used with emoji-sun feature)
```

#### Emoji Customization

You can customize the day and night emoji indicators by setting `day_emoji` and `night_emoji` in the config section. These can be any Unicode character or emoji:

```yaml
config:
  emoji: true
  day_emoji: "☀️"        # Use a different day emoji
  night_emoji: "🌛"     # Use a different night emoji
```

This allows you to use any symbols you prefer, such as text (e.g., "D"/"N") or other emojis.

## Features

### emoji-sun

Enables accurate sunrise/sunset calculations based on geographic coordinates. Without this feature, emoji indicators use a simple hour-based heuristic (7-18 = day, else night).

Build with the feature:
```bash
cargo build --release --features emoji-sun
```

## Output Format

Each timezone entry displays:
- **Label**: Left-aligned timezone label
- **Time**: Right-aligned current time (emoji appended if enabled)
- **Subline**: Day indicator (Today/Tomorrow/Yesterday) and hour offset

Example output (12h):
```
My Local Time                         03:03 PM 🌞
Today

Cali Office                           07:03 AM 🌞
Today, 8 hours behind

UTC                                   03:03 PM 🌞
Today
```

Example output (24h):
```
My Local Time                         15:03 🌞
Today

Cali Office                           07:03 🌞
Today, 8 hours behind
```

## Interactive Mode

In interactive mode (`-i`), use:
- **LEFT arrow**: Shift time back by 1 hour
- **RIGHT arrow**: Shift time forward by 1 hour
- **q**: Quit

The display updates in real-time as you navigate through time.

## Dependencies

Core dependencies (minimal and stable):
- `chrono` - Time and date handling
- `chrono-tz` - Timezone database
- `serde` & `serde_yaml` - YAML configuration parsing
- `pico-args` - Lightweight CLI argument parsing
- `crossterm` - Terminal control for interactive mode

Optional:
- `sunrise` - Accurate sunrise/sunset calculations (feature: `emoji-sun`)

## Troubleshooting

**Config file not found**: The app automatically creates `~/.config/now/timezones.yaml` on first run with sensible defaults.

**Invalid timezone error**: Verify your timezone strings are valid IANA timezone identifiers (e.g., "America/Los_Angeles", "Europe/London", "Asia/Tokyo").

**Terminal issues in interactive mode**: The app uses `crossterm` for terminal control. If you experience issues, try a different terminal emulator.
