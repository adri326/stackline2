#![feature(iter_intersperse)]

use stackline::prelude::*;
use stackline::tile::*;
use clap::Parser;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::io::Write;

fn main() {
    let args = Args::parse();

    // let mut world = World::new();
    // let mut pane = Pane::empty(4, 4).unwrap();
    // pane.set_tile((0, 0), FullTile::from(Wire::new(Orientation::Any)));
    // world.set_pane(String::from("main"), pane);
    // let raw = serde_json::to_string(&world).unwrap();
    // std::fs::write(&args.file, &raw).unwrap();

    let raw = std::fs::read_to_string(&args.file).expect(&format!("Couldn't open {}!", args.file.display()));

    let mut world: World = serde_json::from_str(&raw).unwrap();

    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        line = line.trim().to_string();
        let mut tokens = line.split(' ');
        match tokens.next() {
            None => continue,
            Some("run") => {
                if let Some(Ok(steps)) = tokens.next().map(|s| s.parse::<usize>()) {
                    run(&mut world, steps).unwrap();
                } else {
                    eprintln!("Syntax error: invalid number of steps");
                }
            }
            Some("step") => {
                step(&mut world);
            }

            Some("print") => {
                print!("{}", world);
            }
            Some("get") => {
                if let (Some(x), Some(y)) = (tokens.next(), tokens.next()) {
                    if let (Ok(x), Ok(y)) = (x.parse(), y.parse()) {
                        get(&world, x, y);
                    }
                } else {
                    eprintln!("Expected two arguments");
                }
            }

            Some("set") => {
                if let (Some(x), Some(y), Some(name)) = (tokens.next(), tokens.next(), tokens.next()) {
                    if let (Ok(x), Ok(y)) = (x.parse(), y.parse()) {
                        set(&mut world, x, y, name);
                    }
                } else {
                    eprintln!("Expected three arguments");
                }
            }
            Some("remove") => {
                if let (Some(x), Some(y)) = (tokens.next(), tokens.next()) {
                    if let (Ok(x), Ok(y)) = (x.parse(), y.parse()) {
                        remove(&mut world, x, y);
                    }
                } else {
                    eprintln!("Expected two arguments");
                }
            }
            Some("prop") => {
                if let (Some(x), Some(y), Some(prop_name)) = (tokens.next(), tokens.next(), tokens.next()) {
                    if let (Ok(x), Ok(y)) = (x.parse(), y.parse()) {
                        prop(&mut world, x, y, prop_name, tokens.intersperse(" ").collect());
                    }
                } else {
                    eprintln!("Expected four arguments");
                }
            }
            Some("state") => {
                if let (Some(x), Some(y), Some(new_state)) = (tokens.next(), tokens.next(), tokens.next()) {
                    if let (Ok(x), Ok(y)) = (x.parse(), y.parse()) {
                        state(&mut world, x, y, new_state);
                    }
                } else {
                    eprintln!("Expected three arguments");
                }
            }

            Some("signal") => {
                if let (Some(x), Some(y)) = (tokens.next(), tokens.next()) {
                    if let (Ok(x), Ok(y)) = (x.parse(), y.parse()) {
                        signal(&mut world, x, y);
                    }
                } else {
                    eprintln!("Expected two arguments");
                }
            }
            Some("clear") => {
                if let (Some(x), Some(y)) = (tokens.next(), tokens.next()) {
                    if let (Ok(x), Ok(y)) = (x.parse(), y.parse()) {
                        clear(&mut world, x, y);
                    }
                } else {
                    eprintln!("Expected two arguments");
                }
            }
            Some("push") => {
                if let (Some(x), Some(y)) = (tokens.next(), tokens.next()) {
                    if let (Ok(x), Ok(y)) = (x.parse(), y.parse()) {
                        push(&mut world, x, y, tokens.intersperse(" ").collect());
                    }
                } else {
                    eprintln!("Expected three arguments");
                }
            }
            Some("pop") => {
                if let (Some(x), Some(y)) = (tokens.next(), tokens.next()) {
                    if let (Ok(x), Ok(y)) = (x.parse(), y.parse()) {
                        pop(&mut world, x, y);
                    }
                } else {
                    eprintln!("Expected two arguments");
                }
            }
            Some("dir") => {
                if let (Some(x), Some(y), Some(direction)) = (tokens.next(), tokens.next(), tokens.next()) {
                    if let (Ok(x), Ok(y)) = (x.parse(), y.parse()) {
                        dir(&mut world, x, y, direction);
                    }
                } else {
                    eprintln!("Expected three arguments");
                }
            }

            Some("load") => {
                if let Some(path) = tokens.next() {
                    load(&mut world, path);
                } else {
                    load(&mut world, &args.file);
                }
            }
            Some("save") => {
                if let Some(path) = tokens.next() {
                    save(&world, path);
                } else {
                    save(&world, &args.file);
                }
            }

            Some("help") => {
                println!("- `print`: prints the current world");
                println!("- `get <x> <y>`: prints the JSON-serialized data of the tile at (x, y)");
                println!("- `set <x> <y> <tilename>`: sets the tile at (x, y) to a default tilename");
                println!("- `remove <x> <y>`: removes the tile at (x, y)");

                println!("- `prop <x> <y> <prop_name> [data]`: sets the property of the tile at (x, y)");
                println!("  if the tile is a single tuple struct, then prop_name is ignored.");
                println!("  if the tile is a tuple struct, then prop_name should be the index of the property");

                println!("- `state <x> <y> <state>`: sets the state at (x, y) to `state`");

                println!("- `signal <x> <y>`: adds an empty signal to the tile at (x, y)");
                println!("- `push <x> <y> <value>`: pushes `value` to the signal at (x, y)");
                println!("- `pop <x> <y>`: pops a value from the signal at (x, y)");
                println!("- `clear <x> <y>`: clears the signal of the tile at (x, y)");
                println!("- `dir <x> <y> <dir>`: sets the direction of the signal at (x, y)");

                println!("- `run <steps>`: runs a number of steps");
                println!("- `step`: runs a single step");
                println!("- `load [file]`: saves the current state to `file` (defaults to the path in the parameters)");
                println!("- `save [file]`: saves the current state to `file` (defaults to the path in the parameters)");
            }
            Some(cmd) => {
                eprintln!("Syntax error: unknown command {}", cmd);
            }
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, value_parser)]
    file: PathBuf,
}

fn run(world: &mut World, steps: usize) -> std::io::Result<()> {
    let mut stdout = std::io::stdout();
    let mut first = true;
    for _ in 0..steps {
        if !first {
            world.step();
            write!(stdout, "\x1b[4;A")?;
        }
        first = false;
        write!(stdout, "{}", world)?;
        stdout.flush()?;
        std::thread::sleep(Duration::new(0, 100_000_000));
    }
    Ok(())
}

fn step(world: &mut World) {
    world.step();
    print!("{}", world);
}

fn get(world: &World, x: i32, y: i32) {
    match world.get((x, y)) {
        Some(tile) => {
            match serde_json::to_string_pretty(&*tile) {
                Ok(serialized) => println!("{}", serialized),
                Err(err) => eprintln!("Error while serializing tile at {}:{}; {}", x, y, err),
            }
        }
        None => {
            eprintln!("No tile at {}:{}!", x, y);
        }
    }
}

fn prop(world: &mut World, x: i32, y: i32, prop_name: &str, value: String) {
    use serde_json::Value;

    let tile = match world.get_mut((x, y)) {
        Some(tile) => {
            if let Some(tile) = tile.get_mut() {
                tile
            } else {
                eprintln!("Tile at {}:{} is empty!", x, y);
                return
            }
        }
        None => {
            eprintln!("No tile at {}:{}!", x, y);
            return;
        }
    };

    let mut tile_value = match serde_json::to_value(&tile) {
        Ok(serialized) => serialized,
        Err(err) => {
            eprintln!("Error while serializing tile at {}:{}; {}", x, y, err);
            return
        }
    };

    let parsed: Value = match serde_json::from_str(&value) {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("Error while parsing value: {}", err);
            return
        }
    };


    if let Value::Object(ref mut enum_map) = &mut tile_value {
        if let Some(enum_value) = enum_map.values_mut().next() {
            match enum_value {
                Value::Object(map) => {
                    map.insert(prop_name.to_string(), parsed);
                }
                Value::Array(vec) => {
                    if let Ok(num) = prop_name.parse::<usize>() {
                        if num >= vec.len() {
                            eprintln!("Index out of bound: len is {} but index is {}", vec.len(), num);
                            return
                        }
                        vec[num] = parsed;
                    }
                }
                _ => {
                    *enum_value = parsed;
                }
            }
        } else {
            eprintln!("Format error: expected enum to be encoded as a single-element map.");
            return
        }
    } else {
        eprintln!("Format error: expected enum to be encoded as a single-element map.");
        return
    }

    *tile = match serde_json::from_value(tile_value) {
        Ok(tile) => tile,
        Err(err) => {
            eprintln!("Error while inserting value: {}", err);
            return
        }
    };
}

fn set(world: &mut World, x: i32, y: i32, name: &str) {
    let tile = match world.get_mut((x, y)) {
        Some(tile) => tile,
        None => {
            eprintln!("No tile at {}:{}!", x, y);
            return;
        }
    };

    *tile = match AnyTile::new(name) {
        Some(tile) => FullTile::from(tile),
        None => {
            eprintln!("No tile named {}", name);
            return;
        }
    };
}

fn remove(world: &mut World, x: i32, y: i32) {
    match world.get_mut((x, y)) {
        Some(tile) => *tile = FullTile::new(None),
        None => {
            eprintln!("No tile at {}:{}!", x, y);
            return;
        }
    }
}

fn signal(world: &mut World, x: i32, y: i32) {
    match world.get_mut_with_pos((x, y)) {
        Some((tile, x, y)) => {
            tile.set_signal(Some(Signal::empty((x, y), Direction::Right)));
            tile.set_state(State::Active);
        }
        None => {
            eprintln!("No tile at {}:{}!", x, y);
            return;
        }
    }
}

fn clear(world: &mut World, x: i32, y: i32) {
    match world.get_mut((x, y)) {
        Some(tile) => {
            tile.set_signal(None);
            tile.set_state(State::Idle);
        }
        None => {
            eprintln!("No tile at {}:{}!", x, y);
            return;
        }
    }
}

fn push(world: &mut World, x: i32, y: i32, value: String) {
    use serde_json::Value as JValue;

    let value: JValue = match serde_json::from_str(&value) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Error while parsing value: {}", err);
            return
        }
    };

    let value: Value = match value {
        JValue::Number(num) => {
            if let Some(f) = num.as_f64() {
                Value::Number(f)
            } else {
                eprintln!("Unsupported value: {:?}", num);
                return
            }
        }
        JValue::String(s) => Value::String(s),
        x => {
            eprintln!("Unsupported value: {:?}", x);
            return
        }
    };

    match world.get_mut((x, y)) {
        Some(tile) => {
            match tile.take_signal() {
                Some(mut signal) => {
                    signal.push(value);
                    tile.set_signal(Some(signal));
                }
                None => {
                    eprintln!("No signal at {}:{}!", x, y);
                }
            }
        }
        None => {
            eprintln!("No tile at {}:{}!", x, y);
            return;
        }
    }
}

fn pop(world: &mut World, x: i32, y: i32) {
    match world.get_mut((x, y)) {
        Some(tile) => {
            match tile.take_signal() {
                Some(mut signal) => {
                    let popped = signal.pop();
                    tile.set_signal(Some(signal));

                    if let Some(popped) = popped {
                        match serde_json::to_string_pretty(&popped) {
                            Ok(pretty) => println!("{}", pretty),
                            Err(err) => {
                                eprintln!("Error while printing popped value: {}", err);
                            }
                        }
                    } else {
                        eprintln!("Nothing to pop at {}:{}!", x, y);
                    }
                }
                None => {
                    eprintln!("No signal at {}:{}!", x, y);
                }
            }
        }
        None => {
            eprintln!("No tile at {}:{}!", x, y);
            return;
        }
    }
}

fn dir(world: &mut World, x: i32, y: i32, direction: &str) {
    let direction: Direction = match serde_json::from_str(direction) {
        Ok(direction) => direction,
        Err(err) => {
            eprintln!("Error while parsing direction: {}", err);
            return
        }
    };

    match world.get_mut((x, y)) {
        Some(tile) => {
            match tile.take_signal() {
                Some(signal) => {
                    tile.set_signal(Some(signal.moved(direction)));
                }
                None => {
                    eprintln!("No signal at {}:{}!", x, y);
                }
            }
        }
        None => {
            eprintln!("No tile at {}:{}!", x, y);
            return;
        }
    }
}

fn state(world: &mut World, x: i32, y: i32, state: &str) {
    let state: State = match serde_json::from_str(state) {
        Ok(state) => state,
        Err(err) => {
            eprintln!("Error while parsing state: {}", err);
            return
        }
    };

    match world.get_mut((x, y)) {
        Some(tile) => {
            tile.set_state(state);
        }
        None => {
            eprintln!("No tile at {}:{}!", x, y);
            return;
        }
    }
}

fn save(world: &World, path: impl AsRef<Path>) {
    match serde_json::to_string(world) {
        Ok(raw) => {
            std::fs::write(path.as_ref(), &raw).unwrap_or_else(|err| {
                eprintln!("Error while saving: {}", err);
            });
        }
        Err(err) => {
            eprintln!("Error while converting world to JSON: {}", err);
        }
    }
}

fn load(world: &mut World, path: impl AsRef<Path>) {
    match std::fs::read_to_string(path.as_ref()) {
        Ok(string) => {
            match serde_json::from_str(&string) {
                Ok(parsed) => {
                    *world = parsed;
                }
                Err(err) => {
                    eprintln!("Error while parsing file: {}", err);
                }
            }
        }
        Err(err) => {
            eprintln!("Error while reading file: {}", err);
        }
    }
}
