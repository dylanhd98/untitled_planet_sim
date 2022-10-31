use glium::{Surface,glutin};
use nalgebra_glm as glm;
use noise::{NoiseFn, Perlin, Seedable};//NOTE FOR FUTURE ME REMOVE THIS PLEASE ITS ANNOYING AND I DO NOT LIKE

use crate::graphics;

struct PlanetMesh{
    positions :glium::VertexBuffer<graphics::Vertex>,
    texture_coords :glium::VertexBuffer<graphics::TexCoords>,
    indices: glium::IndexBuffer<u32>,

    world:[[f32; 4]; 4]
}

pub struct Planet{
    texture: glium::texture::SrgbTexture2d,
    mesh: PlanetMesh,
}

impl Planet{
    pub fn new(display:&glium::Display,texture:glium::texture::SrgbTexture2d,iterations :u8)->Planet{
        let perlin = Perlin::new(1);
        let scale = 2.0;

        let base_shape = 
        graphics::Shape::icosahedron()
        .subdivide(iterations)
        .normalize();

        let verts:Vec::<graphics::Vertex>  = base_shape.vertices.iter()
        .map(|v| graphics::Vertex{
            position: [v.x,v.y,v.z]
        })
        .collect();

        let mut coords  =vec![graphics::TexCoords{tex_coords:[0.5,0.0]};verts.len()];
        for i in 0..coords.len(){
            let pos:Vec<f64> = verts[i].position
                .into_iter()
                .map(|c| c as f64)
                .collect();

            coords[i].tex_coords = {
                let x =perlin.get([pos[0]*scale,pos[1]*scale,pos[2]*scale]);
                let y =perlin.get([pos[0]*scale,(pos[1]+500.0)*scale,pos[2]*scale]);
                [x as f32 ,y as f32] 
            }
            //coord.tex_coords = [perlin.get[]rng.gen()];
        }

        Planet{
            texture,

            mesh: 
            PlanetMesh{
                //normal buffer, not to be updated regularly
                positions: glium::VertexBuffer::new(display, &verts).unwrap(),
                //is a dynamic buffer because we intend to update the texture coordinates regularly
                //makes all 0.0 as theyll be updated almost immedietly
                texture_coords:glium::VertexBuffer::dynamic(display, 
                    &coords)
                    .unwrap(),
                indices: glium::IndexBuffer::new(display,glium::index::PrimitiveType::TrianglesList, &base_shape.indices).unwrap(),

                world: glm::translation(&glm::Vec3::new(0.0,0.0,0.0)).into()//TEMPORARY, FUTURE ME PLEASE CHANGE THIS
            }
        }
    }

    pub fn draw(&self, target:&mut glium::Frame, program:&glium::Program, params:&glium::DrawParameters,cam:&graphics::Camera){
        let uniform = glium::uniform! {
            perspective: cam.perspective,
            view: cam.view,
            world: self.mesh.world,
            tex: &self.texture
        };
        target.draw((&self.mesh.positions,&self.mesh.texture_coords),&self.mesh.indices,program,&uniform,params).unwrap();
    }
}