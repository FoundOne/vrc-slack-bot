use slack::{EventHandler, Event, RtmClient, Message, api};
use std::collections::HashMap;
use config;
use command_parser::Parser;
use std::rc::Rc;

pub struct BotHandler {
    settings: config::Config,
    tipping_channel_id: String,
    user_im: HashMap<String, String>,
    parser: Rc<Parser>,
}

impl BotHandler {
    pub fn new() -> BotHandler {

        let mut settings = config::Config::default();
        //Read Settings.toml
        settings.merge(config::File::with_name("Settings")).unwrap();
        info!("{:#?}", settings.clone().try_into::<HashMap<String, String>>().unwrap());

        let handler = BotHandler { 
                        settings: settings,
                        tipping_channel_id: "".to_owned(),
                        user_im: HashMap::new(),
                        parser: Rc::new(Parser::new()), 
                      };
        handler
    }
    
    pub fn run(&mut self) {
        let token = self.settings.get_str("token").expect("Slack token not found!");
        let r = RtmClient::login_and_run(&token, self);
        match r {
            Ok(_) => {},
            Err(e) => panic!("Error: {}", e),
        };
    }

    fn private_msg(&mut self, sender: &str, msg: &str, cli: &RtmClient){
        let mut tmp: Option<String> = None; // Ugly hack to get the string out
        { //The only way to force the stack known to me
            let im = self.user_im.get(sender).or_else(||{
                info!("The user {} is missing in the database!\n Requesting im info...", &sender);
                let client = api::requests::default_client().unwrap();;
                let requerst = api::im::OpenRequest{ user: sender, return_im: None };
                let response = api::im::open(&client, self.settings.get_str("token").unwrap().as_str(), &requerst).unwrap();
                let im = response.channel.unwrap();
                info!("{:#?}", &im);
                tmp = im.id;
                tmp.as_ref()
            }).unwrap();
            
            let _ = cli.sender().send_message(im, msg);
        }
        if let Some(id) = tmp {
            self.user_im.insert(sender.to_owned(), id);
        }
    }
    
    fn public_msg(&mut self, msg: &str, cli: &RtmClient){
        let _ = cli.sender().send_message(self.tipping_channel_id.as_str(), msg);
    }
    
}

impl EventHandler for BotHandler {
    
    fn on_event(&mut self, cli: &RtmClient, event: Event){
       if let Event::Message(msg) = event {
        if let Message::Standard(msg) = *msg {
            debug!("On message: {:#?} channel: {:#?} user: {:#?}",
                    msg.text.as_ref().unwrap(),
                    msg.channel.as_ref().unwrap(),
                    msg.user.clone().unwrap()
                );
            if (msg.channel.as_ref().unwrap() == &self.tipping_channel_id) && cli.start_response()
                .slf
                .as_ref()
                .and_then(|slf|{
                    let mention = format!("<@{}>", slf.id.as_ref().unwrap());
                    Some(msg.text.as_ref().unwrap().starts_with(&mention))
                })
                .unwrap()
                || self.user_im.get(msg.user.as_ref().unwrap()) == Some(msg.channel.as_ref().unwrap()){
                    let sender = msg.user.as_ref().unwrap();
                    let line = msg.text.unwrap();
                    let parser = self.parser.clone();
                    let message = parser.parse(sender.as_str(), line.as_str());
                    
                    info!("{:?}", message);
                    if let Some(message) = message {
                        if message.private == true {
                            self.private_msg(sender, &message.content, cli);
                        }
                        else {
                            self.public_msg(&message.content, cli);
                        }
                    }
                }
        }
       }
       else if let Event::ChannelJoined{channel} = event {
            info!("{:?}", channel);
       }
    }
    
    fn on_close(&mut self, _cli: &RtmClient){
        debug!("Client closed!");
    }
    
    fn on_connect(&mut self, cli: &RtmClient){
        debug!("Client connected successfully!");
        // Setting the tipping channel
        let tipping_channel = match self.settings.get_str("tipping_channel") {
            Ok(chan) => chan,
            Err(_) => "general".to_owned()
        };

        let tipping_channel_id = cli.start_response()
            .channels
            .as_ref()
            .and_then(|channels| {
                channels
                    .iter()
                    .find(|chan| match chan.name {
                            None => false,
                            Some(ref name) => {
                                    debug!("Found channel: {}", name);
                                    name == &tipping_channel
                                },
                    })
            })
            .and_then(|chan| chan.id.as_ref())
            .expect("Tipping channel not found!");
        self.tipping_channel_id = tipping_channel_id.clone();
        info!("Tiping channel id: {}", tipping_channel_id);
        info!("Tipping channel name: {}", tipping_channel);
        
        // Populating users' IMs
        cli.start_response()
        .ims
        .as_ref()
        .and_then(|ims| {
            ims
                .iter()
                .for_each(|im| {
                    let user = im.user.as_ref().unwrap().clone();
                    let id = im.id.as_ref().unwrap().clone();
                    debug!("user {} : im {}", &user, &id);
                    self.user_im.insert(user, id);
                });
                Some(())
            }
        );
    }
}
