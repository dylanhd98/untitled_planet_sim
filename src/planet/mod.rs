//external crates
use glium::{Surface,glutin};
use nalgebra_glm as glm;

//other internal modules
use crate::graphics;

//child modules
mod surface;

//struct containing all thigs needed for rendering
struct PlanetBuffers{
    shape_data :glium::VertexBuffer<graphics::VertexPos>,
    planet_data: glium::VertexBuffer<surface::CellData>,
    indices: glium::IndexBuffer<u32>,
}


pub struct Planet{
    texture: glium::texture::SrgbTexture2d,

    buffers: PlanetBuffers,

    surface: surface::Surface,

    axis: glm::Vec3,

    to_sun: glm::Vec3
}
impl Planet{
    pub fn new(display:&glium::Display, texture:glium::texture::SrgbTexture2d, iterations :u8)->Planet{
        
        let axis = glm::vec3(0.0,1.0,0.25).normalize();

        //generates base shape
        let base_shape = graphics::shapes::Shape::icosahedron()
            .subdivide(iterations)
            .normalize();
        
        //creates planet surface
        let mut surface = surface::Surface::new(&base_shape);

        //maps vertices of base shape into format used in buffer
        let mapped_vertices:Vec<graphics::VertexPos> = base_shape.vertices
            .iter()
            .map(|v| graphics::VertexPos{
                position:[v.x,v.y,v.z],
            })
            .collect();

        Planet{
            texture,

            buffers: 
            PlanetBuffers{
                //buffer containing base shape of planet, most likely the sphere
                shape_data: glium::VertexBuffer::new(display,&mapped_vertices).unwrap(),
                //buffer containing cell data needed for rendering, dynamic as this will change frequently
                planet_data: glium::VertexBuffer::dynamic(display, &surface.contents).unwrap(),
                //indices, define triangles of planet
                indices: glium::IndexBuffer::new(display,glium::index::PrimitiveType::TrianglesList, &base_shape.indices).unwrap(),
            },

            surface: surface,

            axis: axis,

            to_sun: glm::vec3(1.0,0.0,0.0)
        }
    }

    pub fn update(&mut self, years: f32){
        //latitude that gets maximum sunlight from the sun
        let sun_max = glm::dot(&self.to_sun, &self.axis);

        //updates cell temp
        self.surface.contents.iter_mut()
            .zip(self.surface.positions.iter())
            .for_each(|c|
                c.0.temperature = (1.0-c.0.height)* 
                glm::max2_scalar(1.0-f32::abs(sun_max- glm::dot(c.1,&self.axis)), 0.0));

        //one year is 360 days here for simplicity
        self.to_sun= glm::rotate_y_vec3(&self.to_sun, years*(std::f32::consts::PI*2.0));
        
        //write surface contents to planet buffer
        self.buffers.planet_data.write(&self.surface.contents);
    }

    pub fn draw(&self, target:&mut glium::Frame, program:&glium::Program, params:&glium::DrawParameters,cam:&graphics::camera::Camera){
        //let translation:[[f32;4];4] =  glm::translation(&glm::vec3(0.0,0.0,0.0)).into();
        let pers:[[f32;4];4] = cam.perspective.into();
        let view:[[f32;4];4] = cam.view.into();
        let uniform = glium::uniform! {
            perspective:pers,
            view: view,
            //world: translation,

            tex: glium::uniforms::Sampler::new(&self.texture).wrap_function(glium::uniforms::SamplerWrapFunction::Clamp),
            to_light: [self.to_sun.x,self.to_sun.y,self.to_sun.z]
        };

        target.draw((&self.buffers.shape_data,&self.buffers.planet_data),&self.buffers.indices,program,&uniform,params).unwrap();
    }
}