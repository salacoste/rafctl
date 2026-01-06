# Quota Monitoring

The `rafctl quota` command displays your Claude API usage limits and current utilization.

## Usage

```bash
# Show quota for all authenticated profiles
rafctl quota

# Show quota for a specific profile
rafctl quota <profile>

# JSON output for scripting
rafctl quota --json

# Plain output (no colors)
rafctl quota --plain
```

## Output

The command shows two usage windows:

| Window | Description |
|--------|-------------|
| **5-hour** | Rolling 5-hour usage limit |
| **7-day** | Rolling 7-day usage limit |

### Example Output

```
Profile: work
═══════════════════════════════════════════════════

5-hour usage:
  ████████░░░░░░░░░░░░  42.0%
  Resets at: 2026-01-06 23:00 (in 2h 15m)

7-day usage:
  ██████████████░░░░░░  68.5%
  Resets at: 2026-01-10 03:00 (in 3d 5h)
```

### Color Coding

The progress bar changes color based on utilization:

| Color | Utilization | Meaning |
|-------|-------------|---------|
| Green | 0-70% | Safe usage |
| Yellow | 70-90% | Approaching limit |
| Red | 90-100% | Near or at limit |

## Requirements

- **macOS only**: Quota monitoring requires macOS Keychain for OAuth token access
- **OAuth profiles only**: API Key profiles don't have quota limits via this endpoint
- **Authenticated profile**: The profile must be logged in via `rafctl auth login`

## JSON Output

For scripting and automation, use `--json`:

```bash
rafctl quota work --json
```

```json
{
  "profile": "work",
  "five_hour": {
    "utilization": 42.0,
    "resets_at": "2026-01-06T23:00:00Z"
  },
  "seven_day": {
    "utilization": 68.5,
    "resets_at": "2026-01-10T03:00:00Z"
  }
}
```

### Scripting Examples

```bash
# Check if approaching 5-hour limit
USAGE=$(rafctl quota work --json | jq '.five_hour.utilization')
if (( $(echo "$USAGE > 80" | bc -l) )); then
  echo "Warning: High 5-hour usage ($USAGE%)"
fi

# Get time until reset
rafctl quota --json | jq -r '.five_hour.resets_at'
```

## How It Works

The quota command:

1. Reads the OAuth token from macOS Keychain for the profile
2. Calls the Anthropic API: `GET https://api.anthropic.com/api/oauth/usage`
3. Parses the response and displays utilization

## Troubleshooting

### "Profile is not authenticated"

Run the auth flow first:

```bash
rafctl auth login <profile>
```

### "Failed to fetch quota" (Linux/Windows)

Quota monitoring currently only works on macOS due to Keychain dependency. On other platforms, use the Anthropic Console directly.

### "No OAuth token found"

The profile might be using API Key mode, which doesn't support this endpoint:

```bash
rafctl profile show <profile>
# Check "Auth Mode" field
```

## See Also

- [Authentication](./authentication.md) - How to authenticate profiles
- [Status Command](./status.md) - Profile status overview
