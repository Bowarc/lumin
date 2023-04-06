This only host an idea.


# 4)
Today i was able to use multiple windows of the explorer to display 3 mpv video (1 each screen)
but the problem was that the mpvs were behind the taskbar but in front of the desktop icons..
![](assets/test1.jpg "test one screenshot")

i need to learn more about how this works..



# 3)
I found a way to use a WorkerW to display mpv behind desktop icons, got some graphical bugs but i think i understood & fixed thoses.

I still don't know if there is a way to display multiple mpv for multiple screens. WorkerW is only one window for all the screens connected to the machine (hence it's size & position).

For a triple screen setup:

Monitor position: (0, 0), size: 1920 x 1080

Monitor position: (1920, 0), size: 1366 x 768

Monitor position: (-1280, 0), size: 1024 x 819

It's size is x-1280, y0, w4566, h1080.


# 2)

Then i checked [livewallpaper](https://github.com/DaZiYuan/livewallpaper/) which was very interesting.

it uses [mpv](https://github.com/mpv-player/mpv) to display a video in background

Might be a good idea to test this route


# 1)

The only thing that is currently holding me from doing it is taht i don't have a stable method to set a different wallpaper on each screen on windows

i checked [more-wallpapers](https://github.com/LuckyTurtleDev/more-wallpapers) but the feature im looking for is not available on windows

While reading [the Lively source code](https://github.com/rocksdanister/lively)

I found
```csharp
public void SetWallpaper(ILibraryModel wallpaper, IDisplayMonitor display)
```
[Line 97](https://github.com/rocksdanister/lively/blob/c27d2d04e9d4e921c83ba74465e0869402e4fc83/src/Lively/Lively/Core/WinDesktopCore.cs#L97)

Which calls [SendMessageTimeout](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessagetimeoutw) at [Line 115](https://github.com/rocksdanister/lively/blob/c27d2d04e9d4e921c83ba74465e0869402e4fc83/src/Lively/Lively/Core/WinDesktopCore.cs#L115)

With the comment
`// Send 0x052C to Progman. This message directs Progman to spawn a  WorkerW behind the desktop icons. If it is already there, nothing happens.`
progman = program manager.
[Microsoft docs](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessagetimeouta) says `Sends the specified message to one or more windows.` about this `SendMessageTimeout` function

Lively uses this function to create a window behind the desktop icons (as the comment says), or at least ping it if it already exists.

Then on [line 135](https://github.com/rocksdanister/lively/blob/c27d2d04e9d4e921c83ba74465e0869402e4fc83/src/Lively/Lively/Core/WinDesktopCore.cs#L135) Lively is looking for a window that has `SHELLDLL_DefView` as a child
Once it found that window, Lively looks for its 'next sibling' that is the WorkerW

Then, if found WorkerW Lively starts [WatchDog](https://github.com/rocksdanister/lively/tree/core-separation/src/Lively/Lively.Watchdog)

Then my comprehension of C# show it's limits, soo the next part will not be certain

Then, on [line 194](https://github.com/rocksdanister/lively/blob/c27d2d04e9d4e921c83ba74465e0869402e4fc83/src/Lively/Lively/Core/WinDesktopCore.cs#L194) it calls
```csharp
IWallpaper instance = wallpaperFactory.CreateWallpaper(wallpaper, display, userSettings);
```
and then
```csharp
instance.Show();
```
The `CreateWallpaper` function returns all sort of structures depending on the wallpaper type (video(what format), gif, static image(url ?) etc) all implementing the `IWallpaper` interface
The C# 'interface' system works a bit like a rust trait, but it can be used more like [enum_dispatch](https://docs.rs/enum_dispatch/latest/enum_dispatch/)

Anyways.

This `IWallpaper` interface has a `.Show()` function (that is different depending on what structure implement that interface).

I checked the implementation of this function for some structs and it appears that it's just booting up a process [VideoMpvPlayer.cs](https://github.com/rocksdanister/lively/blob/c27d2d04e9d4e921c83ba74465e0869402e4fc83/src/Lively/Lively/Core/Wallpapers/VideoMpvPlayer.cs#L334), [ExtPrograms](https://github.com/rocksdanister/lively/blob/c27d2d04e9d4e921c83ba74465e0869402e4fc83/src/Lively/Lively/Core/Wallpapers/ExtPrograms.cs#L143) except for [PictureWinAPI](https://github.com/rocksdanister/lively/blob/c27d2d04e9d4e921c83ba74465e0869402e4fc83/src/Lively/Lively/Core/Wallpapers/PictureWinAPI.cs#L130) that appears to be placing the image on the given monitor (This is what i would like to do)

The thing i don't understand yet is:
```csharp
WindowInitialized?.Invoke(this, new WindowInitializedArgs()
{
    Success = true,
    Error = null,
    Msg = null
});
```
But i did not found any declaration of this `WindowInitialized` object.
