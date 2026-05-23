# Godo
A command-line tool for [Godot Engine](https://github.com/godotengine/godot) version management.  

Written in ***PURE*** Rust.  
  
## Installation

You can install `godo` by downloading the artifact manually in Github Release page


## Configuration

`godo` is configurable through config file which lies in `~/.godo/config.toml`

Here is an example:

```toml
# The directory for engines to install
engine_dir = "/home/<user>/.godo/engine"

# The temporary directory for downloading engine tarballs
temp_dir = "/tmp/godo"

# The invalidation time for local cache of Godot Release List from Github
# This avoids accessing to Github API everytime, which might trigger the rate limit
invalidate_time = 10800

# Optional, Enter your github token for higher rate limit to access Github API
# A personal access token for Public Repostories without any permission is enough
github_token = "<token>"
```


## Quick Start

List all the Godot Versions (and highlight ones you've installed)
```shell
godo list
```

Install the latest Godot 4.x release

```shell
godo install 4
```

Uninstall a specific version
```shell
godo rm 4.3.1
```

Set the **current** engine version, which creates a symlink in the engine directory called `current`
```shell
godo current 4.6.3
```

And finally, Launch the current Engine!
```shell
godo run
```

Of course you can launch a specific version
```shell
godo run 4.3.2
```
