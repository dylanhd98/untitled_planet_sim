use std::vec;

//external crates
use noise::{NoiseFn, Perlin, Seedable};
use nalgebra_glm as glm;

//internal crates
use crate::graphics::shapes;

//data for each cell on the planet
#[derive(Copy, Clone)]
pub struct CellData {
    pub position: [f32;3],
    pub height: f32,
    pub humidity: f32,
    pub temperature: f32
}
glium::implement_vertex!(CellData,position,height,humidity,temperature);


//handles perlin noise for generating base
fn octive_noise(perlin: Perlin, pos:&glm::Vec3, scale:f32, octives:u8, persistance:f32, lacunarity:f32)->f32{
    let mut noise_value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;

    for _o in 0..octives{
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

//data for every plate
pub struct Plate{
    axis: glm::Vec3,
    density: f32,
    speed: f32,
}

//data relating to the cell
pub struct Cell{
    //what is contained within the cell
    pub contents: CellData,
    //all other cells the cell is connected too
    pub connections: Vec<u32>,
    //physical position of cell
    pub position: glm::Vec3, 
}

//contains all data for the surface of the planet
pub struct Surface{
    pub cells: Vec<Cell>,
    pub triangles: Vec<u32>,
    pub bank: Vec<u32>,//contains indices of all cells currently out of use
    
    pub plates: Vec<Plate>,
}
impl Surface{
    pub fn new(shape: shapes::Shape, seed:u32)->Surface{
        //creates cells for surface
        let cells:Vec<Cell> = {
            let perlin = Perlin::new(seed);
            //gets connections
            let connections = shape.indices_to_connections();
            //generates cells
            connections.into_iter()
            .zip(shape.vertices.clone().into_iter())
            .map(|cell|
                Cell{
                    contents: CellData{
                        position: [cell.1.x,cell.1.y,cell.1.z],
                        height: octive_noise(perlin, &cell.1, 2.5, 7, 0.6, 2.5),
                        humidity: octive_noise(perlin, &(cell.1+glm::vec3(0.0,100.0,0.0)), 2.25, 5, 0.55, 2.5),
                        temperature: 0.5,
                    },
                    connections: cell.0,
                    position: cell.1,
                }
            )
            .collect()
        };

        //creates plates for surface
        let plates = vec![
            Plate{
                axis: glm::vec3(0.0,0.0,1.0),
                density: 0.5,
                speed: 0.005,
            },
            Plate{
                axis: glm::vec3(0.0,0.0,1.0),
                density: 0.5,
                speed: -0.005,
            }
        ];

        Surface{
            cells,
            triangles: shape.indices,
            bank: Vec::with_capacity(shape.vertices.len()/2),
            plates,
        }
    }

    pub fn update_fill(&mut self){
        let new_heights:Vec<f32> = self.cells.iter()
            .map(|c| {
                if c.connections.iter().any(|conn| self.cells[*conn as usize].contents.height >=1.5){
                    1.5
                }else{
                    c.contents.height
                }
            })
            .collect();

        for (cell,height) in self.cells.iter_mut().zip(new_heights.into_iter()){
            cell.contents.height = height;
        }
    }

    pub fn unselct(&mut self){
        self.cells.iter_mut()
            .for_each(|c|c.contents.height = 0.0);
    }

    pub fn axial_tilt(&mut self,years:f32){
        
    }

    pub fn tectonics(&mut self,years:f32){
        for cell in self.cells.iter_mut(){
            //translate according to plate
            cell.position= 
            if cell.position.x>0.0{
                glm::rotate_y_vec3(&cell.position,years)
            }else{
                glm::rotate_y_vec3(&cell.position,-years)
            };
            //put cell pos into cell data
            cell.contents.position=[cell.position.x,cell.position.y,cell.position.z];
        }
        
        //the behaviours, divergent, convergent, transform
        //diverge -> new cell at lengthened tris, center
        //converge -> densest of two close cells destroyed, other get higher besed
        //transform -> search connections of too far cell for closer one, closer replaces old in connections
        
        //there is a threshhold for connection length
        //for each connection in cell, if connection too long, search that second cells connections
        //and select any within threshhold for use as new connection
        

    }

    fn triangulate(&self, cells: &Vec<u32>)-> Vec<u32>{
        vec![cells[0],cells[1],cells[2]]
    }

    pub fn remove_cell(&mut self,cell: u32){
        //removes any triangle containing cell
        self.triangles = self.triangles.chunks(3)
            .filter(|chunk| !chunk.contains(&cell))//get only the triangles which do not contain the target cell
            .flatten()
            .map(|n|*n)
            .collect();
        //then marks cell as unused by pushing to bank    
        self.bank.push(cell);
        //finally, triangulates surrounding cells
        self.triangles.append(&mut self.triangulate(&self.cells[cell as usize].connections))
    }

    pub fn update(&mut self,years:f32){
        self.tectonics(years);
        
    }
}