use std::{vec,collections::HashMap};

use egui::epaint::ahash::{HashSet, HashSetExt};
use glm::all;
//external crates
use noise::{NoiseFn, Perlin, Seedable};
use nalgebra_glm as glm;
use rand::{Rng, seq::SliceRandom};

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
    pub fn new(pos:glm::Vec3)->Cell{
        Cell { 
            contents: CellData { 
                position: pos.into(),
                height: 0.0,
                humidity: 0.0,
                temperature: 0.0
            },
            position:pos,
            plate: Some(0)
        }
    }
}

//contains all data for the surface of the planet
pub struct Surface{
    //every cell
    pub cells: Vec<Cell>,
    //triangle data, u32 as thats whats needed for passing to gpu
    pub triangles: Vec<u32>,
    //contains indices of all cells not in use
    pub bank: Vec<usize>,
    //distnace used for cell collisions, absolute closest one can be to aother before one gets destroyed
    pub cell_distance: f32,
    //all tectonic plates on the surface
    pub plates: Vec<Plate>,
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

        //length of edge
        let cell_distance = (cells[edges[0].0].position - cells[edges[0].1].position).magnitude()*0.75;

        //creates randomized plates for surface
        let mut plates:Vec<Plate> = (0..gen.plate_no)
            .map(|_|{
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
                    speed: rng.gen_range(0.00..0.2)/6371000.0, //done in meters per second, 6371000 is earths radius
                }
            })
            .collect();

        if !plates.is_empty(){
            //place seed cells randomly for each plate for each to spread out from
            for plate in 0..plates.len(){
                let target = rng.gen_range(0..cells.len());
                cells[target].plate = Some(plate);
            }

            //fill planet with the cells via random fill, if there are plates to even fill with
            while cells.iter().any(|c| !c.plate.is_some()){
                //get all plate boundries
                let plate_boundries:Vec<&(usize,usize)> = edges.iter()
                    .filter(|e| 
                    &cells[e.0].plate != &cells[e.1].plate)
                    .collect();

                for _ in 0..16{
                    //extend plate across random boundries 16 times
                    let edge = plate_boundries.choose(&mut rng).unwrap();
                    if cells[edge.0].plate == None{
                        cells[edge.0].plate = cells[edge.1].plate;
                    }else if cells[edge.1].plate == None{
                        cells[edge.1].plate = cells[edge.0].plate;
                    }
                }
            }
        }
        

        Surface{
            cells,
            triangles: shape.indices,
            bank: Vec::with_capacity(shape.vertices.len()/2),
            cell_distance,
            plates,
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
    pub fn tectonics(&mut self,years:f32){
        //for every cell with plate info, move according to plate
        for cell in self.cells.iter_mut().filter(|c|c.plate.is_some()){
            //plate cell belongs too
            let plate = &self.plates[cell.plate.unwrap()];
            //translate according to plate
            cell.position= glm::rotate_vec3(&cell.position, plate.speed*years, &plate.axis);
            //put cell pos into cell data
            cell.contents.position=cell.position.into();
        }
        
        //the behaviours, divergent, convergent, transform
        //diverge -> new cell at lengthened tris, center
        //converge -> densest of two close cells destroyed, other get higher besed
        //transform -> search connections of too far cell for closer one, closer replaces old in connections
        
        //there is a threshhold for connection length
        //for each connection in cell, if connection too long, search that second cells connections
        //and select any within threshhold for use as new connection
        
        let edges = indices_to_edges(&self.triangles);

        //filter edges to get only ones on plate boundries, then test for the collisions
        let plate_boundries:Vec<&(usize,usize)> = edges.iter()
            .filter(|e| 
                &self.cells[e.0].plate != &self.cells[e.1].plate)
                .collect();

        //for each edge along the plate boundry
        for edge in plate_boundries{
            
            let edge_length = edge_length(&self.cells, edge);
            //if cells collide
            if edge_length < self.cell_distance{
                //when two collide remove the denser one, as it subducts
                if self.plates[self.cells[edge.0].plate.unwrap()].density<self.plates[self.cells[edge.1].plate.unwrap()].density{
                    //if edge.0 is less dense, edge.1 is destroyed and subducts
                    self.remove_cell(edge.1);
                    self.cells[edge.0].contents.height +=0.5
                }else{
                    //otherwise inverse happens
                    self.remove_cell(edge.0);
                    self.cells[edge.1].contents.height +=0.5
                }
            //if cells split too far, spawn new one at midpoint
            }
            else if edge_length > self.cell_distance*1.75{
                self.add_cell(*edge);
            }
        }

        //retriangulate mesh
        //project points stereographic
        let mut all_points:Vec<glm::Vec3> = self.cells.iter()
            .map(|cell| stereographic(cell.position))
            .collect();
        //get indices of all points excluding those in bank
        let to_triangulate:Vec<u32> = (0..all_points.len()).into_iter()
            .filter(|x| !self.bank.contains(x))
            .map(|x| x as u32)
            .collect();

        self.triangles = bowyer_watson(&mut all_points, &to_triangulate);
    }

    //remove cell
    pub fn remove_cell(&mut self,cell: usize){
        //hashset to ensure surrounding points are unique
/*         let mut surrounding_points:HashSet<u32> = HashSet::with_capacity(6);

        //filter out triangles that contain cell, record all other points if they do
        self.triangles = self.triangles.chunks(3)
            .filter(|chunk| {
                //get only the triangles which do not contain the target cell
                if !chunk.contains(&(cell as u32)){
                    true
                }else{
                    //remove cell from triangles
                    let mut tri:Vec<u32> = chunk.iter()
                        .filter(|x| **x != cell as u32)//make sure ther original cell is excluded
                        .map(|x| {
                            surrounding_points.insert(*x);//insert points into hashset
                            *x
                        }) 
                        .collect();
                    false
                }
            })
            .flatten()
            .map(|n|*n)
            .collect();

        let mut all_points:Vec<glm::Vec3> = self.cells.iter()
            .map(|cell| stereographic(cell.position))
            .collect();

        //gets new triangulation 
        let mut triangulation = bowyer_watson(&mut all_points,&mut Vec::from_iter(surrounding_points));
        //adds new triangles
        self.triangles.append(&mut triangulation);*/
            
        //then marks cell as unused by pushing to the cell bank    
        self.bank.push(cell);
    }

    //adds a new cell to the planet between two other cells
    pub fn add_cell(&mut self, parents:(usize,usize)){
        //gets index of new cell to be used if avaliable from bank
        let cell = match self.bank.pop(){
            Some(c) => c,
            None => return, //if no avaliable cells in bank, does nothing
        };

        /* 
        //vec to store surrounding triangles
        let mut tris = Vec::with_capacity(6);

        //removes any triangle containing both parent cells, as these are the ones which will obstruct the new cell, they are also stored for later
        self.triangles = self.triangles.chunks(3)
            .filter(|chunk| {
                //if triangle contains both parents, appends triangle
                if chunk.contains(&(parents.0 as u32))&&chunk.contains(&(parents.1 as u32)){
                    tris.append(&mut vec![chunk[0],chunk[1],chunk[2]]);
                    false
                }else{
                    true
                }
            })
            .flatten()
            .map(|n|*n)
            .collect();*/

        //get midpoint between the two parent cells
        let mid = (self.cells[parents.0 as usize].position+self.cells[parents.1 as usize].position)*0.5;

        //use cell from bank as new cell between the parent cells
        self.cells[cell as usize] = Cell::new(glm::normalize(&mid));

        //add new triangles, connecting the new cell
       // self.triangles.append(&mut connect_point(tris, cell as u32));
    }
}