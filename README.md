# untitled-adventures
Untitled Adventures

# dungeon generation
1. pick a random location for the first room and carve it
1. pick another location for the second room such that it does not overlap with the first
1. connect the two with a tunnel and repeat, yielding a set of connected rooms
1. for every room except the first one we connect it to the previous one
    - you can start by using a vertical tunnel or a horizontal tunnel to reach the same level
    - both are valid methods, so randomly pick one each time

# build
## prerequisites
- Debian/Ubuntu: `sudo apt-get install -Y gcc g++ make libsdl2-dev`
- Fedora: `sudo dnf install -Y gcc g++ make SDL2-devel`

# resources
- Roguelike Tutorial in Rust + tcod: https://tomassedovic.github.io/roguelike-tutorial/
- Field of View discussion: http://www.roguebasin.com/index.php?title=Comparative_study_of_field_of_view_algorithms_for_2D_grid_based_worlds
