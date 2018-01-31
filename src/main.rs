//Use loggers
extern crate log;
extern crate env_logger;
extern crate vrc_slack_bot;
use vrc_slack_bot::slack_bot::BotHandler;

fn main() {
    // Turn loggers on.
    env_logger::init().unwrap();
    
    let mut bot = BotHandler::new();
    bot.run();
}
