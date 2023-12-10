# lith
lith is a simple lithophane generator written in rust, with a basic GUI interface made with EGUI.

## Usage
To install lith to your system, run the following command:
```sh
cargo install --path .
```
Then just run `lith` from the terminal to launch the application.

### Scaling
The scaling parameter controls how much a change in brightness will affect the mesh height. A value between 0.05 and 0.15 is recommended.

### Width
The width parameter controls the width of the final mesh. The height will be auto-scaled to maintain the target aspect ratio. A smaller width
will mean less detail in the final mesh but will greatly reduce the processing time.

### Output
lith will generate a file with the same name as the input, but with the `.stl` extension. This file will be placed in the same directory
as the input file. In some cases, there may be a large extraneous chunk of triangles that has nothing to do with the actual generated lithophane.
If this happens, then increase the width by 1 unit, regenerate, and it will (probably) be fixed.

### Slicing
Ultimaker Cura has been tested and will play nice with generated files. There is no reason any other slicer shouldn't work, but don't be surprised
if they complain. If you don't want to print the brim, then move the mesh down 1 layer height in your slicer. Note that this may remove non-brim material
if your source image has some very bright parts (This method has once again only been tested in Ultimaker Cura so it may not work for other slicers).