
//external crates
use std::{vec,collections::{HashMap,HashSet}};
use noise::{NoiseFn, Perlin, Seedable};
use nalgebra_glm as glm;
use rand::{Rng, seq::SliceRandom, rngs::ThreadRng};

//internal crates
use crate::graphics::shapes;
use super::{GenInfo, SimInfo,utils::*};


//data for each cell on the planet, for rendering
#[derive(Copy, Clone)]
pub struct CellData {
    //position in space of cell, should be normalized
    pub position: [f32;3],
    //height of land in cell from sea level, in km, should be in range -10km to 10-km
    pub height: f32,
    //absolute humidity, as g/m^3, range of 0 to 100
    pub humidity: f32,
    //temperature, in degrees C, range of -50 to 50
    pub temperature: f32
}
glium::implement_vertex!(CellData,position,height,humidity,temperature);

//data for every plate
pub struct Plate{
    //axis around which the plate rotates
    axis: glm::Vec3,
    //density of plates determines which will overlap another and nature of collision
    density: f32,
    //cm per year, avg is 5-15, earth rad = 6,371km,
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
    //creates effectivly blank cell at pos
    pub fn new(pos:glm::Vec3,plate: Option<usize>)->Cell{
        Cell { 
            contents: CellData { 
                position: pos.into(),
                height: -10.0,
                humidity: 0.0,
                temperature: 0.0
            },
            position:pos,
            plate: plate
        }
    }
}

//contains all data for the surface of the planet
pub struct Surface{
    //every cell
    pub cells: Vec<Cell>,
    //triangle data, u32 as thats whats needed for passing to gpu
    pub triangles: Vec<u32>,
    //all tectonic plates on the surface
    pub plates: Vec<Plate>,
    //contains indices of all cells not in use
    pub bank: HashSet<usize>,
    //distace used for cell collisions, absolute closest one can be to another before one gets destroyed
    cell_distance: f32,
    //time passed since last triangulation
    since_triangulation:f32,
    //random generator for surface
    rng: ThreadRng,
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
                Cell{
                    contents: CellData{
                        position: pos.into(),
                        //multiplied by 10 to get hight in the -10km to 10km range
                        height: octive_noise(perlin, &pos, 2.5, 7, 0.6, 2.5)*10.0,
                        //humidity to be in range 0 to 100, so (perlin+1)*50
                        humidity: (octive_noise(perlin, &(pos+glm::vec3(0.0,100.0,0.0)), 2.25, 5, 0.55, 2.5)+1.0)*50.0,
                        //temp set to zero bc its raised almost immedietely in the sim
                        temperature: 0.0,
                    },
                    position: pos,
                    plate: None
                }
            )
            .collect()
        };

        let edges = indices_to_edges(&shape.indices);

        //creates randomized plates for surface
        let mut plates:Vec<Plate> = (0..gen.plate_no)
        .map(|_|{
            //randomized axis the plate moves around
            let rand_axis = {
                let x:f32 = rng.gen_range(0.0..=glm::two_pi());
                let y:f32 = rng.gen_range(0.0..=glm::two_pi());
                glm::rotate_y_vec3(
                    &glm::rotate_x_vec3(&glm::vec3(0.0,1.0,0.0),x), y)
            };

            Plate {
                axis: rand_axis,
                density: rng.gen_range(0.0..10.0),//not representative of real value, just used to compare plates
                speed: rng.gen_range(0.00..0.2)/6371000.0, //done in meters per second, 6371000 is earths radius
            }
        })
         .collect();

        //length of edge
        let cell_distance = (cells[edges[0].0].position - cells[edges[0].1].position).magnitude();

        let mut surface = Surface{
            cells,
            triangles: shape.indices,
            plates,
            bank: HashSet::with_capacity(shape.vertices.len()/2),
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

    //handles tempereture updating
    pub fn temperature(&mut self,years:f32,sim_info: &SimInfo){
        //latitude that gets maximum sunlight from the sun
        let sun_max = glm::dot(&sim_info.to_sun, &sim_info.axis);

        //updates temp for each
        for cell in self.cells.iter_mut(){
            //amount of light recieved as percentage compared to ideal
            //calculates latitude and gets its distance from the ideal/max 
            let light_angle_multiplier = glm::max2_scalar(1.0-f32::abs(sun_max- glm::dot(&cell.position,&sim_info.axis)), 0.0);
            //multiplies ideal temp by angle, then takes lapse rate*height away if above sea level
            cell.contents.temperature = ((40.0*light_angle_multiplier)-(glm::max2_scalar(cell.contents.height,0.0)*sim_info.lapse_rate))*sim_info.greenhouse_effect;
        }
    }

    //handles the teconics on the planets surface
    pub fn tectonics(&mut self,years:f32,sim_info: &mut SimInfo){
        //for every cell with plate info, move according to plate, but also try to stay as far as possible from neighbors of same plate
        for cell in self.cells.iter_mut().filter(|c|c.plate.is_some()){
            //plate cell belongs too
            let plate = &self.plates[cell.plate.unwrap()];
            //translate according to plate
            cell.position = glm::rotate_vec3(&cell.position, plate.speed*years, &plate.axis);
            //put cell pos into cell data
            cell.contents.position=cell.position.into();
        }

        //update counter, check if exceeds interval
        if sim_info.triangulation_interval > self.since_triangulation{
            self.since_triangulation +=years;
            return;
        }
        self.since_triangulation = 0.0;
        
        //get edges from triangles of mesh
        let edges = indices_to_edges(&self.triangles);

        //filter edges to get only ones on plate boundries, then test for the collisions
        let plate_boundaries:Vec<&(usize,usize)> = edges.iter()
            .filter(|e| 
                &self.cells[e.0].plate != &self.cells[e.1].plate)
                .collect();

        //plate boundery edges currently colliding
        let mut colliding = Vec::with_capacity(plate_boundaries.len());
        //plate boundry edges where plates are diverging
        let mut diverging = Vec::with_capacity(plate_boundaries.len());
        //find and store boundries of each type
        for edge in plate_boundaries{
            //get edge length
            let edge_length = edge_length(&self.cells, edge);
            //if edge length too low, colliding
            if edge_length < self.cell_distance{
                colliding.push(edge);
            }
            //if edge too long, diverging
            else if edge_length > self.cell_distance*1.25{
                diverging.push(edge);
            }
        }
        //now deal with the different types of boundries
        for edge in colliding{
            //sort by density
            let dense_sorted =
            if self.plates[self.cells[edge.0].plate.unwrap()].density<self.plates[self.cells[edge.1].plate.unwrap()].density{
                (edge.0,edge.1)
            }else{
                (edge.1,edge.0)
            };
            //when two collide remove the denser one, as it subducts
            self.remove_cell(dense_sorted.0,dense_sorted.1);
            self.cells[dense_sorted.1].contents.height +=0.05
        }
        //handle divering plate boundaries, place new cells at their mid points when able
        //let banked_cells = self.bank.iter();
        for edge in diverging{
            //self.bank.remove(cell);
            //self.add_cell(*edge, *cell);
        }
        
/* 
        //for each edge along the plate boundry
        for edge in plate_boundaries{
            //get edge length
            let edge_length = edge_length(&self.cells, edge);
            //if cells collide
            if edge_length < self.cell_distance{
                //sort by density
                let dense_sorted =
                if self.plates[self.cells[edge.0].plate.unwrap()].density<self.plates[self.cells[edge.1].plate.unwrap()].density{
                    (edge.0,edge.1)
                }else{
                    (edge.1,edge.0)
                };
                //when two collide remove the denser one, as it subducts
                self.remove_cell(dense_sorted.0);
                self.cells[dense_sorted.1].contents.height +=1.0
            }
            //if cells split too far, spawn new one at midpoint
            else if edge_length > self.cell_distance*1.25{
                self.add_cell(*edge);
            }
        }*/
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

        //let mut all_points:Vec<glm::Vec3> = self.cells.iter()
        //    .map(|cell| stereographic(cell.position,&glm::Vec3::y()))
        //    .collect();

        //gets new triangulation 
        //let mut triangulation = bowyer_watson(&mut all_points,&mut Vec::from_iter(surrounding_points));
        //adds new triangles
        self.triangles.append(&mut connect_point(surrounding_tris, provoking as u32));
            
        //then marks cell as unused by pushing to the cell bank    
        self.bank.insert(cell);
    }

    //adds a new cell to the planet between two other cells
    pub fn add_cell(&mut self, parents:(usize,usize),cell:usize){
        //gets index of new cell from bank to be used if avaliable
        //let cell = match self.bank.pop(){
        //    Some(c) => c,
        //    None => return, //if no avaliable cells in bank, does nothing
        //};

        //get midpoint between the two parent cells
        let mid = (self.cells[parents.0 as usize].position+self.cells[parents.1 as usize].position)*0.5;
        //select random plate of the two parents to make the new plate belong to
        let plate = if self.rng.gen(){
            self.cells[parents.0 as usize].plate
        }else{
            self.cells[parents.1 as usize].plate
        };
        //use cell from bank as new cell between the parent cells
        self.cells[cell as usize] = Cell::new(glm::normalize(&mid),plate);

        //hashset to ensure surrounding points are unique
        let mut surrounding_points:HashSet<u32> = HashSet::with_capacity(12);

        //removes any triangle containing both parent cells, as these are the ones which will obstruct the new cell, they are also stored for later
        self.triangles = self.triangles.chunks(3)
            .filter(|chunk| {
                //if triangle contains both parents, insert triangle points into hash set
                if chunk.contains(&(parents.0 as u32))&&chunk.contains(&(parents.1 as u32)){
                    for c in chunk.iter(){
                        surrounding_points.insert(*c);
                    }
                    false
                }else{
                    true
                }
            })
            .flatten()//flatten to remove seperation of triangles
            .map(|n|*n)
            .collect();

        //add new cell to hashset aswell, then triangulate
        surrounding_points.insert(cell as u32);
        //gets all points projected
        let mut all_points:Vec<glm::Vec3> = self.cells.iter()
            .map(|cell| stereographic(cell.position,&glm::Vec3::y()))
            .collect();
        //triangulate points
        let mut triangulation = bowyer_watson(&mut all_points,&mut Vec::from_iter(surrounding_points));
        //adds new triangles to mesh
        self.triangles.append(&mut triangulation);
    }
}