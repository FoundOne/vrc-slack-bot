use config;
use std::collections::HashMap;
use regex::{Regex, Captures, CaptureNames};
use mustache::{MapBuilder, Template, compile_str};

#[derive(Debug)]
pub struct Message {
    pub content: String,
    pub private: bool,
}

fn regex_compile() -> &'static HashMap<String, Regex> {
    lazy_static!{
        static ref RULES: HashMap<String, Regex> = {
            let mut re_config = config::Config::default();
            let mut re = HashMap::new();
            re_config.merge(config::File::with_name("Regex")).unwrap();
            re_config
                .try_into::<HashMap<String, String>>()
                .and_then(|re_config|{
                    re_config
                    .iter()
                    .for_each(|(key, value)|{
                        re.insert(key.clone(), Regex::new(value).unwrap());
                    });
                    Ok(())
                })
                .expect("Regex parse failure!");
            re
        };
    }
    &RULES
}

fn temlate_compile() -> &'static HashMap<String, Template> {
    lazy_static!{
        static ref TEMPLATES: HashMap<String, Template> = {
            let mut tmpl_config = config::Config::default();
            let mut tmpl = HashMap::new();
            tmpl_config.merge(config::File::with_name("Messages")).unwrap();
            tmpl_config.try_into::<HashMap<String, String>>()
                .and_then(|tmpl_config| {
                    tmpl_config
                    .iter()
                    .for_each(|(key, value)|{
                       tmpl.insert(key.clone(), compile_str(value.as_str()).unwrap());
                    });
                    Ok(())
                })
                .expect("Template parse failure!");
            tmpl
        };
    }
    &TEMPLATES
}

pub struct Parser;

impl Parser {

    pub fn new() -> Parser {
        info!("Compiling the regexes...");
        regex_compile();
        info!("Compiling the templates...");
        temlate_compile();

        let parser = Parser;
        parser
    }

    pub fn parse(&self, user: &str, line: &str) -> Option<Message> {
        let mut msg = None;
        let rx = regex_compile();
        info!("REGEX: {:?}", &rx);
        rx
        .iter()
        .for_each(|(rx_name, regex)|{
            if let Some(caps) = regex.captures(&line) {
                info!("Matched");
                msg = self.render(user, caps, regex.capture_names(), rx_name);
                return;
            }
        });
        msg
    }
    
    fn render(&self, user: &str, caps: Captures, cap_names: CaptureNames, tmpl_name: &str) -> Option<Message> {
        let tmpl_db = temlate_compile();
        if let Some(tmpl) = tmpl_db.get(tmpl_name) {
            // Only tipping is posted on the tipping channel.
            let private = if tmpl_name == "tip" {
                false
            } 
            else {
                true
            };

            let mut text = MapBuilder::new();
            text = text.insert_str("sender", user);
            
            match tmpl_name {
                "balance" => {
                    text = text.insert_str("amount", 0); // XXX Here should be the wallet balance interface
                },
                
                "tip" => {
                    let amount: f64 = caps["amount"].parse().unwrap();
                    info!("{} tipped {} {}VRC", &user, &caps["user"], amount);
                },
                _ => {}
            }

            for name in cap_names {
                if let Some(name) = name {
                    text = text.insert_str(name, caps.name(name).unwrap().as_str());
                }
            }
            let txt = tmpl.render_data_to_string(&text.build()).unwrap();
            Some(Message{ content: txt, private: private })
        }
        else {
            None
        }
    }
}
