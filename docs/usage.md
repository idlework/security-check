# Usage Guide

## Quick Start

```sh
# Build and run
cargo build --release
./target/release/security-check

# Or install globally
cp target/release/security-check /usr/local/bin/
security-check
```

## Command Line Options

```
security-check [OPTIONS]

Options:
      --json                 Output results as JSON
  -v, --verbose              Show detailed information for each check
      --category <CATEGORY>  Run only checks in a specific category
      --list                 List all available checks without running them
  -h, --help                 Print help
  -V, --version              Print version
```

## Running Without sudo

By default, `security-check` runs without elevated privileges. Most checks work fine this way. Checks that require root access are marked as `SKIP` and excluded from your score.

```
  SKIP   Remote Login (SSH)                  Requires sudo to check
```

The summary will suggest running with sudo if any checks were skipped:

```
  Run with sudo for complete results: sudo security-check
```

## Running With sudo

To include all checks (like SSH status detection via `lsof`):

```sh
sudo security-check
```

The tool never prompts for a password itself -- it simply detects whether it's running as root and adjusts accordingly.

## Output Modes

### Default (colored terminal)

The standard output groups checks by category and uses colors:

- **PASS** (green) -- correctly configured
- **WARN** (yellow) -- could be improved
- **FAIL** (red) -- security issue
- **SKIP** (gray) -- could not run (usually needs sudo)

Warnings and failures include a `Hint:` line explaining how to fix the issue.

### Verbose (`-v` / `--verbose`)

Adds extra detail lines below checks that have additional context:

```sh
security-check --verbose
```

This shows review notes for informational checks like Launch Agents and Launch Daemons.

### JSON (`--json`)

Outputs a structured JSON report suitable for scripting, monitoring, or piping to other tools:

```sh
security-check --json
```

The JSON structure:

```json
{
  "system_info": "Mac14,9 (Apple M2 Pro) -- macOS 15.7.4",
  "checks": [
    {
      "category": "system_protection",
      "name": "System Integrity Protection",
      "status": "pass",
      "message": "SIP is enabled"
    }
  ],
  "summary": {
    "passed": 21,
    "warnings": 9,
    "failures": 1,
    "skipped": 1,
    "score_pct": 79,
    "grade": "C"
  }
}
```

Useful for:

```sh
# Pretty-print
security-check --json | python3 -m json.tool

# Extract score
security-check --json | python3 -c "import sys,json; print(json.load(sys.stdin)['summary']['grade'])"

# List failures only
security-check --json | jq '.checks[] | select(.status == "fail")'
```

### No Color

Colors are automatically disabled when output is piped. To explicitly disable:

```sh
NO_COLOR=1 security-check
```

## Filtering by Category

Run checks for a single category:

```sh
security-check --category firewall
security-check --category malware
security-check --category network
security-check --category privacy
```

Category names are matched loosely -- `firewall`, `Firewall`, and `fire` all work.

Available categories:

| Filter value | Category |
|---|---|
| `system` | System Protection |
| `encryption` | Encryption |
| `firewall` | Firewall |
| `malware` | Malware Protection |
| `updates` | Software Updates |
| `network` | Network |
| `hardware` | Hardware |
| `user` | User Security |
| `privacy` | Privacy |

## Understanding the Checks

### System Protection

**System Integrity Protection (SIP)** -- Prevents modifications to protected system files and directories, even by root. Should always be enabled. Disabling requires booting into Recovery Mode.

**Gatekeeper** -- Validates that applications are signed by identified developers or from the App Store before allowing them to run. Protects against unsigned malware.

### Encryption

**FileVault** -- Full-disk encryption using XTS-AES-128 with a 256-bit key. Protects data at rest if your Mac is lost or stolen. Enabled by default on Apple Silicon Macs.

### Firewall

**Application Firewall** -- Controls incoming connections per-application. When disabled, all incoming connections are allowed.

**Stealth Mode** -- When enabled, your Mac does not respond to ICMP ping requests or connection attempts from closed TCP/UDP ports, making it harder to discover on a network.

**Firewall Logging** -- Records firewall activity for security monitoring and incident investigation.

### Malware Protection

**XProtect Version** -- Apple's malware signature database version number. Updated silently in the background.

**XProtect Definitions** -- The actual malware signature files (plist + YARA rules) used to identify known threats.

**XProtect Remediator** -- Specialized modules (26+) that can detect and remove specific malware families. Each module targets a different threat.

**XProtect Services** -- Background services that perform scanning. Should always be running.

**XProtect Recency** -- How recently the definitions were updated. Definitions older than 30 days trigger a warning; older than 90 days trigger a failure.

### Software Updates

**Auto-Check / Auto-Download / Auto-Install** -- Whether macOS automatically checks for, downloads, and installs updates. All should be enabled for timely security patches.

**Critical Update Install** -- Whether Rapid Security Responses (critical patches) are installed automatically. These are Apple's fast-track security fixes.

**Pending Updates** -- Lists any available updates not yet installed.

### Network

**Remote Login (SSH)** -- Whether the SSH server is running, allowing remote shell access. Should be disabled unless actively needed.

**File Sharing** -- Whether SMB file sharing is active. Exposes files on the network.

**Screen Sharing** -- Whether VNC/screen sharing is active. Allows remote screen control.

**Remote Management** -- Whether Apple Remote Desktop agent is active.

**DNS Configuration** -- Whether custom DNS servers are configured. Default ISP DNS may not offer privacy protections or DNSSEC validation.

### Hardware

**System Info** -- Displays model, chip, memory, and partial serial number (informational, does not affect score).

**Activation Lock** -- Whether Find My Mac / Activation Lock is enabled. Prevents someone from erasing and reusing a stolen Mac.

### User Security

**Screen Lock** -- Whether the screen locks immediately when the display sleeps or the screensaver activates. A delay allows physical access to an unlocked machine.

**Screensaver Password** -- Whether a password is required to dismiss the screensaver.

**Guest Account** -- Whether the guest account is enabled. Guest accounts provide unauthenticated access to the machine.

**Auto-Login** -- Whether a user is automatically logged in at boot. Bypasses the login screen entirely.

**MDM Enrollment** -- Whether the device is managed by Mobile Device Management. Informational only -- does not affect score.

### Privacy

**AirDrop** -- Whether AirDrop is set to receive from everyone, contacts only, or disabled. "Everyone" mode allows strangers to send files.

**User Launch Agents** -- Lists user-installed background processes in `~/Library/LaunchAgents`. Review for unexpected entries.

**Third-Party Launch Daemons** -- Lists non-Apple system daemons in `/Library/LaunchDaemons`. These run as root at boot.

**Analytics Sharing** -- Whether diagnostic data is shared with Apple. Disabling improves privacy.

## Scoring System

Each check has a weight reflecting its security impact:

| Category | Weight per check |
|---|---|
| System Protection | 10 |
| Encryption | 10 |
| Firewall (enabled) | 10 |
| Firewall (stealth/logging) | 3 |
| Malware Protection | 8 |
| Software Updates | 5 |
| Network | 5 (DNS: 3) |
| User Security | 5 |
| Privacy | 3 |
| Hardware (info) | 0 |

Score calculation:

- **PASS** = full weight earned
- **WARN** = half weight earned
- **FAIL** = 0 points
- **SKIP** = excluded from total (does not penalize)

Final score: `(earned / possible) * 100%`

Informational checks (system info, MDM enrollment, Launch Agent/Daemon listings) have weight 0 and do not affect the score.
