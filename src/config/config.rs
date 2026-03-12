#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub path: String,
}
impl Config {
    pub fn build(mut args: Vec<String>) -> Config {
        if args.len() < 3 {
            panic!("not enough arguments");
        }
        let path = std::mem::take(&mut args[2]);
        let port = args[1]
            .parse::<u16>()
            .ok()
            .filter(|&p| p != 0)
            .unwrap_or(8080);
        // let port: String = match args.get(1) {
        //     Some(port) => port.clone(),
        //     None => String::from("8080"),
        // };
        // let parsed = port.parse::<u16>();
        // config.port = match parsed {
        //     Ok(port) => port,
        //     _ => 8080u16,
        // };
        // config.port = if config.port == 0 { 8080 } else { config.port };
        // config.path = match args.get(2) {
        //     Some(path) => path.clone(),
        //     None => String::from(""),
        // };
        Config { port, path }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config1() {
        let args = vec!["".to_string(), "8080".to_string(), "/".to_string()];
        let config = Config::build(args);
        assert_eq!(config.port, 8080);
        assert_eq!(config.path, "/");
    }
    #[test]
    #[should_panic]
    fn painc() {
        let args = vec!["".to_string()];
        Config::build(args);
    }
    #[test]
    #[should_panic]
    fn painc2() {
        let args = vec!["".to_string(), "".to_string()];
        Config::build(args);
    }
    #[test]
    fn default() {
        let args = vec!["".to_string(), "".to_string(), "".to_string()];
        let config = Config::build(args);
        assert_eq!(config.port, 8080);
        assert_eq!(config.path, "");
    }
    #[test]
    fn port() {
        let args = vec!["".to_string(), "1111".to_string(), "".to_string()];
        let config = Config::build(args);
        assert_eq!(config.port, 1111);
        assert_eq!(config.path, "");
    }
    #[test]
    fn port0() {
        let args = vec!["".to_string(), "0".to_string(), "".to_string()];
        let config = Config::build(args);
        assert_eq!(config.port, 8080);
        assert_eq!(config.path, "");
    }
}
