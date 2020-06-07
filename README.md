# Jagged
Jagged provides an API to read data entries from JAG archives.

## Example
```rust
use anyhow::Result;
use jagged::{Archive, ReadWriteArchive};
use std::path::Path;

fn main() -> Result<()> {
    let mut archive = Archive::new();
    let data: Vec<u8> = include_bytes!("some_file").to_vec();
    let output_path = Path::new("archive.jag")?;

    assert_eq!(archive.len(), 0);

    let _ = archive.insert("some_name", data);
    archive.save(output_path)?;
}
```

For more examples see the [examples](https://github.com/hikilaka/jagged/tree/master/examples) directory of this repository.

## License
This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program. If not, see [http://www.gnu.org/licenses/](http://www.gnu.org/licenses/).
