use bevy::{math::Vec3, prelude::*};
use noise::{NoiseFn, OpenSimplex, Seedable};
use rand::{SeedableRng, Rng};

#[derive(Clone, Copy)]
pub struct ProcGen {
    seed: u32,
    map_size: usize,
    simplex: OpenSimplex
}

impl ProcGen {
    pub fn new(seed: u32, map_size: usize) -> Self {
        Self {
            seed,
            map_size,
            simplex: OpenSimplex::new().set_seed(seed)
        }
    }

    pub fn noise(&self, coords: Vec3) -> f64 {
        self.simplex.get([coords.x.into(), coords.z.into()])
    }

    pub fn fbm(
        &self,
        octaves: i32,
        pos: Vec3,
    ) -> f32 {
        let mut pos = pos.clone();
        let mut value = 0.0;
        let mut amplitude = 0.5;

        for _octave in 0..octaves {
            value += amplitude * self.noise(pos);
            pos *= 2.;
            amplitude *= 0.5;
        }

        value as f32
    }

    pub fn gen_noise_map__(&self, map_position: Vec3) -> Vec<f32> {
        let mut height_map = vec![0.; self.map_size.pow(2)];
        
        for z in 0..self.map_size {
            for x in 0..self.map_size {
                let block_x = (x as f32 + map_position.x * self.map_size as f32) as f64;
                let block_z = (z as f32 + map_position.z * self.map_size as f32) as f64;

                let mut value = self.fbm(6, Vec3::new(block_x as f32, 1., block_z as f32));
                //let mut value = noise.get_noise(block_x / chunk::CHUNK_SIZE as f32, block_z / chunk::CHUNK_SIZE as f32);
            
                //value = (value + 1.0) / 2.0;
                value *= 32.0;

                height_map[z * self.map_size + x] = value as f32;
            }
        }
    
        height_map
    }

    pub fn gen_noise_map( &self, map_position: Vec3) -> Vec<f32> {
        let mut height_map = vec![0.; self.map_size.pow(2)];
        
        for z in 0..self.map_size {
            for x in 0..self.map_size {
                let block_x = (x as f32 + map_position.x * self.map_size as f32) as f64;
                let block_z = (z as f32 + map_position.z * self.map_size as f32) as f64;

                let mut value = self.simplex.get([block_x / self.map_size as f64, block_z / self.map_size as f64]);
                //let mut value = noise.get_noise(block_x / chunk::CHUNK_SIZE as f32, block_z / chunk::CHUNK_SIZE as f32);
            
                value = (value + 1.0) / 2.0;
                value *= 16.0;

                height_map[z * self.map_size + x] = value as f32;
            }
        }
    
        height_map
    }

    pub fn gen_noise_map_old(
        &self,
        map_position: Vec3,
        mut scale: f64,
        mut octaves: i32,
        mut persistence: f32,
        mut lacunarity: f32,
        offset: Vec2,
    ) -> Vec<f32> {
        // validation for values that should never be below a certain value
        if octaves < 0 {
            octaves = 0;
        }

        if lacunarity < 1. {
            lacunarity = 1.;
        }

        if scale <= 0. {
            scale = 0.0001;
        }

        if persistence < 0. {
            persistence = 0.;
        } else if persistence > 1. {
            persistence = 1.;
        }

        let mut noise_map = vec![0.; self.map_size.pow(2)];
        let mut prng = rand_chacha::ChaChaRng::seed_from_u64(self.seed as u64);
        
        let mut octave_offsets = Vec::<Vec2>::new();
        for _ in 0..octaves {
            let offset_x = prng.gen_range(-100_000_f32..100_000_f32) + offset.x;
            let offset_y = prng.gen_range(-100_000_f32..100_000_f32) + offset.y;

            octave_offsets.push(Vec2 { x: offset_x, y: offset_y });
        }

        let mut min_noise_height = f64::MAX;
        let mut max_noise_height = f64::MIN;

        //let half = self.map_size as f32 / 2.;

        for z in 0..self.map_size {
            for x in 0..self.map_size {
                let block_x = ((x as f32) + map_position.x * self.map_size as f32) as f64;
                //let block_y = (y as f32 + map_position.y * map_size as f32) as f64;
                let block_z = ((z as f32) + map_position.z * self.map_size as f32) as f64;

                let mut amplitude = 1.;
                let mut frequency = 1.;
                let mut noise_height = 0.;

                for octave in 0..octaves {
                    let octave_offset = octave_offsets[octave as usize];
                    let sample_x = (block_x / scale) * frequency + octave_offset.x as f64;
                    let sample_z = (block_z / scale) * frequency + octave_offset.y as f64;

                    let simplex_val = self.simplex.get([sample_x, sample_z]) * 2. - 1.;
                    noise_height += simplex_val * amplitude;

                    amplitude *= persistence as f64;
                    frequency *= lacunarity as f64;
                }

                if noise_height > max_noise_height {
                    max_noise_height = noise_height;
                } else if noise_height < min_noise_height {
                    min_noise_height = noise_height;
                }
                
                noise_map[z * self.map_size + x] = noise_height as f32;
            }
        }

        for z in 0..self.map_size {
            for x in 0..self.map_size {
                let inverse_lerped = inverse_lerp(
                    min_noise_height as f32,
                    max_noise_height as f32,
                    noise_map[z * self.map_size + x],
                );

                //print!("({x}, {inverse_lerped}, {z}) ");

                noise_map[z * self.map_size + x] = inverse_lerped;
            }
        }

        //println!("");

        noise_map
    }
}

pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let col_a_f32 = a.as_rgba_f32();
    let col_b_f32 = b.as_rgba_f32();

    Color::Rgba {
        red: lerp(col_a_f32[0], col_b_f32[0], t),
        green: lerp(col_a_f32[1], col_b_f32[1], t),
        blue: lerp(col_a_f32[2], col_b_f32[2], t),
        alpha: lerp(col_a_f32[3], col_b_f32[3], t)
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn inverse_lerp(a: f32, b: f32, v: f32) -> f32 {
    (v - a) / (b - a)
}