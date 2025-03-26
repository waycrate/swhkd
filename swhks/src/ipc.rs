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
pub fn calculate_hash(t: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

pub fn server_loop(sock_file_path: &str) -> std::io::Result<()> {
    let mut prev_hash = calculate_hash("");

    let listener = UnixListener::bind(sock_file_path)?;
    // Init a buffer to read the incoming message
    let mut buff = [0; 1];
    log::debug!("Listening for incoming connections...");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(e) = stream.read_exact(&mut buff) {
                    log::error!("Failed to read from stream: {}", e);
                    continue;
                }

                match buff[0] {
                    1 => {
                        // If the buffer is [1] then it is a VERIFY message
                        // the hash of the environment variables is sent back to the client
                        // then the stream is flushed and the loop continues
                        log::debug!("Received VERIFY request from swhkd");
                        if let Err(e) = stream.write_all(prev_hash.to_string().as_bytes()) {
                            log::error!("Failed to write hash to stream: {}", e);
                        } else {
                            log::debug!("Sent hash to swhkd");
                        }
                    }
                    2 => {
                        // If the buffer is [2] then it is a GET message
                        // the environment variables are sent back to the client
                        // then the stream is flushed and the loop continues
                        log::debug!("Received GET request from swhkd");

                        match get_env() {
                            Ok(env) => {
                                let new_hash = calculate_hash(&env);
                                if prev_hash == new_hash {
                                    log::debug!("No changes in environment variables");
                                } else {
                                    log::debug!("Environment variables updated");
                                    prev_hash = new_hash;
                                }

                                if let Err(e) = stream.write_all(env.as_bytes()) {
                                    log::error!("Failed to send environment variables: {}", e);
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to retrieve environment variables: {}", e);
                                let _ = stream.write_all(b"ERROR: Unable to fetch environment");
                            }
                        }
                    }
                    _ => {
                        log::warn!("Received unknown request: {}", buff[0]);
                    }
                }

                if let Err(e) = stream.flush() {
                    log::error!("Failed to flush stream: {}", e);
                }
            }
            Err(e) => {
                log::error!("Error handling connection: {}", e);
                break;
            }
        }
    }

    Ok(())
}
