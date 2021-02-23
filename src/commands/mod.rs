//! Provides handlers for subcommands.
//!
//! Takes care of most of actual application logic, throws errors and writes to
//! the disk. It is split into `control` module for starting, stopping and
//! handling periods and a module `stats` for printing statistics about past and
//! current work periods.

pub mod config;
pub mod control;
pub mod stats;
