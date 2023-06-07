# r-gh-create-release

A wrapper utility of `gh release`.


# Commandline options.

```
    -h, --help          usage
        --publish       go publish
        --dry-run       dry run
        --determine-version-from STRING
                        Determines version string from file. (Cargo.toml,
                        etc...)
        --notes STRING  string
        --tag STRING    create release using tag.
        --title STRING  string
        --target STRING string
        --file ARRAY    string
```

# Examples

### Create release with 1 attachment from your main branch.
* Release notes will be created automatically.

```sh
r-gh-create-release --file path/to/your/app
```

### Create release with title and 1 attachment.
* Release notes will be created automatically.

```sh
r-gh-create-release --title "My First Release!" --file path/to/your/app
```

### Create release with release notes and 1 attachment.

```sh
r-gh-create-release --notes "Your Release Note" --file path/to/your/app
```

### Create release with release notes and 1 attachment.
* Release notes will be created from file.

```sh
r-gh-create-release --notes path/to/your/notes.txt --file path/to/your/app
```

### Create release from your branch.
```sh
r-gh-create-release --target your-branch-name --file path/to/your/app
```

# Download

* Latest versions are available here
  * Windows  
    https://github.com/mass10/r-gh-create-release/releases/latest/download/r-gh-create-release.exe
  * Linux  
    https://github.com/mass10/r-gh-create-release/releases/latest/download/r-gh-create-release
