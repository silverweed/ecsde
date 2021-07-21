### NOTES
- Currently hotloading is not working on Windows (untested on Mac) due to a GLFW crash. For some reason the same thing is working on Linux, but I haven't had time to investigate exactly why.

- The gfx backend currently uses OpenGL >= 3.3 but this may change in the future. 

- To add a font, its MSDF atlas must be generated. Currently I'm using [msdf-atlas-gen](https://github.com/Chlumsky/msdf-atlas-gen) to do so. Inexplicably, I haven't written a script to automatize the process yet, but I'll probably do it soon. 
