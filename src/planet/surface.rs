
//external crates
use std::{vec,collections::{HashMap,HashSet}};
use noise::{NoiseFn, Perlin, Seedable};
use nalgebra_glm as glm;
use rand::{Rng, seq::SliceRandom, rngs::ThreadRng};

//internal crates
use crate::graphics::shapes::{self, Shape};
use super::{GenInfo, SimInfo,utils::*};


//data for each cell on the planet, this data is sent to gpu
#[derive(Copy, Clone)]
pub struct CellData {
    //position in space of cell
    pub position: [f32;3],
    //height of land in cell from sea level, in km, should be in range -10km to 10-km
    pub height: f32,
    //absolute humidity, as g/m^3
    pub humidity: f32,
    //percentage water coverage 
    pub water: f32,
    //temperature, in degrees C
    pub temperature: f32
}
glium::implement_vertex!(CellData,position,height,humidity,water,temperature);

//data for every plate
pub struct Plate{
    //axis around which the plate rotates
    pub axis: glm::Vec3,
    //density of plates determines which will overlap another and nature of collision
    pub density: f32,
    //cm per year, avg is 5-15, earth rad = 6,371km,
    pub speed: f32,
}
impl Plate{
    //creates new random plate
    pub fn random(rng:&mut ThreadRng)->Plate{
        //randomized axis the plate moves around
        let rand_axis = {
            let x:f32 = rng.gen_range(-std::f32::consts::PI..=std::f32::consts::PI);
            let y:f32 = rng.gen_range(-std::f32::consts::PI..=std::f32::consts::PI);
            glm::rotate_y_vec3(
                &glm::rotate_x_vec3(&glm::Vec3::y(),x), y)
        };

        Plate {
            axis: rand_axis,
            density: rng.gen_range(0.0..10.0),//not representative of real value, just used to compare plates
            speed: rng.gen_range(0.00..0.2)/6371000.0, //done in meters per second, 6371000 is earths radius
        }
    }
}

//data relating to the cell
pub struct Cell{
    //what is contained within the cell
    pub contents: CellData,
    //physical position of cell
    pub position: glm::Vec3,
    //plate that the cell belongs too
    pub plate: Option<usize>,
    //index of cell on the original stored mesh
    pub base_index: u32
}
impl Cell{
    //creates effectivly blank cell at pos
    pub fn new(pos:glm::Vec3,base_index:u32,plate: Option<usize>)->Cell{
        Cell { 
            contents: CellData { 
                position: pos.into(),
                height: -10.0,
                humidity: 0.0,
                water: 0.0,
                temperature: 0.0
            },
            position:pos,
            plate,
            base_index,
        }
    }
    //creates a new cell with perlin noise
    pub fn from_perlin(position:glm::Vec3,base_index: u32,plate: Option<usize>,perlin: Perlin)->Cell{
        let height = octive_noise(perlin, &position, 2.5, 7, 0.6, 2.5)*10.0;
        Cell{
            contents: CellData{
                position: position.into(),
                //multiplied by 10 to get hight in the -10km to 10km range
                height,
                //humidity to be in range 0 to 100, so (perlin+1)*50
                humidity: (octive_noise(perlin, &(position+glm::vec3(0.0,100.0,0.0)), 2.25, 5, 0.55, 2.5)+1.0)*50.0,
                //initial water content just based on sea level
                water: if height<0.0 {1.0} else {0.0},
                //temp set to zero bc its raised almost immedietely in the sim
                temperature: 0.0,
            },
            position,
            plate,
            base_index
        }   
    }
}

//contains all data for the surface of the planet
pub struct Surface{
    //base mesh containing the orignal planet shape
    pub base_mesh: Shape,
    //every cell
    pub cells: Vec<Cell>,
    //triangle data, u32 as thats whats needed for passing to gpu
    pub triangles: Vec<u32>,
    //all tectonic plates on the surface
    pub plates: Vec<Plate>,
    //contains indices of all cells not in use
    pub bank: HashSet<usize>,
    //distace used for cell collisions, absolute closest one can be to another before one gets destroyed
    pub cell_distance: f32,
    //time passed since last triangulation
    pub since_triangulation:f32,
    //random generator for surface
    pub rng: ThreadRng,
}
impl Surface{
    pub fn new(shape: shapes::Shape,gen: &GenInfo)->Surface{
        let mut rng = rand::thread_rng();
        //creates cells for surface
        let mut cells:Vec<Cell> = {
            let perlin = Perlin::new(gen.seed);
            //generates cells with perlin noise
            shape.vertices.clone().into_iter()
            .map(|pos|
                Cell::from_perlin(pos,0 ,None, perlin)
            )
            .collect()
        };
        //get edges from mesh
        let edges = indices_to_edges(&shape.indices);

        //creates randomized plates for surface
        let mut plates:Vec<Plate> = (0..gen.plate_no)
        .map(|_|{
            Plate::random(&mut rng)
        })
         .collect();

        //length of edge to be used to determine collision
        let cell_distance = (cells[edges[0].0].position - cells[edges[0].1].position).magnitude();
        //bank for recording unused vertices
        let bank = HashSet::with_capacity(shape.vertices.len()/2);
        
        let triangles = shape.indices.clone();

        let mut surface = Surface{
            base_mesh: shape,
            cells,
            triangles,
            plates,
            bank,
            cell_distance,
            since_triangulation:0.0,
            rng,
        };
        surface.fill_plates();
        surface
    }

    //generates specified number of plates
    pub fn fill_plates(&mut self){
        if !self.plates.is_empty(){
            //unset plates from all cells
            self.cells.iter_mut()
                .for_each(|c| c.plate = None);
            //get edges to be used to extend plates accross
            let edges = indices_to_edges(&self.triangles);
            //place seed cells randomly for each plate for each to spread out from
            for plate in 0..self.plates.len(){
                let target = self.rng.gen_range(0..self.cells.len());
                self.cells[target].plate = Some(plate);
            }
            //fill planet with the plates via random fill, if there are plates to even fill with
            while (0..self.cells.len()).into_iter().any(|c| 
                !self.cells[c].plate.is_some()&&
                !self.bank.contains(&c)){
                //get all plate boundries
                let plate_boundries:Vec<&(usize,usize)> = edges.iter()
                    .filter(|e| 
                    &self.cells[e.0].plate != &self.cells[e.1].plate)
                    .collect();
                let extend_no = usize::max(plate_boundries.len()/8, 1);
                //extend plate across 1/8 of boundries randomly
                for _ in 0..extend_no{
                    let edge = plate_boundries.choose(&mut self.rng).unwrap();
                    if self.cells[edge.0].plate == None{
                        self.cells[edge.0].plate = self.cells[edge.1].plate;
                    }else if self.cells[edge.1].plate == None{
                        self.cells[edge.1].plate = self.cells[edge.0].plate;
                    }
                }
            }
        }
        
    }

    //remove cell
    pub fn remove_cell(&mut self,cell: usize,provoking: usize){
        //hashset to ensure surrounding points are unique
        //let mut surrounding_points:HashSet<u32> = HashSet::with_capacity(6);
        let mut surrounding_tris:Vec<u32> = Vec::with_capacity(16);

        //filter out triangles that contain cell, record all other points if they do
        self.triangles = self.triangles.chunks(3)
            .filter(|chunk| {
                //get only the triangles(chunks of 3 indices) which do not contain the target cell
                if !chunk.contains(&(cell as u32)){
                    true
                }else{
                    //add all surrounding points that arent the cell itself to hashset for triangulation
                    surrounding_tris.append(
                       &mut chunk.iter()
                        .map(|i| *i)
                        .collect());
                    false
                }
            })
            .flatten()
            .map(|n|*n)
            .collect();

        //gets new triangulation 
        let mut triangulation =  connect_point(surrounding_tris, provoking as u32);
        //adds new triangles
        self.triangles.append(&mut triangulation);
    }

    //adds a new cell to planet using a provoking edge belonging to one plate
    pub fn add_cell_new(&mut self,edge:(usize,usize),cell:usize){
        //get base mesh indices of cells on edge
        let base_edge = (self.cells[edge.0].base_index,self.cells[edge.1].base_index);
        //use base indices to search base mesh for triangle that edge exists in 
        //and return index of the third point
        let third_index:Option<u32> = self.base_mesh.indices.chunks(3)
            .find_map(|t| {
                //skip tri if doesnt contain both
                if !t.contains(&base_edge.0)||!t.contains(&base_edge.0){
                    return None;
                }
                for i in 0..3{
                    //check if edge is anywhere in triangle
                    if t[i] == base_edge.0 && t[(i+1)%3] == base_edge.1{
                        //return third point
                        return Some(t[(i+2)%3]);
                    }
                }
                None
            });
        //use index of new point to get pos of new point
        if let Some(index) = third_index {
            //get position of new cell
            let new_pos = self.base_mesh.vertices[index as usize];
            //use new pos to create new cell
            self.cells[index as usize] = Cell::new(new_pos, index, self.cells[edge.0].plate);
            //put new cell into planet mesh by connecting to provoking edge
        }
    }

    //adds a new cell to the planet between two other cells
    pub fn add_cell(&mut self, parents:(usize,usize),cell:usize){
        //select random plate of the two parents to make the new plate belong to
        let plate = if self.rng.gen(){
            self.cells[parents.0 as usize].plate
        }else{
            self.cells[parents.1 as usize].plate
        };
        //get midpoint between the two parent cells
        let mid = (self.cells[parents.0 as usize].position+self.cells[parents.1 as usize].position)*0.5;
        //use cell from bank as new cell between the parent cells
        self.cells[cell as usize] = Cell::new(glm::normalize(&mid),0,plate);

        //record surrounging triangles
        let mut surrounding_tris:Vec<u32> =Vec::with_capacity(16);

        //removes any triangle containing both parent cells, as these are the ones which will obstruct the new cell, they are also stored for later
        self.triangles = self.triangles.chunks(3)
            .filter(|chunk| {
                //if triangle contains both parents, record triangle
                if chunk.contains(&(parents.0 as u32))&&chunk.contains(&(parents.1 as u32)){
                    chunk.iter()
                        .for_each(|c| surrounding_tris.push(*c));
                    false
                }else{
                    true
                }
            })
            .flatten()//flatten to remove seperation of triangles
            .map(|n|*n)
            .collect();

        //triangulate points
        let mut triangulation = connect_point(surrounding_tris, cell as u32);
        //adds new triangles to mesh
        self.triangles.append(&mut triangulation);
    }
}