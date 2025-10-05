//! Basic implementation of `TileMap` for `CLI`-based games!
//!
//! This module contains the [`Tile`] trait, allowing to represent other data types as `tile`,
//! or more specifically as [`StyledContent<&'static str>`], provided by the [`crossterm`] crate,
//! and the [`TileMap<T>`] type, representing a tilemap of `T`, where `T`: [`Tile`] + [`Default`],
//! which is based on the [`GridMap<V>`] from [`grid-math`] crate, which is a wrapper around the [`std::collections::HashMap`].
//!
//! [`TileMap<T>`] implements the [`Deref`] and the [`DerefMut`] traits to deref to the inner `HashMap`,
//! so we can work with it in the same way as with the [`std::collections::HashMap`].
//!
//! # Note
//!
//! - Crate is in the "work in progress" state, so the public API may change in the future. Feel free to contribute!
//!
//! # Examples
//!
//! TileMap for custom data type:
//! ```
//! use cli_tilemap::{Tile, TileMap, Formatting};
//! use crossterm::style::{Stylize, StyledContent};
//! use grid_math::Cell;
//! use std::io::stdout;
//!
//!
//! #[derive(Default, Debug)]
//! enum Entity {
//!     Enemy,
//!     Hero,
//!     #[default]
//!     Air,
//! }
//!
//! impl Tile for Entity {
//!     fn tile(&self) -> StyledContent<&'static str> {
//!         match self {
//!             Self::Air => "[-]".dark_grey().bold(),
//!             Self::Hero => "[&]".green().bold(),
//!             Self::Enemy => "[@]".red().bold(),
//!         }
//!     }
//! }
//!
//! // new 5x5 tilemap:
//! let mut map: TileMap<Entity> = TileMap::new(5, 5);
//! // insert entities:
//! map.insert(Cell::new(3, 3), Entity::Enemy);
//! map.insert(Cell::new(1, 0), Entity::Hero);
//! // draw map to the raw stdout:
//! map.draw(&mut stdout()).expect("should be able to draw to the stdout!");
//! // change row and tile spacing:
//! map.formatting.row_spacing = 2;
//! map.formatting.tile_spacing = 4;
//! // format as a string and print:
//! let map_string = map.to_string();
//! println!("{map_string}");
//! ```
//!
//! For more documentation about the `Grid`, `GridMap` and `Cell` types, visit https://crates.io/crates/grid-math

use crossterm::{
    execute,
    style::{Print, PrintStyledContent, StyledContent},
};
use grid_math::{Cell, Grid, GridMap};
use std::{
    collections::HashMap,
    convert::From,
    fmt::Display,
    io,
    ops::{Deref, DerefMut},
};

/// `Tile` allows to represent any other data type as `tile`,
/// or more specifically as `StyledContent<&'static str>`
///
/// # Examples
///
/// ```
/// use cli_tilemap::Tile;
/// use crossterm::style::{Stylize, StyledContent};
///
/// #[derive(Default, Debug)]
/// enum Entity {
///     Enemy,
///     Hero,
///     #[default]
///     Air,
/// }
///
/// impl Tile for Entity {
///     fn tile(&self) -> StyledContent<&'static str> {
///         match self {
///             Self::Air => "[-]".dark_grey().bold(),
///             Self::Hero => "[&]".green().bold(),
///             Self::Enemy => "[@]".red().bold(),
///         }
///     }
/// }
///
/// let hero = Entity::Hero;
/// assert_eq!(hero.tile(), "[&]".green().bold());
/// ```
pub trait Tile {
    fn tile(&self) -> StyledContent<&'static str>;
}

/// `Formatting` represents instructions for `TileMap<T>` on how to draw tilemap to the terminal
///
/// `row_spacing` - number of additional newlines between every row, defaults to 1
/// `tile_spacing` - number of spaces between every tile, defaults to 1
/// `top_indent` - number of newlines to insert before drawing the tilemap, defaults to 3
/// `left_indent` - number of tabs to insert at the start of every row, defaults to 1
/// `bottom_indent` - number of newlines to insert after drawing the tilemap, defaults to 2
///
/// # Examples
///
/// ```
/// use cli_tilemap::Formatting;
///
/// let f = Formatting::default();
/// assert_eq!(f.row_spacing, 1);
/// assert_eq!(f.tile_spacing, 1);
/// assert_eq!(f.top_indent, 3);
/// assert_eq!(f.left_indent, 1);
/// assert_eq!(f.bottom_indent, 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Formatting {
    pub row_spacing: u8,
    pub tile_spacing: u8,
    pub top_indent: u8,
    pub left_indent: u8,
    pub bottom_indent: u8,
}

/// Implements default values for `Formatting`
///
impl Default for Formatting {
    fn default() -> Self {
        Self {
            row_spacing: 1,
            tile_spacing: 1,
            top_indent: 3,
            left_indent: 1,
            bottom_indent: 2,
        }
    }
}

/// `TileMap<T>`, represents a tilemap over the type `T`, where `T` is `Tile` + `Default`
///
/// `TileMap<T>` is based on the `GridMap<V>` from the `grid-math` crate,
/// and implements the `Deref` and the `DerefMut` traits to deref to the inner `GridMap<T>`
///
/// # Examples
///
/// ```
/// use cli_tilemap::{Tile, TileMap, Formatting};
/// use crossterm::style::{Stylize, StyledContent};
/// use grid_math::Cell;
/// use std::io::stdout;
///
///
/// #[derive(Default, Debug)]
/// enum Entity {
///     Enemy,
///     Hero,
///     #[default]
///     Air,
/// }
///
/// impl Tile for Entity {
///     fn tile(&self) -> StyledContent<&'static str> {
///         match self {
///             Self::Air => "[-]".dark_grey().bold(),
///             Self::Hero => "[&]".green().bold(),
///             Self::Enemy => "[@]".red().bold(),
///         }
///     }
/// }
///
/// // new 5x5 tilemap:
/// let mut map: TileMap<Entity> = TileMap::new(5, 5);
/// // insert entities:
/// map.insert(Cell::new(3, 3), Entity::Enemy);
/// map.insert(Cell::new(1, 0), Entity::Hero);
/// // draw map to the raw stdout:
/// map.draw(&mut stdout()).expect("should be able to draw to the stdout!");
/// // change row and tile spacing:
/// map.formatting.row_spacing = 2;
/// map.formatting.tile_spacing = 4;
/// // format as a string and print:
/// let map_string = map.to_string();
/// println!("{map_string}");
/// ```
#[derive(Debug, Clone)]
pub struct TileMap<T>
where
    T: Tile + Default,
{
    pub formatting: Formatting,
    gridmap: GridMap<T>,
}

impl<T> TileMap<T>
where
    T: Tile + Default,
{
    /// Creates new `TileMap<T>` with the empty inner `GridMap<T>` of specified size,
    /// and with the defult `Formatting`
    ///
    /// For more info, visit `grid-math` crate docs
    ///
    pub fn new(width: u8, depth: u8) -> Self {
        Self {
            formatting: Formatting::default(),
            gridmap: GridMap::new(width, depth),
        }
    }

    /// Creates new `TileMap<T>` with the empty inner `GridMap<T>` of specified size,
    /// and with the given `Formatting`
    ///
    /// For more info, visit `grid-math` crate docs
    ///
    pub fn formatted(width: u8, depth: u8, formatting: Formatting) -> Self {
        Self {
            formatting,
            gridmap: GridMap::new(width, depth),
        }
    }

    /// Draws the `TileMap<T>` to the given `stdout`, using the inner `Formatting` rules
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_tilemap::{Tile, TileMap};
    /// use crossterm::style::{Stylize, StyledContent};
    /// use std::io::stdout;
    ///
    /// #[derive(Default)]
    /// struct Empty;
    ///
    /// impl Tile for Empty {
    ///     fn tile(&self) -> StyledContent<&'static str> {
    ///         "[-]".dark_grey().bold()
    ///     }
    /// }
    ///
    /// let mut map: TileMap<Empty> = TileMap::new(5, 5);
    /// map.draw(&mut stdout()).expect("should be able to draw to the stdout!");
    /// ```
    pub fn draw<W: io::Write>(&self, stdout: &mut W) -> io::Result<()> {
        execute!(
            stdout,
            Print("\n".repeat(self.formatting.top_indent as usize))
        )?;
        for row in self.grid().rows() {
            execute!(
                stdout,
                Print("\n".repeat(self.formatting.row_spacing as usize)),
                Print("\t".repeat(self.formatting.left_indent as usize))
            )?;
            for cell in row.cells() {
                execute!(
                    stdout,
                    Print(" ".repeat(self.formatting.tile_spacing as usize)),
                    PrintStyledContent(self.get(&cell).unwrap_or(&T::default()).tile())
                )?;
            }
            execute!(stdout, Print("\n"))?;
        }
        execute!(
            stdout,
            Print("\n".repeat(self.formatting.bottom_indent as usize))
        )?;
        Ok(())
    }
}

impl<T> Display for TileMap<T>
where
    T: Tile + Default,
{
    /// Implements `fmt` method for the `TileMap<T>` in the same way as the `draw` method works
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_tilemap::{Tile, TileMap};
    /// use crossterm::style::{Stylize, StyledContent};
    ///
    /// #[derive(Default)]
    /// struct Empty;
    ///
    /// impl Tile for Empty {
    ///     fn tile(&self) -> StyledContent<&'static str> {
    ///         "[-]".dark_grey().bold()
    ///     }
    /// }
    ///
    /// let mut map: TileMap<Empty> = TileMap::new(5, 5);
    /// println!("{map}");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "\n".repeat(self.formatting.top_indent as usize))?;
        for row in self.grid().rows() {
            write!(f, "{}", "\n".repeat(self.formatting.row_spacing as usize))?;
            write!(f, "{}", "\t".repeat(self.formatting.left_indent as usize))?;
            for cell in row.cells() {
                write!(f, "{}", " ".repeat(self.formatting.tile_spacing as usize))?;
                write!(f, "{}", self.get(&cell).unwrap_or(&T::default()).tile())?;
            }
            writeln!(f)?;
        }
        write!(f, "{}", "\n".repeat(self.formatting.bottom_indent as usize))?;
        Ok(())
    }
}

impl<T> From<Grid> for TileMap<T>
where
    T: Tile + Default,
{
    /// Creates new empty `TileMap<T>` from the specified `Grid`
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_tilemap::{Tile, TileMap};
    /// use crossterm::style::{Stylize, StyledContent};
    /// use grid_math::{Cell, Grid};
    ///
    /// #[derive(Default)]
    /// struct Empty;
    ///
    /// impl Tile for Empty {
    ///     fn tile(&self) -> StyledContent<&'static str> {
    ///         "[-]".dark_grey().bold()
    ///     }
    /// }
    ///
    /// let cells = (Cell::new(2, 2), Cell::new(5, 5));
    /// let grid = Grid::from(cells);
    /// let map: TileMap<Empty> = TileMap::from(grid);
    /// assert_eq!(map.grid(), grid);
    /// ```
    fn from(grid: Grid) -> Self {
        Self {
            formatting: Formatting::default(),
            gridmap: GridMap::from(grid),
        }
    }
}

impl<T> From<GridMap<T>> for TileMap<T>
where
    T: Tile + Default,
{
    /// Creates `TileMap<T>` from the existing `GridMap<T>` where `T`: `Tile` + `Default`
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_tilemap::{Tile, TileMap};
    /// use crossterm::style::{Stylize, StyledContent};
    /// use grid_math::{Cell, GridMap};
    ///
    /// #[derive(Debug, Default, PartialEq, Eq)]
    /// struct Empty;
    ///
    /// impl Tile for Empty {
    ///     fn tile(&self) -> StyledContent<&'static str> {
    ///         "[-]".dark_grey().bold()
    ///     }
    /// }
    ///
    /// let mut gridmap: GridMap<Empty> = GridMap::new(5, 5);
    /// let target = Cell::new(1, 2);
    /// gridmap.insert(target, Empty);
    /// let map: TileMap<Empty> = TileMap::from(gridmap);
    /// assert_eq!(map.get(&target), Some(&Empty));
    /// ```
    fn from(gridmap: GridMap<T>) -> Self {
        Self {
            formatting: Formatting::default(),
            gridmap,
        }
    }
}

impl<T> From<(Grid, HashMap<Cell, T>)> for TileMap<T>
where
    T: Tile + Default,
{
    /// Creates `TileMap<T>` from the existing `HashMap<Cell, T>` and the given `Grid`
    ///
    /// # Panics
    /// Panics if the given `HashMap<Cell, T>` contains `Cell`s that are not within the given `Grid`
    /// This panic is a part of `grid-math` crate current state, error handling may change in the future
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_tilemap::{Tile, TileMap};
    /// use crossterm::style::{Stylize, StyledContent};
    /// use grid_math::{Cell, Grid};
    /// use std::collections::HashMap;
    ///
    /// #[derive(Debug, Default, PartialEq, Eq)]
    /// struct Empty;
    ///
    /// impl Tile for Empty {
    ///     fn tile(&self) -> StyledContent<&'static str> {
    ///         "[-]".dark_grey().bold()
    ///     }
    /// }
    ///
    /// let grid = Grid::new(5, 5);
    /// let mut hashmap: HashMap<Cell, Empty> = HashMap::new();
    /// let target = Cell::new(1, 2);
    /// hashmap.insert(target, Empty);
    /// let map: TileMap<Empty> = TileMap::from((grid, hashmap));
    /// assert_eq!(map.get(&target), Some(&Empty));
    /// ```
    ///
    /// ```should_panic
    /// use cli_tilemap::{Tile, TileMap};
    /// use crossterm::style::{Stylize, StyledContent};
    /// use grid_math::{Cell, Grid};
    /// use std::collections::HashMap;
    ///
    /// #[derive(Debug, Default, PartialEq, Eq)]
    /// struct Empty;
    ///
    /// impl Tile for Empty {
    ///     fn tile(&self) -> StyledContent<&'static str> {
    ///         "[-]".dark_grey().bold()
    ///     }
    /// }
    ///
    /// let grid = Grid::new(5, 5);
    /// let mut hashmap: HashMap<Cell, Empty> = HashMap::new();
    /// let target = Cell::new(7, 1);
    /// hashmap.insert(target, Empty);
    /// let map: TileMap<Empty> = TileMap::from((grid, hashmap)); // panic!
    /// ```
    fn from(data: (Grid, HashMap<Cell, T>)) -> Self {
        Self {
            formatting: Formatting::default(),
            gridmap: GridMap::from(data),
        }
    }
}

/// Implements `Deref` trait for `TileMap<T>`, to return ref to the inner `GridMap<T>`
///
/// For more info, visit `grid-math` crate docs
///
impl<T> Deref for TileMap<T>
where
    T: Tile + Default,
{
    type Target = GridMap<T>;
    fn deref(&self) -> &Self::Target {
        &self.gridmap
    }
}

/// Implements `Deref` trait for `TileMap<T>`, to return ref to the inner `GridMap<T>`
///
/// For more info, visit `grid-math` crate docs
///
impl<T> DerefMut for TileMap<T>
where
    T: Tile + Default,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.gridmap
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::style::Stylize;
    use std::io::stdout;

    // declare Entity enum
    #[derive(Default, Debug)]
    enum Entity {
        Enemy,
        Hero,
        #[default]
        Air,
    }

    // represent Entity as tile
    impl Tile for Entity {
        fn tile(&self) -> StyledContent<&'static str> {
            match self {
                Self::Air => "[-]".dark_grey().bold(),
                Self::Hero => "[&]".green().bold(),
                Self::Enemy => "[@]".red().bold(),
            }
        }
    }

    #[test]
    fn draw_tilemap() {
        // create 5x5 tilemap:
        let mut map: TileMap<Entity> = TileMap::new(5, 5);
        // insert entities:
        map.insert(Cell::new(3, 3), Entity::Enemy);
        map.insert(Cell::new(1, 0), Entity::Hero);
        // draw map to the raw stdout:
        map.draw(&mut stdout()).expect("should draw!");
    }
}

// 🦀!⭐!!!
