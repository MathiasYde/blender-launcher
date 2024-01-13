# Simple Blender Launcher
![](assets/blender_launcher_program.png)

Easily launch different versions of Blender

# Features
- Launch different versions of Blender
- Command line arguments passed to Blender Launcher is also passed to Blender

# Configuration
The launcher is configured with a YAML file, usually located at ``%USERPROFILE%\blender_launcher_config.yaml``.
The config filepath is configurable by the ``BLENDER_LAUNCHER_CONFIG_FILEPATH`` environment variable.

## Features
- Register .blend file extension to preferred build
- Custom startup arguments for launching Blender

# Add a new Blender instance
Simply drag and drop the Blender executable onto the window.

# What makes this project special?
This project is obviously heavily inspired by [DotBow's Blender Launcher](https://github.com/DotBow/Blender-Launcher),
however this launcher does not aim to do any of the fancy features that DotBow's launcher does.
It does NOT automatically check for new Blender builds, it does NOT download and install Blender builds for you.
You must download the portable version of Blender yourself and add it to the launcher.
