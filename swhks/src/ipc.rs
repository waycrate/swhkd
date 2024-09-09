use std::{
    hash::{DefaultHasher, Hash, Hasher},
    io::{Read, Write},
    os::unix::net::UnixListener,
    process::Command,
};

/// Get the environment variables
/// These would be requested from the default shell to make sure that the environment is up-to-date
fn get_env() -> Result<String, Box<dyn std::error::Error>> {
    let shell = std::env::var("SHELL")?;
    let cmd = Command::new(shell).arg("-c").arg("env").output()?;
    let stdout = String::from_utf8(cmd.stdout)?;
    Ok(stdout)
}

/// Calculates a simple hash of the string
/// Uses the DefaultHasher from the std::hash module which is not a cryptographically secure hash,
/// however, it is good enough for our use case.
pub fn calculate_hash(t: String) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

pub fn server_loop(sock_file_path: &str) -> std::io::Result<()> {
    let mut prev_hash = calculate_hash(String::new());

    let listener = UnixListener::bind(sock_file_path)?;
    // Init a buffer to read the incoming message
    let mut buff = [0; 1];
    log::debug!("Listening for incoming connections...");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                stream.read_exact(&mut buff)?;
                // If the buffer is [1] then it is a VERIFY message
                // the hash of the environment variables is sent back to the client
                // then the stream is flushed and the loop continues
                if buff == [1] {
                    log::debug!("Received VERIFY message from swhkd");
                    let _ = stream.write_all(prev_hash.to_string().as_bytes());
                    log::debug!("Sent hash to swhkd");
                    stream.flush()?;
                    continue;
                }
                // If the buffer is [2] then it is a GET message
                // the environment variables are sent back to the client
                // then the stream is flushed and the loop continues
                if buff == [2] {
                    log::debug!("Received GET message from swhkd");
                    let env = get_env().unwrap();
                    if prev_hash == calculate_hash(env.clone()) {
                        log::debug!("No changes in environment variables");
                    } else {
                        log::debug!("Changes in environment variables");
                    }
                    prev_hash = calculate_hash(env.clone());
                    let _ = stream.write_all(env.as_bytes());
                    stream.flush()?;
                    continue;
                }
            }
            Err(e) => {
                log::error!("Error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
