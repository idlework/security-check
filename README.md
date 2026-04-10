# security-cli

A macOS security audit tool that checks your system's security configuration and scores it. Inspired by [SilentKnight](https://eclecticlight.co/lockrattler-systhist) and Apple's [built-in security tools](https://www.huntress.com/blog/built-in-macos-security-tools).

Run a single command to see what's properly configured and what could be improved -- no installs, no accounts, no data leaves your machine.

```
$ security-cli

  macOS Security Audit
  Mac14,9 (Apple M2 Pro) -- macOS 15.7.4

  System Protection

   PASS   System Integrity Protection         SIP is enabled
   PASS   Gatekeeper                          Assessments enabled

  Encryption

   PASS   FileVault                           Disk encryption is on

  Firewall

   FAIL   Firewall                            Firewall is disabled
          Hint: Enable in System Settings > Network > Firewall
   WARN   Stealth Mode                        Stealth mode is disabled
   WARN   Firewall Logging                    Logging is disabled

  ...

  Summary

  21 passed  9 warnings  1 failures  1 skipped

  Score: 79% (C) -- 126/160 points
```

## Install

```sh
git clone <repo-url> && cd security-cli
cargo build --release
cp target/release/security-cli /usr/local/bin/
```

Requires [Rust](https://rustup.rs/) to build. Produces a single 1.2MB binary with no runtime dependencies.

## Usage

See [docs/usage.md](docs/usage.md) for detailed documentation.

```sh
security-cli              # run all checks
security-cli --verbose    # show extra details and fix hints
security-cli --json       # machine-readable JSON output
security-cli --category firewall  # run one category only
sudo security-cli         # include checks that require root
```

## What it checks

| Category | Checks | What it covers |
|---|---|---|
| System Protection | 2 | SIP, Gatekeeper |
| Encryption | 1 | FileVault |
| Firewall | 3 | Firewall state, stealth mode, logging |
| Malware Protection | 5 | XProtect version, definitions, remediator, services, recency |
| Software Updates | 5 | Auto-check, auto-download, auto-install, critical updates, pending |
| Network | 5 | SSH, File Sharing, Screen Sharing, Remote Management, DNS |
| Hardware | 2 | System info, Activation Lock |
| User Security | 5 | Screen lock, screensaver password, guest account, auto-login, MDM |
| Privacy | 4 | AirDrop, Launch Agents, Launch Daemons, analytics sharing |

## Scoring

Each check has a weight based on its security impact. The final score is a percentage:

| Grade | Score |
|---|---|
| A+ | 95-100% |
| A | 90-94% |
| B+ | 85-89% |
| B | 80-84% |
| C | 70-79% |
| D | 60-69% |
| F | <60% |

Checks that are skipped (e.g. requiring sudo) are excluded from the score rather than counting against you.

## License

MIT
