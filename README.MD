# jvox

<center>
    <img src="render.png">
</center>

This code is not organized, safe, idiomatic, efficient, or bug-free. However, it serves as a nice experimentation bed for 3D rendering with wgpu-rs. At the time of writing this, the feature set of this program is as follows:

- Render some basic simplex-noised based terrain, painted with some of the worst textures anybody has ever seen.
- Enable basic movement around the scene; currently movement is mapped to the arrow keys or `FRST` (_not_ `WASD`!). Sorry QWERTY users. `Q` can be used to move in the up direction, and `W` can be used to move in the down direction.

The shaders used by the program are included in both `GLSL` and binary (`SPIR-V`) formats. These shaders can be compiled manually using `make` (`glslangValidator` must be in `$PATH`).

Only Windows is known to work, though the code should be fully adaptable to other operating systems, and the browser.
