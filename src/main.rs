use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use serde_derive::Deserialize;
use ssh2::Session;
use toml;

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Config {
    host: Host,
    files: Option<Vec<Files>>,
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Host {
    remote_ip: String,
    username: String,
    pubkey: String,
    privatekey: String,
    passphrase: String,
    interval: u64,
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Files {
    local_path: String,
    remote_path: String,
}


fn create_session(config: &Config) -> Session {
    let tcp = TcpStream::connect(config.host.remote_ip.as_str()).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_pubkey_file(
        config.host.username.as_str(),
        Option::Some(Path::new(config.host.pubkey.as_str())),
        Path::new(config.host.privatekey.as_str()),
        Option::Some(config.host.passphrase.as_str()),
    ).unwrap();
    sess
}

fn upload(config: &Config) {
    let sess = create_session(config);
    let sftp = sess.sftp().unwrap();
    for files in config.files.as_ref() {
        for f in files {
            let mut local_file = File::open(&f.local_path).expect("Unable to open file");
            let mut contents = String::new();
            local_file.read_to_string(&mut contents).unwrap();
            let mut remote_file = sftp.create(
                Path::new(&f.remote_path.as_str())
            ).expect("Unable to create remote file");
            remote_file.write(contents.as_bytes()).expect("Unable to write remote file:");
        }
    }
}

fn read_config() -> Config {
    let mut config_file = File::open("config.toml").expect("Unable to open config_file");
    let mut contents = String::new();
    config_file.read_to_string(&mut contents).expect("Read config error");
    let config: Config = toml::from_str(contents.as_str()).unwrap();
    println!("Load {:?}", config);
    config
}

fn main() {
    let config = read_config();
    loop {
        upload(&config);
        sleep(Duration::new(config.host.interval, 0));
    }
}