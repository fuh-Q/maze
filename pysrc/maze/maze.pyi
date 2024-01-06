"""
Maze and maze image generation
"""

from io import BytesIO
from typing import List, Tuple, Sequence
from typing_extensions import Never

__version__: str
__all__: Tuple[str, ...]

XY = Direction = Tuple[int, int]
Rgb = Sequence[int]
Rgba = Sequence[int]

UP: Direction
DOWN: Direction
LEFT: Direction
RIGHT: Direction

class SolutionNotFound(Exception): ...

class Maze:
    def __init__(self) -> Never:
        """This class is not to be instantiated directly, use the `generate_maze` function instead"""
    def has_wall_between(self, a: XY, b: XY, /) -> bool: ...
    def undraw_at(self, xy: XY, /) -> None: ...
    def draw_player_at(self, xy: XY, /) -> None: ...
    def compute_solution(self, *, draw_path: bool) -> None: ...
    def get_solution_expensively(self) -> Tuple[int, List[str]]: ...
    def get_image_expensively(self) -> BytesIO: ...
    def move_max(self, current: XY, direction: Direction, /) -> XY: ...

def generate_maze(
    *,
    width: int,
    height: int,
    bg_colour: Rgb | Rgba,
    wall_colour: Rgb | Rgba,
    solution_colour: Rgb | Rgba,
    player: bytes | None = ...,
    endzone: bytes | None = ...,
) -> Maze: ...
