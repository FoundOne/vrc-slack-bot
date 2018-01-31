extern crate config; // The config module
extern crate slack; // The slack module
extern crate regex;
extern crate mustache; // For the templates

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

pub mod slack_bot; // slack api module
pub mod command_parser; // parser for the commands 
// pub mod wallet_api; // vericoin api module
