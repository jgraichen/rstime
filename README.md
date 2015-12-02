# rstime

A small utility to export one or multiple GNOME hamster applet databases into tsv format.

Written in Rust.

## Usage

```
./target/release/rstime [paths]
```

Export GNOME hamster applet databases from given paths. Paths defaults to `hamster.db*` in default hamster location.

### Examples

Export databases from e.g. multiple systems synchronized via owncloud:

```
./target/release/rstime ~/ownCloud/hamster/*.db
```

Export items from November 2015:

```
./target/release/rstime ~/ownCloud/hamster/*.db | grep 2015-10
```

Copy to clipboard to paste in e.g. excel sheet for accounting department:

```
./target/release/rstime ~/ownCloud/hamster/*.db | grep 2015-10 | xclip
```

## License

```
Copyright (C) 2015 Jan Graichen <jgraichen@altimos.de>

This program is free software; you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation; either version 2 of the License, or (at your option) any later
version.

This program is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
this program; if not, write to the Free Software Foundation, Inc., 51 Franklin
Street, Fifth Floor, Boston, MA 02110-1301 USA.
```
