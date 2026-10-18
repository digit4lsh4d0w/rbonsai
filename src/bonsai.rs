use crossterm::{
    cursor::MoveTo,
    execute, queue,
    style::{Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear},
};
use rand::{RngExt, rngs::SmallRng};
use std::{
    io::{Write, stdout},
    time::Duration,
};

use crate::{
    Config,
    base::draw_base,
    utility::{Style, choose_color, choose_string, set_deltas, wait_for_key_press},
};

/// The different types of each symbol
#[derive(Clone, Copy)]
pub enum BranchType {
    /// The main trunk of the tree
    Trunk,
    /// A branch that grows to the left
    ShootLeft,
    /// A branch that grows to the right
    ShootRight,
    /// A branch that is dying
    Dying,
    /// A branch that is dead
    Dead,
}

impl BranchType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Trunk => "Trunk",
            Self::ShootLeft => "ShootLeft",
            Self::ShootRight => "ShootRight",
            Self::Dying => "Dying",
            Self::Dead => "Dead",
        }
    }
}

struct Counters {
    shoots: i32,
    shoot_counter: i32,
    tree_bottom: u16,
    max_x: u16,
    max_y: u16,
}

pub struct Val {
    pub style: Style,
    pub char: &'static str,
    pub pos: Position,
    pub dx: i32,
    pub dy: i32,
    pub life: i32,
    pub branch_type: BranchType,
    pub shoots: i32,
    pub shoot_cooldown: i32,
}

#[derive(Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

fn in_bounds(pos: Position, max_x: u16, max_y: u16) -> bool {
    u16::try_from(pos.x).is_ok_and(|x| x < max_x) && u16::try_from(pos.y).is_ok_and(|y| y < max_y)
}

pub fn grow_tree(config: &Config, rng: &mut SmallRng) -> Vec<Val> {
    let (max_x, max_y) = terminal::size().unwrap();
    grow_tree_at(config, rng, max_x, max_y, config.verbose)
}

pub fn grow_tree_at(
    config: &Config,
    rng: &mut SmallRng,
    max_x: u16,
    max_y: u16,
    show_debug: bool,
) -> Vec<Val> {
    let tree_bottom = match config.base {
        1 => max_y.saturating_sub(5),
        2 => max_y.saturating_sub(4),
        _ => max_y,
    };

    let mut counters = Counters {
        shoots: 0,
        // Initialize shoot counter to a random value
        shoot_counter: (rng.random::<i32>() % 3) + 1,
        tree_bottom,
        max_x,
        max_y,
    };

    if show_debug {
        let mut stdout = stdout();
        execute!(
            stdout,
            MoveTo(5, 2),
            Print(format!("maxX: {max_x:03}, maxY: {tree_bottom:03}")),
        )
        .unwrap();
    }

    let mut tree = Vec::new();

    branch(
        config,
        &mut counters,
        &mut tree,
        rng,
        Position {
            x: i32::from(max_x / 2),
            y: i32::from(tree_bottom),
        },
        BranchType::Trunk,
        config.life,
    );

    tree
}

fn branch(
    config: &Config,
    counters: &mut Counters,
    tree: &mut Vec<Val>,
    rng: &mut SmallRng,
    mut pos: Position,
    branch_type: BranchType,
    mut life: i32,
) {
    let mut shoot_cooldown = config.multiplier;
    let (max_x, max_y) = (counters.max_x, counters.max_y);

    while life > 0 {
        life -= 1;
        let age = config.life - life;

        let (dx, mut dy) = set_deltas(branch_type, life, age, config.multiplier, rng);

        if dy > 0 && pos.y >= i32::from(counters.tree_bottom) {
            dy -= 1;
        }

        if !in_bounds(pos, max_x, max_y) {
            break;
        }

        if life < 3 {
            branch(config, counters, tree, rng, pos, BranchType::Dead, life);
        } else {
            match branch_type {
                BranchType::Trunk | BranchType::ShootLeft | BranchType::ShootRight
                    if life < config.multiplier.saturating_add(2) =>
                {
                    branch(config, counters, tree, rng, pos, BranchType::Dying, life);
                }
                BranchType::Trunk
                    if rng.random_range(0..3) == 0 || life % config.multiplier.max(1) == 0 =>
                {
                    if rng.random_range(0..8) == 0 && life > 7 {
                        shoot_cooldown = config.multiplier.saturating_mul(2);
                        let random_life = life.saturating_add(rng.random_range(-2..3));
                        branch(
                            config,
                            counters,
                            tree,
                            rng,
                            pos,
                            BranchType::Trunk,
                            random_life,
                        );
                    } else if shoot_cooldown <= 0 {
                        shoot_cooldown = config.multiplier.saturating_mul(2);
                        counters.shoots += 1;
                        counters.shoot_counter += 1;
                        let shoot_direction = if counters.shoot_counter % 2 == 0 {
                            BranchType::ShootLeft
                        } else {
                            BranchType::ShootRight
                        };
                        let shoot_life = life.saturating_add(config.multiplier);
                        branch(
                            config,
                            counters,
                            tree,
                            rng,
                            pos,
                            shoot_direction,
                            shoot_life,
                        );
                    }
                }
                _ => {}
            }
        }

        shoot_cooldown -= 1;
        pos.x += dx;
        pos.y += dy;

        if !in_bounds(pos, max_x, max_y) {
            continue;
        }

        tree.push(Val {
            pos,
            style: choose_color(branch_type, rng),
            char: choose_string(branch_type, life, dx, dy),
            branch_type,
            dx,
            dy,
            life,
            shoots: counters.shoots,
            shoot_cooldown,
        });
    }
}

// returns true if the tree finished drawing. Returns false if it didn't and
// the user chose to exit early
pub fn draw_tree(config: &Config, tree: &[Val]) -> bool {
    let mut stdout = stdout();
    let delay = Duration::from_secs_f64(config.time.max(0.0));

    for val in tree {
        if config.verbose {
            queue!(
                stdout,
                MoveTo(5, 3),
                Print(format!("life: {}", val.life)),
                MoveTo(5, 4),
                Print(format!("shoots: {:02}", val.shoots)),
                MoveTo(5, 5),
                Print(format!("dx: {:02}", val.dx)),
                MoveTo(5, 6),
                Print(format!("dy: {:02}", val.dy)),
                MoveTo(5, 7),
                Print(format!("type: {}", val.branch_type.as_str())),
                MoveTo(5, 8),
                Print(format!("shootCooldown: {:3}", val.shoot_cooldown)),
            )
            .unwrap();
            stdout.flush().unwrap();
        }

        queue!(
            stdout,
            SetAttribute(val.style.attribute),
            SetForegroundColor(val.style.foreground_color),
            SetBackgroundColor(val.style.background_color),
            MoveTo(val.pos.x as u16, val.pos.y as u16),
            Print(val.char),
        )
        .unwrap();

        if config.live {
            stdout.flush().unwrap();
            if wait_for_key_press(delay) {
                let _ = queue!(stdout, ResetColor);
                let _ = stdout.flush();
                return false;
            }
        }
    }

    queue!(stdout, ResetColor).unwrap();
    stdout.flush().unwrap();
    true
}

pub fn init(args: &Config) {
    let mut stdout = stdout();
    execute!(stdout, Clear(terminal::ClearType::All)).unwrap();
    draw_base(args);
}
