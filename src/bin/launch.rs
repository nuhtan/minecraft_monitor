//! Crate wide documentation?
extern crate minecraft_monitor as mon;
use mon::functions::configuration::{determine_config, Verbosity};
use mon::functions::minecraft_related::*;
use mon::functions::shared_data::*;
use mon::functions::web_server::handle_connections;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::{env, net::Ipv4Addr};

fn main() {
    // Create a writer that will write content to a file, only interactions that happen from this software will be logged, Minecraft logs itself
    let (
        address,
        port,
        web_index,
        root_location,
        jar_name,
        gen_args,
        min_ram,
        max_ram,
        web_log,   // If interactions with the webserver should be saved to a log
        verbosity, // Currently this is not set up
    ) = determine_config(env::args().collect()).unwrap();

    env::set_current_dir(Path::new(&root_location)).unwrap();
    // Logger::new(); // TODO Logging will be hard

    let shared_data = ServerSharedData::new();
    // call launch with shared data
    loop {
        launch(
            shared_data.clone(),
            address.clone(),
            port.clone(),
            web_index.clone(),
            jar_name.clone(),
            gen_args.clone(),
            min_ram.clone(),
            max_ram.clone(),
            web_log.clone(),
            verbosity.clone(),
        );
        let mut state = shared_data.gen_state.lock().unwrap();
        if *state == GeneralState::Restart {
            *state = GeneralState::Running;
            let mut mc_state = shared_data.mcserver_state.lock().unwrap();
            *mc_state = MinecraftServerState::Starting;
            println!("Restarting Server");
        } else if *state == GeneralState::ShutDown {
            println!("Shutting Down Server");
            break;
        } // If it does not shutdown then it restarts, it shouldn't reach this point without being either shutdown or restart
    }
}

fn launch(
    shared_data: ServerSharedData,
    address: Ipv4Addr,
    port: u16,
    web_index: String,
    jar_name: String,
    gen_args: Option<String>,
    min_ram: String,
    max_ram: String,
    web_log: bool,
    verbosity: Verbosity,
) {
    let (web_sender, web_receiver) = mpsc::channel::<String>();
    let shared_data_web = shared_data.clone();
    let web_sender_clone = web_sender.clone();
    let web_handle = thread::spawn(move || {
        handle_connections(
            shared_data_web,
            web_sender_clone,
            address,
            port,
            web_index,
            verbosity,
        )
        .unwrap()
    });

    let mut child;
    if gen_args == None {
        child = Command::new("java")
            .args(&[
                format!("-Xms{}", min_ram).as_str(),
                format!("-Xmx{}", max_ram).as_str(),
                "-XX:+UseG1GC",
                "-jar",
                format!("{}", jar_name).as_str(),
                "nogui",
            ])
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .expect("Error starting server, refer to console for more details.");
    } else {
        // let args = Vec::new();
        let args = gen_args.unwrap();
        let split_args = args.split(" ").collect::<Vec<&str>>();
        // let test = args.clone();
        child = Command::new("java")
            .args(split_args)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .expect("Error starting server, refer to console for more details.");
    }

    // Output section
    let mut mcserver_out = BufReader::new(
        child
            .stdout
            .take()
            .expect("[Error] Failed to open server output"),
    );

    let shared_data_output = shared_data.clone();
    let output_verbosity = verbosity.clone(); // This might not be needed because COPY has been derived now
    let output_sender = web_sender.clone();
    let output_handle = thread::spawn(move || {
        let mut line_num: u32 = 0;
        let output_sender_thread = output_sender.clone();
        loop {
            // let output_sender3 = output_sender2.clone();
            {
                // If the server is trying to restart exit the output thread to the minecraft server
                let mc_state = shared_data_output.mcserver_state.lock().unwrap();
                // println!("Checking state in output");
                if *mc_state == MinecraftServerState::Off {
                    let mut state = shared_data_output.gen_state.lock().unwrap();
                    if *state == GeneralState::Restart || *state == GeneralState::ShutDown {
                        break;
                    } else if *state == GeneralState::Running {
                        *state = GeneralState::Restart;
                        break;
                    }
                }
                let state = shared_data_output.gen_state.lock().unwrap();
                if *mc_state == MinecraftServerState::Eula && *state == GeneralState::Restart {
                    break;
                }
            }
            let chat = shared_data_output.server_output.clone();
            let mut buf = Vec::new();
            mcserver_out.read_until(b'\n', &mut buf).unwrap();
            let line = String::from_utf8(buf).unwrap();
            if line != "".to_string() {
                let content = &line.clone()[17..];
                if line != "" {
                    let mut term = chat.lock().unwrap();
                    if output_verbosity == Verbosity::Mine || verbosity == Verbosity::MineWeb {
                        print!("\x1b[0;36m[Console]:\x1b[0m {}", line);
                    }
                    term.push_front((line_num, line));
                    while term.len() > 1000 {
                        term.pop_back();
                    }
                }
                // Check if a player has joined
                server_output_scanning(content, shared_data_output.clone(), &output_sender_thread);
                line_num += 1;
            }
        }
    });

    // Input section
    let shared_data_input = shared_data.clone();
    let input_verbosity = verbosity.clone();
    let input_handle = thread::spawn(move || {
        loop {
            // let verbosity = verbosity.clone();
            {
                // If the server is trying to restart exit the input thread to the minecraft server
                let mc_state = shared_data_input.mcserver_state.lock().unwrap();
                if *mc_state == MinecraftServerState::Off {
                    let mut state = shared_data_input.gen_state.lock().unwrap();
                    println!("Checking state in input");
                    if *state == GeneralState::Restart || *state == GeneralState::ShutDown {
                        break;
                    } else if *state == GeneralState::Running {
                        *state = GeneralState::Restart;
                        break;
                    }
                }
                let state = shared_data_input.gen_state.lock().unwrap();
                if *mc_state == MinecraftServerState::Eula && *state == GeneralState::Restart {
                    break;
                }
            }
            // Sleeping per the tick rate, this might be slightly extreme for the purposes of this application
            match web_receiver.recv_timeout(Duration::from_millis(50)) {
                Ok(mut cmd) => {
                    cmd = cmd + "\n";
                    if input_verbosity == Verbosity::Mine || input_verbosity == Verbosity::MineWeb {
                        print!("\x1b[0;35m[Command]:\x1b[0m {}", cmd);
                    }
                    {
                        let server_in = child.stdin.as_mut().unwrap();
                        server_in.write_all(cmd.as_bytes()).unwrap();
                    }
                }
                Err(_) => {
                    // Input is empty after time out, there is either no new input or the server is processing the next input.
                }
            }
        }
    });

    output_handle.join().unwrap();
    if verbosity == Verbosity::Mine || verbosity == Verbosity::MineWeb {
        println!("Minecraft Server Output Thread Closed");
    }
    input_handle.join().unwrap();
    if verbosity == Verbosity::Mine || verbosity == Verbosity::MineWeb {
        println!("Minecraft Server Input Thread Closed");
    }
    web_handle.join().unwrap();
    if verbosity == Verbosity::Web || verbosity == Verbosity::MineWeb {
        println!("Web Server Thread Closed");
    }
}
