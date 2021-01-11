//! Provides handlers for subcommands.
//!
//! Takes care of most of actual application logic, throws erros and writes to the disk. It is split
//! into `control` module for starting, stoping and handling periods and a module `stats` for
//! printing statistics about past and current work periods.

pub mod control;
pub mod stats;
