# THIS IS A SMALL SCRIPT I USE TO UNDERSTAND MORE ON HOW THIS METHOD WORKS, PLEASE IGNORE IT

import win32gui


progman = 0x9408b4
workerW = 0xf0808


# Left screen
win32gui.MoveWindow(progman, -1280, 0, 1024, 819, True)

# Center screen
win32gui.MoveWindow(0x1306cc, 0, 0, 1920, 1080, True)

# Right screen
win32gui.MoveWindow(workerW, 1920, 0, 1366, 768, True)
