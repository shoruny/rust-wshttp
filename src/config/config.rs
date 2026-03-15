use std::{env, path::PathBuf};

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub path: String,
}
impl Config {
    pub fn build(args: Vec<String>) -> Config {
        if args.len() < 2 {
            panic!("not enough arguments");
        }
        let input = args.get(2).cloned().unwrap_or_default();
        let input_path = PathBuf::from(input);

        let absolute_path = if input_path.is_absolute() {
            input_path
        } else {
            // 只有在需要时才尝试获取当前目录
            env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(input_path)
        };

        // 最终丢给线程的字符串
        let thread_safe_path = absolute_path.to_string_lossy().into_owned();

        let port = args[1]
            .parse::<u16>()
            .ok()
            .filter(|&p| p != 0)
            .unwrap_or(8080);
        Config {
            port,
            path: thread_safe_path,
        }
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
