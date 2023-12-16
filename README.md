# simple_windows
Simple windows is a rust library crate that lets me use features from the Windows API without putting windows api calls directly in the binary crate that I am working on. The goal is to put the bare minimum work on the binary that uses it and to have the windows crate as its only depedency. This is designed for simple cpu bitmap graphics with optimized repainting. I may add support in the future for painting functions, but I plan to keep the window primarily a bitmap with menu options. Consistent framerates aren't supported, this is intended for apps that update the display on a need-to basis and not really real time animation.