"""
Maze and maze image generation
"""

from io import BytesIO
from typing import List, NamedTuple, Sequence, Tuple
from typing_extensions import Never

__version__: str
__all__: Tuple[str, ...]

_XY = _Direction = Tuple[int, int]
_Rgb = _Rgba = Sequence[int]

UP: _Direction
DOWN: _Direction
LEFT: _Direction
RIGHT: _Direction

class _Solution(NamedTuple):
    move_count: int
    directions: List[str]

class SolutionNotFound(Exception): ...

class Maze:
    def __init__(self) -> Never:
        """This class is not to be instantiated directly, use the `generate_maze` function instead"""
    def has_wall_between(self, a: _XY, b: _XY, /) -> bool: ...
    def undraw_at(self, xy: _XY, /) -> None: ...
    def draw_player_at(self, xy: _XY, /) -> None: ...
    def compute_solution(self, *, draw_path: bool) -> None: ...
    def get_solution_expensively(self) -> _Solution: ...
    def get_image_expensively(self) -> BytesIO: ...
    def move_max(self, current: _XY, direction: _Direction, /) -> _XY: ...

def generate_maze(
    *,
    width: int,
    height: int,
    bg_colour: _Rgb | _Rgba,
    wall_colour: _Rgb | _Rgba,
    solution_colour: _Rgb | _Rgba,
    player: bytes | None = ...,
    endzone: bytes | None = ...,
) -> Maze: ...
