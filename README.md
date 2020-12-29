# Minecraft Monitor

What is this? This program runs a minecraft server in a child process, input and output to the server are handled in seperate threads. There is a seperate web server that gets launched along with the server that lets a user get information about the current state of the minecraft server and lets the user input commands to the server.

** Currently the server binds to the port 8000 on localhost **

## How to use this?
Linux
```bash
$ git clone https://github.com/nuhtan/minecraft_monitor.git && cd minecraft_monitor
$ mkdir server
$ cargo build --release
$ ./target/release/minecraft_monitor
```

Windows
```powershell
git clone https://github.com/nuhtan/minecraft_monitor.git; cd minecraft_monitor
mkdir server
cargo build --release
target\release\minecraft_monitor.exe
```
\*This tool expects a 'server' directory to contain the server jar. Any .jar should work, default, bukkit, spigot, paper, tuinity. For all cases change the child args in [main.rs](src/main.rs). This process will change in the future to support both a configuration file and args.

## Features that are in progress
- Add systems for setting the child arguments without having to recompile the project. This will include a configuration file and arguments, arguments will be taken with precedence over the configuration file.

## Planned changes for the future
- Shutting down the server does not exit the application which is the intended use case.
- There is no way to restart the server.
- There is currently no ui in the web server.
- Have releases for the project on github.
- Refactor the data contained in Arc<Mutex<>>'s, the handle_connections() function is a mess, I think that using a struct would be the ideal solution but that introduces explicit runtimes of which I have no experience.
- A verbose mode?
- Have an option for what ip and port the web server are bound to, probably in the configuration file and arguments.
- Output should be logged.
- Http error's should contain page content so that the browser can still properly load.
- Length checking for detecting when a player joins or leaves.
- Read when the minecraft server has finished starting, Done (time)! and then execute the list command to get the max player count.
- Detect current os and determine what directory indicators to use (/) or (\\\\). I think that this will be primarily just detecting if Windows is being used. std::env::consts::OS should be used, this should be done once cli args and a config file works.

## Completed Features
- The current players does not update in anyway, I need to read console output to determine when a player joins or leaves. Issues arise as I think that server plugins can change the prefixes and suffixes for general chat so it might be possible that a player sending a message could be interpreted as a player leaving or joining as there is currently not any semblance of relation between different server outputs, ie. When a player joins a server there are typically three output messages but I need to verify that they are always together or potentially write a regex for the messages that references the current list of players.
- Restructure file to have helper functions in a separate file.
- Extract reading server lines to the minecraft_related.rs file, player joining and leaving is a target for this.

## Other notes
- If running in WSL 2 please note that ports are no longer automatically forwarded to Windows, also note that now that WSL is more akin to a hypervisor the ip address will change on both WSL and Windows restarts. Binding a single address can be done with: 
```powershell
netsh interface portproxy add v4tov4 listenport=port listenaddress=0.0.0.0 connectport=port connectaddress=WSLAddress
```