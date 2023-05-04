Lumin is an application to set any video as your wallapper (maybe more later)

For non-developpers{
    lumin_client.exe => The client of lumin, the head of the app. Launch this to boot up lumin
                        You can close it whithout any consequences
    lumin_daemon.exe => The brain of lumin, a background process that the client automaticly runs on start.
                        Do not run it yourself
                        Runs at 10 ticks per secconds for minimal cpu usage
    lumin_mpv.exe => The video player that will be thrown on top of your wallapaper in your desktop background

    The daemon stays open even if you close the client to monitor the lumin_mpv processes and to be able to close the client anytime.
    If you manually kill the daemon, you wont be able to modify running backgrounds and you'll have to clean any running backgrounds yourself. (`taskkill /im "lumin_mpv.exe" /f` should do job)


    Almost forgot.. If you wish to execute the client from your desktop etc.. do not move the executable (lumin_client.exe) create a shortcut and move that shortcut
}
    
For developpers{
    My github is `github.com/Bowarc`
    Lumin's source code is at `github.com/Bowarc/lumin`
    It's a bit ugly and unfinished for now but it works well

    The daemon stores the running background(s) (only one is supported for now)
    so you can close the client and re-open it freely without losing any data
}

If You have any questions, my discord is `Bowarc#4159` feel free to dm me anytime

For bug report/feature request you can create an issue at `github.com/Bowarc/Lumin/issues/new`


- Bowarc