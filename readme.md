soo.?.

what project structure do i go towards,

I don't know much about web stuff, so i don't think i can do something like [livewallpaper](https://github.com/DaZiYuan/livewallpaper/)

but something offline with a daemon+foreground app (a bit like [my WTBC project](https://github.com/Bowarc/WTBC/)) is more of my skill range

On app launch, if the daemon is not started, it silently runs it.

The thing is.. users will probably don't want to download wallpapers themselves.


Is the daemon usefull ?

it might be for playlist but if you're only gonna set a single video in repeat-inf, it's not gonna do sht

I guess create stats ? that could be fun

# Animated desktop background changer

## State - Working but not finished

- [x] Client - Daemon comunication (used std::net::TcpStream and the Daemon has a std::net::TcpListener)
- [x] Client boots up daemon and connects to it on startup
- [x] Client can build, validate and send background setup requests to the daemon
- [x] Daemon can apply a given video as the desktop background (fixed options for now)
- [x] Client can tell daemon to add and remove backgrounds
- [x] Better ux with [egui notify](https://github.com/ItsEthra/egui-notify)
- [x] Daemon informs the client of the currently playing backgrounds when client connects
- [x] Daemon never crashes (well, sort of.. due to how explorer and windows works, the daemon has to restart in some cases, so i make the client somewhat smart with daemon crashes) 
- [x] Daemon tells the client whenever it gets into a critical or client retated error.
- [x] Daemon handles mpv process cleanly (no phantom processes)
- [ ] Daemon logs it's actions
- [ ] Multiple animated backgrounds

https://user-images.githubusercontent.com/63136904/233821402-66cf7828-48ba-4efb-8f36-633233083b74.mp4


# Memory/cpu usage
- Memory: 180MB / 200MB, ~80MB GPU - in debug
- Cpu: bewteen 1.5% and 3%  - in debug


# Ideas
- Youtube downloader !!MPV WORKS IF YOU GIVE IT A YT LINK, So you can still make a downloader, but make it a
  toggle-able option
- Settings for sound volume, speed, etc..
- gh pages for easy download
- gh actions for simple compilation & versioning with pages
- easy graphical installer for install (duh) and version managing
- run on startup with the last background played, easy with the installer and a config file
- tray menu ?
- Stop the daemon if the client disconnects and there is no backrounds playing (It's useless to keep the daemon
  running if it's just gonna be idle)

# Dev questions: 
- About he installer: 
    - As the installer will be used to update and uninstall, how does the installer keeps track of the existing
      Lumin install(s, multiple is usless btw)
    - There will be one installer, any installer update(there will not be a lot) will be managed through a self 
      updater 
- About the client, 
    - A global refactor where the client never stops and serves the purpose of the actual client + daemon
      but there is still the question of: How do the client manage WorkerW not found error on windows?
        There is 2 solution to this: 
        - 1, 'WorkerW is fetch by another tiny program'
        - 2, 'FIX THE FCING METHOD'
      This could allow us to:
        - Remove all the client - daemon comunication code, would simplify error management etc.
        - Put the client in the tray menu and only close the window when the user 'stops' the app


