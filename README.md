the underlying maze generation library for one of my discord bots, [Amaze](https://discord.com/oauth2/authorize?client_id=988862592468521031&permissions=604490816&scope=bot%20applications.commands)

## if you want to use this yourself for whatever reason

- first off, this isn't on PyPI

a collection of pre-built wheels can be found in the [releases](https://github.com/fuh-Q/maze/releases/tag/maze) section of the repository
<br>
> only wheels for python 3.10, 3.11, and 3.12 are available and ONLY for linux, as i only built the wheels i personally needed built


to install a wheel, you can download it from the [releases](https://github.com/fuh-Q/maze/releases/tag/maze) section, and install the package using
```sh
pip install path_to_wheel.whl
```

alternatively if you have Rust installed, you can build the package yourself from source, using the following command
```sh
pip install git+https://github.com/fuh-Q/maze.git
```
