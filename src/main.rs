use std::env;

use wshttp::{config::config::Config, http::serv};

pub fn main() {
    let config = Config::build(env::args().collect());
    // println!("{config:?}");
    serv::run(format!("0.0.0.0:{}", config.port), config.path);
}
