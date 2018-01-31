#VRC Slack Bot
A new fast and secure tipping bot for the needs of the Vericoin/Verium Slack.

####Build Howto
The bot is mainly tested in linux, but it will probably work in windows or mac. 
First you need the rust toolchain:
```bash
curl https://sh.rustup.rs -sSf | sh
```
To build the bot you have to write:
```bash
cargo build --release
```
or directly run it:
```bash
cargo run
```
The executable file can be found in target/release/vrc-slack-bot
Next you have to edit Settings.toml There is a template in the main directory called Settings.toml.exmple which can help you with that. The slack token can be obtained from [here](https://my.slack.com/services/new/bot).
