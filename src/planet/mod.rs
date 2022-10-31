use glium::{Surface,glutin};
use crate::graphics;

struct PlanetMesh{
    positions :glium::VertexBuffer<graphics::Vertex>,
    texture_coords :glium::VertexBuffer<graphics::TexCoords>,
    indices: glium::IndexBuffer<u32>,
}

pub struct Planet{
    mesh: PlanetMesh,
}

impl Planet{
    pub fn new(display:&glium::Display,iterations : u8)->Planet{
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
            mesh: PlanetMesh{
                positions: glium::VertexBuffer::new(display, &verts).unwrap(),
                //is a dynamic buffer because we intend to update the texture coordinates regularly
                texture_coords:glium::VertexBuffer::dynamic(display, 
                    &vec![graphics::TexCoords{tex_coords:[0.0,0.0]};verts.len()])//makes all 0.0 as theyll be updated almost immedietly
                    .unwrap(),
                indices: glium::IndexBuffer::new(display,glium::index::PrimitiveType::TrianglesList, &base_shape.indices).unwrap()
            }
        }
    }

    pub fn draw(target:&glium::Display, program:&glium::Program, params:&glium::DrawParameters,){
        let uniform = glium::uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [ 0.9 , 0.0, 0.0, 1.0f32],
            ],
        };
    }
}