`yexp` is a small tool for expand yaml files


- [Examples](#examples)
- [Installation](#installation)
  - [Via cargo](#via-cargo)

## Examples


`a.yaml`
```yaml
- one
- two
- three
```

`b.yaml`
```yaml
items: !import path/to/a.yaml
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

### Via cargo

```bash
cargo install yexp
```
