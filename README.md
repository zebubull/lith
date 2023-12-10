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
lith generates a **flat**, **unclosed** mesh. It is up to the slicer to actually finish producing the model ~~because I was too lazy to implement it~~.
Ultimaker Cura is tested and works (relatively) well. The behaviour of other slicers is unknown. Note that whatever slicer you use will likely display
multiple errors and heavily complain about a possibly corrupt file. This is not indicative of a bad mesh, although it should be noted that it is
also not indicative of a good mesh (I do not have a lot of faith that this program works reliably).