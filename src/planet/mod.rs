use glium::{Surface,glutin};
use nalgebra_glm as glm;

use crate::graphics;

struct PlanetMesh{
    positions :glium::VertexBuffer<graphics::Vertex>,
    texture_coords :glium::VertexBuffer<graphics::TexCoords>,
    indices: glium::IndexBuffer<u32>,

    world:[[f32; 4]; 4]
}

pub struct Planet{
    mesh: PlanetMesh,
}

impl Planet{
    pub fn new(display:&glium::Display,path_to_lookup: &str,iterations :u8)->Planet{

        let base_shape = 
        graphics::Shape::icosahedron()
        .subdivide(iterations)
        .normalize();

        let verts:Vec::<graphics::Vertex>  = base_shape.vertices.iter()
        .map(|v| graphics::Vertex{
            position: [v.x,v.y,v.z]
        })
        .collect();

        Planet{
            mesh: 
            PlanetMesh{
                //normal buffer, not to be updated regularly
                positions: glium::VertexBuffer::new(display, &verts).unwrap(),
                //is a dynamic buffer because we intend to update the texture coordinates regularly
                //makes all 0.0 as theyll be updated almost immedietly
                texture_coords:glium::VertexBuffer::dynamic(display, 
                    &vec![graphics::TexCoords{tex_coords:[0.5,0.0]};verts.len()])
                    .unwrap(),
                indices: glium::IndexBuffer::new(display,glium::index::PrimitiveType::TrianglesList, &base_shape.indices).unwrap(),

                world: glm::translation(&glm::Vec3::new(0.0,0.0,-5.0)).into()//TEMPORARY, FUTURE ME PLEASE CHANGE THIS
            }
        }
    }

    pub fn draw(&self, target:&mut glium::Frame, program:&glium::Program, params:&glium::DrawParameters,cam:&graphics::Camera){
        let uniform = glium::uniform! {
            perspective: cam.perspective,
            view: cam.view,
            world: self.mesh.world
        };
        target.draw((&self.mesh.positions,&self.mesh.texture_coords),&self.mesh.indices,program,&uniform,params).unwrap();
    }
}