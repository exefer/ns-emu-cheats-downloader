# ns-emu-cheats-downloader

A graphical user interface for downloading cheats for Switch emulators.

This is my first GUI application in Rust and my first time using an immediate mode GUI library. The entrypoint code is somewhat messy, but improvements will come over time.

There is no persistence by design - storing only the mod data location isn't worth creating a config file.

## Supported Providers

- https://github.com/blawar/titledb
- https://github.com/ChanseyIsTheBest/NX-60FPS-RES-GFX-Cheats
- https://www.cheatslips.com
- ~https://gbatemp.net/threads/cheat-codes-ams-and-sx-os-add-and-request.520293/~ (Work in progress)
- https://github.com/HamletDuFromage/switch-cheats-db
- https://github.com/ibnux/switch-cheat
- https://tinfoil.io

Credits to the maintainers of these sites and repositories and their cheat submitters - all credit for the cheats goes to them.
