use kiss3d::{light::Light, window::Window};
use nalgebra::{Point3, Point2, Vector3, UnitQuaternion};
use kiss3d::text::Font;
use anyhow::Result;
use rand::prelude::*;


// window settings
const DIM_X: usize = 35;
const DIM_Y: usize = 35;
const DIM_Z: usize = 35;
const WINDOW_TITLE: &str = "WaTor3D";
const FRAC: f32 = 100.;

// logic settings
const SIMULATION_MAX_SPEED: u64 = 60;
const SHARK_STARVE_RATE: i32 = 4; // this is interesting
const SHARK_REPRODUCE_RATE: i32 = 3;
const FISH_REPRODUCE_RATE: i32 = 3;
const WORLD_GEN_RNG_MAX: i32 = 10;

type World = [[[Entity; DIM_X as usize]; DIM_Y as usize]; DIM_Z as usize];

fn main() -> Result<()> {
    println!("Water Toroid Simulation by Skipper");

    // declarations
    let mut window = Window::new(WINDOW_TITLE);
    let mut frame_count = 0;
    let mut rng: ThreadRng = thread_rng();
    let mut map: World = [[[Entity::new(Lifeform::Empty, 0); DIM_X]; DIM_Y]; DIM_Z];
    let mut cube = window.add_cube(DIM_X as f32 * 1.2 / FRAC, DIM_Y as f32 * 1.2 / FRAC, DIM_Z as f32 * 1.2 / FRAC);
    let cube_rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.003);
    
    // setup
    cube.set_surface_rendering_activation(false);
    cube.set_lines_width(1.);
    window.set_framerate_limit(Some(SIMULATION_MAX_SPEED));
    window.set_light(Light::StickToCamera);
    scramble_map(&mut map, &mut rng);
    
    while window.render() {
        window.draw_text(format!("Iterations: {}", frame_count).as_str(),
        &Point2::new(10.,10.,),
        100.,
        &Font::default(),
        &Point3::new(1.,1.,1.));

        cube.prepend_to_local_rotation(&cube_rot);

        update(&mut map, &mut rng);
        draw(&mut map, &mut window);

        frame_count += 1;
    }
    Ok(())
}

fn draw( world: &mut World, window: &mut Window ) {
    let color = Point3::new(1.,1.,1.);
    let color_s = Point3::new(1.,0.,0.);
    let color_f = Point3::new(0.,0.,1.);
    let xs = DIM_X as f32 / 2. / FRAC * -1.;
    let ys = DIM_Y as f32 / 2. / FRAC * -1.;
    let zs = DIM_Z as f32 / 2. / FRAC * -1.;
    for z in 0..DIM_Z {
        for y in 0..DIM_Y {
            for x in 0..DIM_X {
                match world[z][y][x].form {
                    //Lifeform::Empty => window.draw_point(&Point3::new(x as f32, y as f32, z as f32), &color),
                    Lifeform::Empty => (),
                    Lifeform::Fish => window.draw_point(
                        &Point3::new(x as f32/FRAC + xs, y as f32 /FRAC + ys, z as f32 /FRAC + zs), &color_f),
                    Lifeform::Shark => window.draw_point(
                        &Point3::new(x as f32 / FRAC + xs, y as f32 / FRAC + ys, z as f32 / FRAC + zs), &color_s),
                }
            }
        }
    }
}

fn scramble_map( world: &mut World, rng: &mut ThreadRng ) {
    for dimension in world.iter_mut() {
        for row in dimension.iter_mut() {
            for item in row.iter_mut() {
                let r = rng.gen_range(0..WORLD_GEN_RNG_MAX);
                *item = match r {
                    0 => Entity::new(Lifeform::Fish, 0),
                    1 => Entity::new(Lifeform::Shark, SHARK_STARVE_RATE),
                    _ => Entity::new(Lifeform::Empty, 0),
                }
            }
        }
    }
}

fn update(map: &mut World, rng: &mut ThreadRng) {
    for z in 0..DIM_Z {
        for y in 0..DIM_Y {
            for x in 0..DIM_X {

                //generate random direction
                let rand = rng.gen_range(1..7);
                let mut y1: usize = if rand == 1 {y+1} else if rand == 2 && y > 0 {y-1} else {y};
                let mut x1: usize = if rand == 3 {x+1} else if rand == 4 && x>0 {x-1} else {x};
                let mut z1: usize = if rand == 5 {z+1} else if rand == 6 && z>0 {z-1} else {z};
                y1 = if y1 >= DIM_Y {y1-1} else {y1};
                x1 = if x1 >= DIM_X {x1-1} else {x1};
                z1 = if z1 >= DIM_Z {z1-1} else {z1};
                
                let mut entity = map[z][y][x];
                match entity.form {
                    Lifeform::Shark => {
                        entity.value2 += 1;
                        if let Lifeform::Empty = map[z1][y1][x1].form {
                            entity.value -= 1;
                            map[z1][y1][x1] = entity; //move current entity
                            map[z][y][x] = Entity::new(Lifeform::Empty, 0); //clear currrent field
                        }
                        else if let Lifeform::Fish = map[z1][y1][x1].form {
                            entity.value += 1;
                            map[z1][y1][x1] = entity; //move current entity
                            map[z][y][x] = Entity::new(Lifeform::Empty, 0); //clear currrent field
                        }
                        else {
                            entity.value -= 1;
                            map[z][y][x] = entity;
                        }
                        if entity.value <= 0 {
                            map[z][y][x] = Entity::new(Lifeform::Empty, 0);
                            continue;
                        }
                        if entity.value2 >= SHARK_REPRODUCE_RATE {
                            entity.value2 = 0;
                            map[z][y][x] = entity;
                            map[z1][y1][x1] = entity;
                        }
                    },
                    Lifeform::Fish => {
                        entity.value += 1;
                        if entity.value >= FISH_REPRODUCE_RATE {
                            entity.value = 0;
                            map[z][y][x] = entity;
                            if let Lifeform::Empty = map[z1][y1][x1].form {map[z1][y1][x1] = entity;}
                            continue;
                        }
                        if let Lifeform::Empty = map[z1][y1][x1].form {
                            map[z1][y1][x1] = entity; //move current entity
                            map[z][y][x] = Entity::new(Lifeform::Empty, 0); //clear currrent field
                        }
                        else {
                            map[z][y][x] = entity;
                        }
                    },
                    Lifeform::Empty => (),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Lifeform {
    Shark,
    Fish,
    Empty
}

#[derive(Debug, Clone, Copy)]
struct Entity {
    value: i32,
    value2: i32,
    form: Lifeform
}
impl Entity {
    fn new( form: Lifeform, value: i32 ) -> Self {
        Entity {
            value,
            value2: 0,
            form
        }
    }
}
