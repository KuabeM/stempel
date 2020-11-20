# Stempel

[![Build](https://img.shields.io/github/workflow/status/KuabeM/stempel/build-master)](https://github.com/KuabeM/stempel/actions?query=workflow%3Abuild-master)
[![Crates.io](https://img.shields.io/crates/v/stempel.svg)](https://crates.io/crates/stempel)

Small utility to store and calculate the time spent at work.

## Usage

```bash
  stempel <SUBCOMMAND> (-s <file>)
```

where the available subcommands are

  - `start` writes current time to the file specified in `-s`, use sub-subcommand to start a break
  - `stop` checks if a `start` entry is in `file` and calculates the working time, aborts if no `start` entry is found, use sub-subcommand to stop a break
  - `stats` prints current statistics.

The storage file defaults to `$HOME/.config/stempel.json`

## Planned Features

  - Statistics:
    * [x] pretty printinng
    * [x] provide weekly, monthly,... statistics
    * [ ] allow printing only ranges of stats, e.g. months, years...
  - [ ] Tracking: allow to specify start and stop time as cli arg
  - [x] Tracking: cancel started work
  - [ ] Storage: don't store as seconds and nanoseconds, use something more verbose?

## License

MIT
