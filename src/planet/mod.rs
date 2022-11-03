use glium::{Surface,glutin};
use nalgebra_glm as glm;
use rand::Rng;
use noise::{NoiseFn, Perlin, Seedable};

use crate::graphics;

struct PlanetMesh{
    vertices :glium::VertexBuffer<graphics::PosNormTex>,
    indices: glium::IndexBuffer<u32>,

    to_light: [f32; 3],//vec pointing to light
    world:[[f32; 4]; 4]//NOTE DO NOT DO THIS YOU FOOL
}

//FURURE ME
fn octive_noise(perlin: Perlin, pos:[f32;3], scale:f32, octives:u8, persistance:f32, lacunarity:f32)->f32{
    let mut noise_value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;

    for o in 0..octives{
        let perlin_value = perlin.get([
            (pos[0]/scale * frequency) as f64,
            (pos[1]/scale *frequency) as f64,
            (pos[2]/scale *frequency) as f64
        ]) as f32;

        noise_value += perlin_value*amplitude;
        amplitude *= persistance;
        frequency *= lacunarity;
    }
    noise_value
}

//instead of having each surface cell be a struct, each part of the cells are stored in there own collection, making accessing memory quicker
//this is vital for extra proformance
struct PlanetSurface{
    height:Vec<f32>,
    temperature:Vec<f32>,
    humidity:Vec<f32>
}
impl PlanetSurface{
    fn new(cell_count:usize)->PlanetSurface{
        PlanetSurface{
            height: vec![0.0;cell_count],
            temperature: vec![0.0;cell_count],
            humidity: vec![0.0;cell_count]
        }
    }
    fn step(){

    }
}

pub struct Planet{
    texture: glium::texture::SrgbTexture2d,
    mesh: PlanetMesh,
}
impl Planet{
    pub fn new(display:&glium::Display, texture:glium::texture::SrgbTexture2d, iterations :u8)->Planet{
        let perlin = Perlin::new(1);

        let base_shape = graphics::Shape::icosahedron()
            .subdivide(iterations)
            .normalize();

        let planet_vertices:Vec::<graphics::PosNormTex> = base_shape.vertices
            .iter()
            .map(|v| graphics::PosNormTex{
                position:[v.x,v.y,v.z],
                normal: [v.x,v.y,v.z],
                tex_coords:[(octive_noise(perlin,[v.x,v.y,v.z], 1.3, 10, 0.75, 1.75)+1.0)/2.0, 0.5]
             })
            .collect();

        Planet{
            texture,

            mesh: 
            PlanetMesh{
                vertices: glium::VertexBuffer::dynamic(display,&planet_vertices).unwrap(),
                indices: glium::IndexBuffer::new(display,glium::index::PrimitiveType::TrianglesList, &base_shape.indices).unwrap(),

                to_light: [0.707113562438,0.0,0.707113562438],//TEMP
                world: glm::translation(&glm::Vec3::new(0.0,0.0,0.0)).into()//TEMPORARY, FUTURE ME PLEASE CHANGE THIS
            }
        }
    }

    pub fn draw(&self, target:&mut glium::Frame, program:&glium::Program, params:&glium::DrawParameters,cam:&graphics::Camera){
        let uniform = glium::uniform! {
            perspective: cam.perspective,
            view: cam.view,
            world: self.mesh.world,

            tex: &self.texture,
            to_light: self.mesh.to_light
        };

        target.draw(&self.mesh.vertices,&self.mesh.indices,program,&uniform,params).unwrap();
    }
}