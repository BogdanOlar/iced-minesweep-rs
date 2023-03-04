use rand::Rng;

/// The characteristics of the minefield
#[derive(Clone, Debug)]
pub struct Minefield {
    /// The mine field
    field: Vec<Vec<Spot>>,

    /// Number of mines in the field
    mines: u32,

    /// Width of field grid
    width: u16,

    /// Height of field grid
    height: u16,
}

impl Minefield {
    /// Create an empty minefield grid (with all spots hidden), with the given width and height
    pub fn new(width: u16, height: u16) -> Self {
        // Enforce a minimum number of spots
        let width = if width == 0 { 1 } else { width };
        let height = if height == 0 { 1 } else { height };

        // Create empty field, with all spots hidden
        let field = vec![vec![Spot::default(); height as usize]; width as usize];

        // Create empty Minefield
        Minefield {
            field,
            mines: 0,
            width,
            height,
        }
    }

    /// Build an existing minefield with the given number of mines randomly placed in it
    pub fn with_mines(mut self, mines: u32) -> Self {
        // Total number of spots in our field
        let spot_count = self.width as usize * self.height as usize;

        // Limit the max number of mines to the number of available spots
        let mines = if mines as usize <= spot_count { mines } else { spot_count as u32 };

        self.mines = mines;

        // Add mines to minefield

        // We could just start randomly picking indices in the field and hope we haven't picked them before, but if a
        // user desires a field full of mines, then waiting for the last mines to be placed might take a long time
        // (e.g. if the field is very large).
        // That's a problem for an immediate GUI.
        // So, instead, we'll use some memory in order to ensure that the user can step on a mine as soon as humanly
        // possible.
        let mut spots_remaining: Vec<usize> = (0..spot_count).collect();
        let mut rng = rand::thread_rng();

        // Place mines
        for _ in 0..self.mines {
            let index_rm = rng.gen_range(0..spots_remaining.len());
            let index = spots_remaining.swap_remove(index_rm);
            let x = (index as u32 % self.width as u32) as u16;
            let y = (index as u32 / self.width as u32) as u16;
            self.place_mine(x, y);
        }

        self
    }

    /// Step on a given spot of the field. Coordinates [x=0, y=0] represent the top-left point of the field grid
    pub fn step(&mut self, x: u16, y: u16) -> StepResult {
        if let Some(spot) = self.spot_mut(x, y) {
            match spot.kind {
                SpotKind::Mine => {
                    // Stepped on a mine
                    spot.state = SpotState::Exploded;
                    StepResult::Boom
                },

                SpotKind::Empty(n) => {
                    // Reveal the spot
                    spot.state = SpotState::Revealed;

                    // flood reveal if this is an empty spot with no neighboring mines
                    if n == 0 {
                        let mut spots_to_visit = vec![(x, y)];

                        while let Some((xx, yy)) = spots_to_visit.pop() {                            
                            for (nb_x, nb_y) in self.neighbors_coords(xx, yy) {
                                let spot = &mut self.field[nb_x as usize][nb_y as usize];
                                
                                if SpotState::Hidden == spot.state {
                                    if let SpotKind::Empty(n) = spot.kind {
                                        spot.state = SpotState::Revealed;

                                        if n == 0 {
                                            spots_to_visit.push((nb_x, nb_y));
                                        }   
                                    }                                
                                }
                            }
                        }
                    }

                    // Stepped on empty field
                    StepResult::Phew
                },
            }
        } else {
            // Step is outside minefield
            StepResult::Invalid
        }
    }

    /// Automatically step on all hidden neighbors (i.e. not flagged) of a revealed spot at the given coordiantes
    pub fn auto_step(&mut self, x: u16, y: u16) -> StepResult {
        if let Some(spot) = self.spot(x, y) {
            if let SpotKind::Empty(mines) = spot.kind {
                // count the flags around the given coords
                let placed_flags = self
                    .neighbors_coords(x, y)
                    .filter(|(x, y)| self.field[*x as usize][*y as usize].state == SpotState::Flagged)
                    .count() as u32;
                
                // only try to autostep if the user has placed enough flags around the step whose neighbors will be autorevealed
                if spot.state == SpotState::Revealed  && placed_flags == mines {
                    for (nx, ny) in self.neighbors_coords(x, y) {
                        if SpotState::Hidden == self.field[nx as usize][ny as usize].state {
                            let step_result = self.step(nx, ny);

                            // Stepped on an unflagged mine!
                            if step_result != StepResult::Phew {
                                return step_result;
                            }
                        }
                    }
                }

                StepResult::Phew
            } else {
                StepResult::Invalid
            }
        } else {
            StepResult::Invalid
        }
    }

    /// Check if the minefield has been cleared
    pub fn is_cleared(&self) -> bool {
        for col in &self.field {
            for spot in col {
                // All mines must be flagged, and all other spots must be revealed
                match spot.kind {
                    SpotKind::Mine => {
                        if spot.state != SpotState::Flagged {
                            return false;
                        }
                    },
                    SpotKind::Empty(_) => {
                        if spot.state != SpotState::Revealed {
                            return false;
                        }
                    },
                }
            }
        }
        
        true
    }

    /// Set a flag on a hidden spot, or clear the flag if the spot had one, or do nothing if
    /// the spot cannot be flagged
    pub fn toggle_flag(&mut self, x: u16, y: u16) -> FlagToggleResult {
        if let Some(mut spot) = self.spot_mut(x, y) {
            match spot.state {
                SpotState::Hidden => {
                    spot.state = SpotState::Flagged;
                    
                    // we've added a flag
                    FlagToggleResult::Added
                },
                SpotState::Flagged => {
                    spot.state = SpotState::Hidden;

                    // we've removed a flag
                    FlagToggleResult::Removed
                },
                _ => {
                    // no flag was added or removed
                    FlagToggleResult::None
                },
            }
        } else {
            // invalid coordinates, no flag was added or removed
            FlagToggleResult::None
        }
    }

    /// The width of the minefield
    pub fn width(&self) -> u16 {
        self.width as u16
    }

    /// The height of the minefield
    pub fn height(&self) -> u16 {
        self.height as u16
    }

    /// The number of mines in the minefield
    pub fn mines(&self) -> u32 {
        self.mines
    }    

    /// Get a reference to a spot at the given coordinates in the minefield
    pub fn spot(&self, x: u16, y: u16) -> Option<&Spot> {
        if (x < self.width) && (y < self.height) {
            Some(&self.field[x as usize][y as usize])
        } else {
            None
        }
    }

    /// Get a mutable reference to a spot at the given coordinates in the minefield
    fn spot_mut(&mut self, x: u16, y: u16) -> Option<&mut Spot> {
        if (x < self.width) && (y < self.height) {
            Some(&mut self.field[x as usize][y as usize])
        } else {
            None
        }
    }    

    /// Place a mine at a given field coordiantes, and update neighboring spots
    fn place_mine(&mut self, x: u16, y: u16) {
        let spot = &mut self.field[x as usize][y as usize];

        // Only place a mine in an emty field
        if let SpotKind::Empty(_) = spot.kind {
            // place the mine
            spot.kind = SpotKind::Mine;

            // update neighboring empty spots
            for (nx, ny) in self.neighbors_coords(x, y) {
                let spot = &mut self.field[nx as usize][ny as usize];

                // increment count of neighboring mines for this spot
                if let SpotKind::Empty(n) = &mut spot.kind {
                    *n += 1;
                }
            }
        }
    }

    /// Iterator over the coordinates of all neighbors in a range of 1 unit, relative to the given coordiantes
    fn neighbors_coords(&self, x: u16, y: u16) -> impl Iterator<Item = (u16, u16)>
    {        
        let min_x = if x > 0 {x - 1} else {x};
        let max_x = if x < u16::MAX {x + 1} else {x};

        let min_y = if y > 0 {y - 1} else {y};
        let max_y = if y < u16::MAX {y + 1} else {y};

        let width = self.width;
        let height = self.height;

        (min_x..=max_x)
            .flat_map(move |i| {
                (min_y..=max_y).map(move |j| (i, j))
            })
            .filter(move |(neighbor_x, neighbor_y)| {
                *neighbor_x < width && 
                *neighbor_y < height && 
                !(*neighbor_x == x && *neighbor_y == y)
            })       
    }
}

/// Type of spot in a minefield
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SpotKind {
    /// This spot is a mine
    Mine,

    /// This is an empty spot, surrounded by `N` mines
    Empty(u32),
}

/// State of the spot in a minefield
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SpotState {
    /// This spot has not been visited
    Hidden,

    /// This spot has been visited
    Revealed,

    /// This spot has been flagged as being a mine
    Flagged,

    /// This spot is an exploded mine
    Exploded,
}

/// Spot struct describing the characteristics of the minefield at a particular position
#[derive(Copy, Clone, Debug)]
pub struct Spot {
    kind: SpotKind,
    state: SpotState,
}

impl Spot {
    pub fn kind(&self) -> SpotKind {
        self.kind
    }

    pub fn state(&self) -> SpotState {
        self.state
    }
}

impl Default for Spot {
    fn default() -> Self {
        Self { kind: SpotKind::Empty(0), state: SpotState::Hidden }
    }
}

/// The result of steppin on a spot in the minefield
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum StepResult {
    /// Stepped on empty spot
    Phew,

    /// Stepped on a mine
    Boom,

    /// Step not taken
    Invalid
}

/// The result of toggling a flag in the mine field
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FlagToggleResult {
    /// Exstng flag was removed
    Removed,
    /// A flag was added
    Added,
    /// No flag placed or removed
    None
}

 #[cfg(test)]
 mod tests {
    use super::*;

     #[test]
     fn new_minefield() {
        // Create empty test minefield:
        //     0 1 2
        // 0 [       ]
        // 1 [       ]
        // 2 [       ]
        // 3 [       ]
        //
        let width = 3;
        let height = 4;
        let minefield = Minefield::new(width, height);

        for col in &minefield.field {
            for spot in col {
                assert_eq!(spot.kind, SpotKind::Empty(0));
                assert_eq!(spot.state, SpotState::Hidden);
            }
        }
     }

     #[test]
     fn place_mines() {
         // Create empty minefield
        let width = 3;
        let height = 4;
        let mut minefield = Minefield::new(width, height);

        // Place Mine
        //     0 1 2
        // 0 [   1 ☢ ]
        // 1 [   1 1 ]
        // 2 [       ]
        // 3 [       ]
        //
        let mine_x = 2;
        let mine_y = 0;
        minefield.place_mine(mine_x, mine_y);

        // Was mine placed correctly?
        assert_eq!(minefield.field[mine_x as usize][mine_y as usize].kind, SpotKind::Mine);

        // Were the neighbors updated correctly?
        for (nx, ny) in minefield.neighbors_coords(mine_x, mine_y) {
            assert_eq!(minefield.field[nx as usize][ny as usize].kind, SpotKind::Empty(1));
        }

        // Place another mine
        //     0 1 2
        // 0 [   1 ☢ ]
        // 1 [   1 1 ]
        // 2 [ 1 1   ]
        // 3 [ ☢ 1   ]
        let mine_x = 0;
        let mine_y = 3;
        minefield.place_mine(mine_x, mine_y);

        // Was mine placed correctly?
        assert_eq!(minefield.field[mine_x as usize][mine_y as usize].kind, SpotKind::Mine);

        // Were the neighbors updated correctly?
        for (nx, ny) in minefield.neighbors_coords(mine_x, mine_y) {
            assert_eq!(minefield.field[nx as usize][ny as usize].kind, SpotKind::Empty(1));
        }

        // Place another mine
        //     0 1 2
        // 0 [ 1 2 ☢ ]
        // 1 [ ☢ 2 1 ]
        // 2 [ 2 2   ]
        // 3 [ ☢ 1   ]
        let mine_x = 0;
        let mine_y = 1;
        minefield.place_mine(mine_x, mine_y);

        // Was mine placed correctly?
        assert_eq!(minefield.field[mine_x as usize][mine_y as usize].kind, SpotKind::Mine);

        // Were the neighbors updated correctly?
        for n_coords in minefield.neighbors_coords(mine_x,  mine_y) {
            let expected_mine_count = if n_coords == (0, 0) { 1 } else { 2 };
            assert_eq!(minefield.field[n_coords.0 as usize][n_coords.1 as usize].kind, SpotKind::Empty(expected_mine_count));
        }
     }

     #[test]
     fn step() {
         // Create empty minefield
         let width = 3;
         let height = 4;
         let mut minefield = Minefield::new(width, height);

        // Place mines
        //     0 1 2
        // 0 [   1 ☢ ]
        // 1 [   1 1 ]
        // 2 [ 1 1   ]
        // 3 [ ☢ 1   ]
        let mine_x = 2;
        let mine_y = 0;
        minefield.place_mine(mine_x, mine_y);
        let mine_x = 0;
        let mine_y = 3;
        minefield.place_mine(mine_x, mine_y);

        // Step on spot neighboring mine
        let step_x = 1;
        let step_y = 2;
        let step_result = minefield.step(step_x, step_y);

        // Step was success, and only one spot was revealed
        //     0 1 2
        // 0 [ • • • ]
        // 1 [ • • • ]
        // 2 [ • 1 • ]
        // 3 [ • • • ]
        assert_eq!(step_result, StepResult::Phew);
        assert_eq!(minefield.field[step_x as usize][step_y as usize].state, SpotState::Revealed);
        for (nx, ny) in minefield.neighbors_coords(step_x, step_y) {
            assert_eq!(minefield.field[nx as usize][ny as usize].state, SpotState::Hidden);
        }

        // Step on spot with no neighboring mines
        let step_x = 0;
        let step_y = 1;
        let step_result = minefield.step(step_x, step_y);

        // Step was success, and neighbors were flood revealed
        //     0 1 2
        // 0 [   1 • ]
        // 1 [   1 • ]
        // 2 [ 1 1 • ]
        // 3 [ • • • ]
        assert_eq!(step_result, StepResult::Phew);
        assert_eq!(minefield.field[step_x as usize][step_y as usize].state, SpotState::Revealed);
        for (nx, ny) in minefield.neighbors_coords(step_x, step_y) {
            assert_eq!(minefield.field[nx as usize][ny as usize].state, SpotState::Revealed);
        }

        // Step on mine
        let step_x = 2;
        let step_y = 0;
        let step_result = minefield.step(step_x, step_y);

        // Step was Boom, and only mine spot was newly revealed
        //     0 1 2
        // 0 [   1 ☢ ]
        // 1 [   1 • ]
        // 2 [ 1 1 • ]
        // 3 [ • • • ]
        assert_eq!(step_result, StepResult::Boom);
        assert_eq!(minefield.field[step_x as usize][step_y as usize].state, SpotState::Exploded);
        for (x, y) in minefield.neighbors_coords(step_x,  step_y) {
            let expected_spot_state= if (x, y) == (2, 1) { SpotState::Hidden } else { SpotState::Revealed };
            assert_eq!(minefield.field[x as usize][y as usize].state, expected_spot_state);
        }
     }

     #[test]
     fn flood_reveal() {
        // Create empty bigger minefield
        //     0 1 2 3 4 5 6 7 8 9
        // 0 [     1 ☢ 1           ]
        // 1 [     1 1 1           ]
        // 2 [           1 1 1     ]
        // 3 [   1 1 1   1 ☢ 1 1 1 ]
        // 4 [   1 ☢ 1   1 1 1 1 ☢ ]
        // 5 [   1 1 1         1 1 ]
        // 6 [         1 1 2 1 1   ]
        // 7 [         1 ☢ 2 ☢ 1   ]
        // 8 [         1 1 2 1 1   ]
        // 9 [                     ]
        let width = 10;
        let height = 10;
        let mut minefield = Minefield::new(width, height);

        let mine_coords = [(2, 4), (5, 7), (7, 7), (9, 4), (6, 3), (3, 0)];
        for (x, y) in mine_coords {
            minefield.place_mine(x, y);
        }

        // Place a flag
        //     0 1 2 3 4 5 6 7 8 9
        // 0 [ • • • • • • • • • • ]
        // 1 [ • • • • • ⚐ • • • • ]
        // 2 [ • • • • • • • • • • ]
        // 3 [ • • • • • • • • • • ]
        // 4 [ • • • • • • • • • • ]
        // 5 [ • • • • • • • • • • ]
        // 6 [ • • • • • • • • • • ]
        // 7 [ • • • • • • • • • • ]
        // 8 [ • • • • • • • • • • ]
        // 9 [ • • • • • • • • • • ]
        let flag_x = 5;
        let flag_y = 1;
        minefield.field[flag_x as usize][flag_y as usize].state = SpotState::Flagged;

        // Step on spot (x=9, y=6)
        //     0 1 2 3 4 5 6 7 8 9
        // 0 [     1 • • • • • • • ]
        // 1 [     1 1 1 ⚐ • • • • ]
        // 2 [           1 • • • • ]
        // 3 [   1 1 1   1 • • • • ]
        // 4 [   1 • 1   1 1 1 1 • ]
        // 5 [   1 1 1         1 1 ]
        // 6 [         1 1 2 1 1   ]
        // 7 [         1 • • • 1   ]
        // 8 [         1 1 2 1 1   ]
        // 9 [                     ]
        let step_x = 9;
        let step_y = 6;
        let step_result = minefield.step(step_x, step_y);
        assert_eq!(step_result, StepResult::Phew);

        // All mines are still hidden
        for (x, y) in mine_coords {
            assert_eq!(minefield.field[x as usize][y as usize].state, SpotState::Hidden);
        }

        // Flood revealed the entire maze
        assert_eq!(minefield.field[7][5].state, SpotState::Revealed);

        // Flag is still there
        assert_eq!(minefield.field[flag_x as usize][flag_y as usize].state, SpotState::Flagged);

        // Insulated portion of field is still hidden
        assert_eq!(minefield.field[9][0].state, SpotState::Hidden);
        assert_eq!(minefield.field[7][1].state, SpotState::Hidden);
     }

     #[allow(dead_code)]
     fn print_minefield(minefield: &Minefield) {
        // X axis
        println!();
        print!("   ");
        for y in 0..minefield.width {
            print!(" {}", y);
        }
        println!();

        for y in 0..minefield.height {
            // Y Axis
            print!("{:?} [", y);
            for x in 0..minefield.width {
                match minefield.field[x as usize][y as usize].kind {
                    SpotKind::Mine => {
                        print!(" ☢");
                    },
                    SpotKind::Empty(n) => {
                        if n > 0 {
                            print!(" {}", n);
                        } else {
                            print!("  ");
                        }
                    },
                }
            }
            println!(" ]");
        }
     }

     #[allow(dead_code)]
     fn print_minefield_state(minefield: &Minefield) {
        // X axis
        println!();
        print!("   ");
        for y in 0..minefield.width {
            print!(" {}", y);
        }
        println!();

        for y in 0..minefield.height {
            // Y Axis
            print!("{:?} [", y);
            for x in 0..minefield.width {
                match minefield.field[x as usize][y as usize].state {
                    SpotState::Hidden => {
                        print!(" •");
                    },
                    SpotState::Flagged => {
                        print!(" ⚐");
                    },
                    SpotState::Exploded => {
                        print!(" 💥");
                    }
                    SpotState::Revealed => {
                        match minefield.field[x as usize][y as usize].kind {
                            SpotKind::Mine => {
                                print!(" ☢");
                            },
                            SpotKind::Empty(n) => {
                                if n > 0 {
                                    print!(" {}", n);
                                } else {
                                    print!("  ");
                                }
                            },
                        }
                    },
                }
            }
            println!(" ]");
        }
     }
 }