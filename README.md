# Godo
A command-line tool for managing different versions of [Godot Engine](https://github.com/godotengine/godot)s.  

Written in Rust.  

> [!INFO]  
> Currently **Godo** supports Windows only, and will soon catch up with macos and linux :D

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
  

See what's available to install!
```Bash
godo available
```
  

...with **prereleased** versions
```Bash
godo available -p
```


What's already installed?
```Bash
godo list
```  
  

I don't want the version anymore!
```Bash
godo uninstall 4.2-stable
```
