# yexp

`yexp` is a small tool for expand yaml files

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
extend: path/to/b.yaml
```

`yexp /path/to/c.yaml` outputs:

```yaml
foo: bar
items:
- one
- two
- three
```
