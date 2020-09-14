# garmin_export
Export FIT files from connect.garmin.com

**Work in progress**

## Quickstart

To build:

```
nix develop --command cargo build
```

To run:

```
nix develop --command cargo run <USERNAME> <PASSWORD>
```

```
nix run github:wagdav/garmin_export
```

## Acknowledgements

I read the source code of these projects to understand the idiosyncratic API of
Garmin Connect.

* [garminexport](https://github.com/petergardfjall/garminexport)
* [python-garminconnect](https://github.com/cyberjunky/python-garminconnect)
* [tapiriik](https://github.com/cpfair/tapiriik)
