use std::vec;

//external crates
use noise::{NoiseFn, Perlin, Seedable};
use nalgebra_glm as glm;
use rand::Rng;

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

//get connections of every cell
pub fn indices_to_connections(indices: &Vec<u32>)->Vec<Vec<u32>>{
    //TODO: FIND MORE EFFICIENT WAY TO DO THIS, IM SURE THERE IS ONE
    //iterate through indices, for every index, store other two in triangle
    let mut connections:Vec::<Vec<u32>> = vec![Vec::with_capacity(6);indices.len()/3];
    
    indices.chunks(3)
        .for_each(|x|//for each triangle
            {
                //adds connections of each vert in triangle
                for i in 0..3{
                    connections[x[i] as usize].push(x[(i+1)%3] );
                }
            }
        );
    connections
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
    //physical position of cell
    pub position: glm::Vec3,
    //plate that the cell belongs too
    pub plate: Option<usize>
}
impl Cell{
    pub fn new(pos:glm::Vec3)->Cell{
        Cell { 
            contents: CellData { 
                position: [pos.x,pos.y,pos.z],
                height: 0.0,
                humidity: 0.0,
                temperature: 0.0
            },
            position:pos,
            plate: None
        }
    }
}

//contains all data for the surface of the planet
pub struct Surface{
    //every cell
    pub cells: Vec<Cell>,
    //triangle data
    pub triangles: Vec<u32>,
    //contains indices of all cells not in use
    pub bank: Vec<u32>,
    //all tectonic plates on the surface
    pub plates: Vec<Plate>,
}
impl Surface{
    pub fn new(shape: shapes::Shape,plate_num: u32, seed:u32)->Surface{
        let mut rng = rand::thread_rng();
        //creates cells for surface
        let cells:Vec<Cell> = {
            let perlin = Perlin::new(seed);
            //generates cells with perlin noise
            shape.vertices.clone().into_iter()
            .map(|pos|
                Cell{
                    contents: CellData{
                        position: [pos.x,pos.y,pos.z],
                        height: octive_noise(perlin, &pos, 2.5, 7, 0.6, 2.5),
                        humidity: octive_noise(perlin, &(pos+glm::vec3(0.0,100.0,0.0)), 2.25, 5, 0.55, 2.5),
                        temperature: 0.5,
                    },
                    position: pos,
                    plate: if pos.x>=0.0{//two split plates for debugging reasons
                        Some(0)
                    } else{
                        Some(1)
                    }
                }
            )
            .collect()
        };

        //creates randomized plates for surface
        let mut plates:Vec<Plate> = Vec::with_capacity(plate_num as usize);
        for _ in 0..plate_num{
            plates.push({
                //randomized axis the plate moves around
                let rand_axis = {
                    let x:f32 = rng.gen_range(0.0..=glm::two_pi());
                    let y:f32 = rng.gen_range(0.0..=glm::two_pi());
                    glm::rotate_y_vec3(
                        &glm::rotate_x_vec3(&glm::vec3(0.0,1.0,0.0),x), 
                        y)
                };

                Plate {
                    axis: rand_axis,
                    density: rng.gen_range(0.0..10.0),
                    speed: 0.005,
                }
            });
        }

        Surface{
            cells,
            triangles: shape.indices,
            bank: Vec::with_capacity(shape.vertices.len()/2),
            plates,
        }
    }

    pub fn axial_tilt(&mut self,years:f32){
        
    }

    pub fn tectonics(&mut self,years:f32){
        //for every cell with plate info
        for cell in self.cells.iter_mut().filter(|c|c.plate.is_some()){
            //plate cell belongs too
            let plate = &self.plates[cell.plate.unwrap()];
            //translate according to plate
            cell.position= glm::rotate_vec3(&cell.position, plate.speed*years, &plate.axis);
  
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

        for cell in indices_to_connections(&self.triangles){
            
        }
    }


    //remove cell
    pub fn remove_cell(&mut self,cell: u32){
        //filter out triangles that contain cell
        self.triangles = self.triangles.chunks(3)
            .filter(|chunk| !chunk.contains(&cell))//get only the triangles which do not contain the target cell
            .flatten()
            .map(|n|*n)
            .collect();
        //then marks cell as unused by pushing to the cell bank    
        self.bank.push(cell);
        
    }

    //adds a new cell to the planet between two other cells
    pub fn add_cell(&mut self, parents:(u32,u32)){
        //gets index of new cell to be used if avaliable from bank
        let cell = match self.bank.pop(){
            Some(c) => c,
            None => return //if no avaliable cells in bank, does nothing
        };

        //removes any triangle containing both parent cells, as these are the ones which will obstruct the new cell
        self.triangles = self.triangles.chunks(3)
            .filter(|chunk| !(chunk.contains(&parents.0)&&chunk.contains(&parents.1)))//get only the triangles which do not contain the target cells
            .flatten()
            .map(|n|*n)
            .collect();

        //gets other two cells which are needed for triangulation
        

        //get midpoint between the two parent cells
        let mid = (self.cells[parents.0 as usize].position+self.cells[parents.1 as usize].position)*0.5;

        //use cell from bank as new cell between the parent cells
        self.cells[cell as usize] = Cell::new(glm::normalize(&mid));
    }

    pub fn update(&mut self,years:f32){
        self.tectonics(years);
    }
}