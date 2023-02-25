use serenity::builder::CreateApplicationCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.name("beep").description("Beep boop I'm a bot")
}

pub fn run() -> String {
    "🤖 boop! 🤖".to_string()
}
