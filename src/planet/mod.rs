use glium::{Surface,glutin};
use nalgebra_glm as glm;
use rand::Rng;

use crate::graphics;

struct PlanetMesh{
    vertices :glium::VertexBuffer<graphics::PosNormTex>,
    indices: glium::IndexBuffer<u32>,

    to_light: [f32; 3],//vec pointing to light
    world:[[f32; 4]; 4]//NOTE DO NOT DO THIS YOU FOOL
}

pub struct Planet{
    texture: glium::texture::SrgbTexture2d,
    mesh: PlanetMesh,
}
impl Planet{
    pub fn new(display:&glium::Display,texture:glium::texture::SrgbTexture2d,iterations :u8)->Planet{

        let base_shape = graphics::Shape::icosahedron()
            .subdivide(iterations)
            .normalize();

        let planet_vertices:Vec::<graphics::PosNormTex> = base_shape.vertices
            .iter()
            .map(|v| graphics::PosNormTex{
                position:[v.x,v.y,v.z],
                normal:[v.x,v.y,v.z],
                tex_coords:[0.75,0.5]
             })
            .collect();


        Planet{
            texture,

            mesh: 
            PlanetMesh{
                vertices: glium::VertexBuffer::new(display,&planet_vertices).unwrap(),
                indices: glium::IndexBuffer::new(display,glium::index::PrimitiveType::TrianglesList, &base_shape.indices).unwrap(),

                to_light: [0.707113562438,0.0,0.707113562438],
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