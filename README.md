# Godo
A command-line tool for managing different versions of [Godot Engine](https://github.com/godotengine/godot).  

Written in ***PURE*** Rust.  

> [!NOTE]  
>   
> Currently **Godo** supports Windows only, and will soon catch up with macos and linux :D
  
## Setup
1. Download the latest `.exe` executable at the [Releases Page](https://github.com/Hamster5295/godo/releases)
2. Place it into an empty folder with a considerate name, e.g. `godo`
3. Run command below from a cmd or a PowerShell:
```Bash
setx "PATH" "%PATH%;path\to\godo"
```
4. Run another cmd, and try the commands in ***Quick Start*** section out!

## Quick Start
Install the latest stable version of Godot:
```Bash
godo install
```
  
...with **Mono** support:
```Bash
godo install -m
```  

Install 3.x version:
```Bash
godo install 3
```  
  

Run Godot with latest stable version:
```Bash
godo run
```  

...with specified version:
```Bash
godo run 3
```  
  

See what's available to install:
```Bash
godo available
```
  

...with **prereleased** versions:
```Bash
godo available -p
```


What's already installed?
```Bash
godo list
```  
  

I don't want that version anymore!
```Bash
godo uninstall 4.2-stable
```
