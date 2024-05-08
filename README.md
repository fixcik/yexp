`yexp` is a small tool for expand yaml files

- [Examples](#examples)
- [Installation](#installation)
  - [Prebuilt Binaries](#prebuilt-binaries)
  - [Via cargo](#via-cargo)
  - [Via homebrew](#via-homebrew)

## Examples

`a.yaml`

```yaml
- one
- two
- three
```

`b.yaml`

```yaml
items: !include path/to/a.yaml
```

`c.yaml`

```yaml
foo: bar
extend: # <- can be string or sequence of strings
  - path/to/b.yaml
```

`yexp /path/to/c.yaml` outputs:

```yaml
foo: bar
items:
  - one
  - two
  - three
```

## Installation

### Prebuilt Binaries

Download the latest releases from the [GitHub release page](https://github.com/fixcik/yexp/releases).

### Via cargo

```bash
cargo install yexp
```

### Via homebrew

```bash
brew tap fixcik/tap
brew install yexp
```
