# lith
lith is a simple lithophane generator written in rust, with a basic GUI interface made with EGUI.

## Usage
To install lith to your system, run the following command:
```sh
cargo install --path .
```
Then just run `lith` from the terminal to launch the application.

### Scaling
The scaling parameter controls how much a change in brightness will affect the mesh height. A value of around 2.0 is recommended.  
  
The lightness value of each pixel is multiplied by the scaling before mesh generation. A scaling value *n* will result in the maximum
height of the final mesh being exactly *n* millimeters. I have gotten good results printing lithophanes at around 2 millimeters tall, but your results may vary.

### Width
The width parameter controls the width of the final mesh. The height will be auto-scaled to maintain the target aspect ratio. A smaller width
will mean less detail in the final mesh but will greatly reduce the processing time, memory usage, and output file size.

### Output
lith will generate a file with the same name as the input, but with the `.stl` extension. This file will be placed in the same directory
as the input file. In some cases, there may be a large random chunk of triangles that has nothing to do with the actual generated lithophane.
If this happens, then increase the width by 1 unit, regenerate, and it will (probably) be fixed. I do not know if why this bug happens or if it
still exists but it has happened before.

### Slicing
Ultimaker Cura has been tested and will play nice with generated files. There is no reason any other slicer shouldn't work, but I cannot confirm
whether or not they will work. There may be some model errors but it should probably work fine.