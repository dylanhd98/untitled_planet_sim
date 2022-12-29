//external crates
use glium::{Surface,glutin};
use nalgebra_glm as glm;

//other internal modules
use crate::graphics;

use self::surface::CellData;

//child modules
mod surface;

//struct containing all things needed passed to the gpu
pub struct RenderData{
    //planet vertices
    planet_data: glium::VertexBuffer<surface::CellData>,
    //triangles
    indices: glium::IndexBuffer<u32>,
    //texture lookup for surface
    texture: glium::texture::SrgbTexture2d,
    //how esagerated the planet surface will be
    pub scale: f32,
}


pub struct Planet{
    pub render_data: RenderData,

    pub surface: surface::Surface,

    axis: glm::Vec3,

    to_sun: glm::Vec3
}
impl Planet{
    pub fn new(display:&glium::Display, texture:glium::texture::SrgbTexture2d, iterations :u8,seed:u32)->Planet{
        
        let axis = glm::vec3(0.0,1.0,0.25).normalize();

        //generates base shape
        let base_shape = graphics::shapes::Shape::icosahedron()
            .subdivide(iterations)
            .normalize();
        
        //creates planet surface
        let mut surface = surface::Surface::new(base_shape,seed);

        //extract data for buffer
        let surface_contents:Vec<CellData> = surface.cells.iter()
            .map(|c|
            c.contents
            )
            .collect();

        Planet{
            render_data: 
            RenderData{
                //buffer containing cell data needed for rendering, dynamic as this will change frequently
                planet_data: glium::VertexBuffer::dynamic(display, &surface_contents).unwrap(),
                //indices, define triangles of planet
                indices: glium::IndexBuffer::new(display,glium::index::PrimitiveType::TrianglesList, &surface.triangles).unwrap(),

                texture,

                scale: 0.025
            },

            surface: surface,

            axis: axis,

            to_sun: glm::vec3(1.0,0.0,0.0)
        }
    }

    pub fn update(&mut self, years: f32,display:&glium::Display){
        self.surface.update(years);

        //latitude that gets maximum sunlight from the sun
        let sun_max = glm::dot(&self.to_sun, &self.axis);

        //updates cell temp based on distance from sun_max
        self.surface.cells.iter_mut()
            .for_each(|c|
                c.contents.temperature= (1.0-c.contents.height)* 
                glm::max2_scalar(1.0-f32::abs(sun_max- glm::dot(&c.position,&self.axis)), 0.0));

        //one year is 360 days here for simplicity
        self.to_sun= glm::rotate_y_vec3(&self.to_sun, years*(std::f32::consts::PI*2.0));
        
        //extract data needed for rendering out
        let surface_contents:Vec<CellData> = self.surface.cells.iter()
            .map(|c|
            c.contents
            )
            .collect();

        //write surface contents to planet buffer
        self.render_data.planet_data.write(&surface_contents);
        //FIX THIS, WORKS BUT BAD
        self.render_data.indices= glium::IndexBuffer::new(display,glium::index::PrimitiveType::TrianglesList, &self.surface.triangles).unwrap();        ;
    }

    pub fn draw(&self, target:&mut glium::Frame, program:&glium::Program, params:&glium::DrawParameters,cam:&graphics::camera::Camera){
        let pers:[[f32;4];4] = cam.perspective.into();
        let view:[[f32;4];4] = cam.view.into();
        let uniform = glium::uniform! {
            perspective:pers,
            view: view,

            tex: glium::uniforms::Sampler::new(&self.render_data.texture).wrap_function(glium::uniforms::SamplerWrapFunction::Clamp),
            to_light: [self.to_sun.x,self.to_sun.y,self.to_sun.z],
            terra_scale: self.render_data.scale,
        };

        target.draw(&self.render_data.planet_data,&self.render_data.indices,program,&uniform,params).unwrap();
    }
}