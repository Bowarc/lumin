soo.?.

what project structure do i go towards,

I don't know much about web stuff, so i don't think i can do something like [livewallpaper](https://github.com/DaZiYuan/livewallpaper/)

but something offline with a daemon+foreground app (a bit like [my WTBC project](https://github.com/Bowarc/WTBC/)) is more of my skill range

On app launch, if the daemon is not started, it silently runs it.

The thing is.. users will probably don't want to download wallpapers themselves.


Is the daemon usefull ?

i might be for playlist but if you're only gonna set a single video in repeat-inf, it's not gonna do sht

I guess create stats ? that could be fun

# Animated desktop background changer

## State

- [x] Client - Daemon comunication (used std::net::TcpStream and the Daemon has a std::net::TcpListener)
- [x] Client starts on startup if the deamon is not running (+ restarts it when daemon crashes)
- [x] Client can build, validate and send background setup requests to the daemon
- [x] Daemon can apply a given video as the desktop background (fixed options for now)
- [ ] Daemon never crashes 
- [ ] Daemon tels the client whenever it gets into a critical or client retated error.
- [ ] Daemon handles mpv process cleanly (no phantom processes)
- [ ] Daemon logs it's actions


