# Eson Coming soon ...

## Eson

is a tiny config language that extends from json.

## Features

- [x] Support expression
- [x] Support comments
- [x] Support multi-line string
- [x] Support annotation
- [x] Support

## Example

```eson
@license("Apache-2.0")
@example
{
    // This is a comment
    "name": "Eson",
    "version": "0.1.0",
    "description": "A tiny config language that extends from json.",
    "author": "Yuri"
    "copyright": f"@${ date() | format('YYYY') }",

    // This is a multi-line string
    ...
}
```

## License

Apache-2.0
