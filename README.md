# Stempel

[![Build](https://img.shields.io/github/actions/workflow/status/KuabeM/stempel/build_master.yml)](https://github.com/KuabeM/stempel/actions?query=workflow%3Abuild-master)
[![Crates.io](https://img.shields.io/crates/v/stempel.svg)](https://crates.io/crates/stempel)
[![AUR](https://img.shields.io/aur/version/stempel-bin)](https://aur.archlinux.org/packages/stempel-bin)

Small utility to store and calculate the time spent at work.

> :warning: v0.10.0 introduces a new storage file format. Run `stempel migrate`
> to update your json database to the new format.

## Usage

Example for managing one day:

```bash
# First of, configure some settings
stempel configure
# start working now
stempel start
# start a break five minutes ago
stempel break start --offset 5m-
# optional: break can be canceled:
stempel cancel
# finish break in one hour (only if not canceled above)
stempel break stop --offset 1h+
# Finish the day
stempel stop
# Alternatively, you can specify the time point to stop as a time in 24h format
stempel stop --time 17:38
```

For a detailed reference, run `stempel help` or `stempel SUBCOMMAND --help`.
Available subcommands are:

  - `cancel`s the last break, start or does nothing if no break or start in the
    storage,
  - `break`: use `start` or `stop` as subcommand to handle breaks,
  - `migrate`: migrate storage file from old (before 0.10.0) to new format
    (since v0.10.0)
  - `start`: start a working period, aborts if you already started previously,
  - `stats` prints current statistics.
  - `stop`: checks if a `start` entry is in the storage `file` and calculates
    the working time, aborts if no `start` entry is found,
  - `configure`: set some defaults for stempel and save them alongside the
    database file. Currently available:
    * number of months printed by the statistic command
    * number of hours to work per day

# Completions

Run `stempel completions --shell <shell>` to get a shell completion file, e.g. `stempel completions
--shell zsh` for zsh. Source this file according to your specific shell.

```zsh
source <(stempel completions --shell zsh)
```

### Options:

#### `--offset`

This option allows to specify a positive or negative offset to the current time.
In other words, giving the option `--offset 10m+` means that the command is
executed with the current time plus 10 minutes, `20s-` stands for current time
minus 20 seconds. The syntax allows `[Xh][Xm][Xs](+-)` where `X` can be any
number and `h|m|s` refer to hours, minutes and seconds, respectively. Overridden
by `--time`below.

Some examples:

  - `2h30m4s+`: 2 hours, 30 minutes, 4 seconds from now
  - `1h90s-`: 1 hour 90 seconds before now
  - `20m30s+`: 20 minutes, 30 seconds from now
  - `60s-`: one minute before now

#### `--time`

This options allows to specify a time for the respective action. Use it if you
want to start or stop at a certain time. The format is `HH:MM` in 24h where the
date is set to the current day, e.g. `13:47` means today at `13h 47`.

#### `--storage`

Specify a path to the storage file where all work entries are written to. The
path defaults to `$HOME/.config/stempel.json` and is created on the first
invocation of the `start` subcommand.

## License

MIT
