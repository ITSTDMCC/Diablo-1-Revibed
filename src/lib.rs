#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
//! Diablo 1 port. The source of truth is DevilutionX 1.5.3 (see `port/NOTES.md`).
//!
//! Every ported function carries a tag line
//! `// @port <manifest key> sha=<first 12 hex of the source SHA-256>`
//! that `tests/parity.rs` checks against `port/manifest.csv` and `port/db_functions.csv`.

pub mod appfat;
pub mod bzip2;
pub mod control;
pub mod controls;
pub mod dvlnet;
pub mod freemove;
pub mod ctx;
pub mod diablo;
pub mod diablo_game;
pub mod diablo_ui;
pub mod encrypt;
pub mod enums;
pub mod engine;
pub mod cursor;
pub mod hwcursor;
pub mod inflate;
pub mod ini;
pub mod init;
pub mod interfac;
pub mod inv;
pub mod levels;
pub mod mpq;
pub mod options;
pub mod pack;
pub mod pkware;
pub mod implode;
pub mod sha;
pub mod codec;
pub mod mpq_writer;
pub mod platform;
pub mod port;
pub mod qol;
pub mod spells;
pub mod storm;
pub mod tables;
#[doc(hidden)]
pub mod test_data;
pub mod utils;
pub mod effects;
pub mod effects_data;
pub mod items;
pub mod gamemenu;
pub mod gmenu;
pub mod player;
pub mod automap;
pub mod help;
pub mod capture;
pub mod plrmsg;
pub mod error;
pub mod movie;
pub mod menu;
pub mod monster;
pub mod missiles;
pub mod objects;
pub mod towners;
pub mod doom;
pub mod pfile;
pub mod stores;
pub mod minitext;
pub mod multi;
pub mod dead;
pub mod loadsave;
pub mod lighting;
pub mod msg;
pub mod nthread;
pub mod portal;
pub mod quests;
pub mod sync;
pub mod tmsg;
pub mod track;
pub mod panels;
