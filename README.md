# Stempel

[![Build](https://img.shields.io/github/workflow/status/KuabeM/stempel/build-master)](https://github.com/KuabeM/stempel/actions?query=workflow%3Abuild-master)
[![Crates.io](https://img.shields.io/crates/v/stempel.svg)](https://crates.io/crates/stempel)

Small utility to store and calculate the time spent at work.

## Usage

```bash
  stempel <SUBCOMMAND> (-s <file>)
```

where the available subcommands are

  - `start`: start a working period, aborts if you already started previously,
  - `stop`: checks if a `start` entry is in the storage `file` and calculates
    the working time, aborts if no `start` entry is found,
  - `break`: use `start` or `stop` as subcommand to handle breaks,
  - `cancel`s the last break, start or does nothing if no break or start in the
    storage,
  - `stats` prints current statistics.

### Options:

#### `--offset`

This option allows to specify a positive or negative offset to the current time.
In other words, giving the option `--offset 10m+` means that the command is
executed with the current time plus 10 minutes, `20s-` stands for current time
minus 20 seconds. The syntax allows `([0-9]*)(h|m|s)(+|-)` where `h|m|s` refers
to hours, minutes and seconds, respectively.

#### `--storage`

Specifiy a path to the storage file where all work entries are written to. The
path defaults to `$HOME/.config/stempel.json` and is created on the first
invocation of the `start` subcommand.

## Planned Features

  - Statistics:
    * [x] pretty printinng
    * [x] provide weekly, monthly,... statistics
    * [ ] allow printing only ranges of stats, e.g. months, years...
  - [ ] Tracking: allow to specify start and stop time as cli arg
  - [x] Tracking: cancel started work
  - [x] Specify an offset from current time when starting or stoping
  - [ ] Storage: don't store as seconds and nanoseconds, use something more verbose?

## License

MIT
