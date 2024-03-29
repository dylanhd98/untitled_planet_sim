//managing all the planets systems
//implimented on surface in new module for better structuring

use std::collections::HashSet;

//external crates
use nalgebra_glm as glm;
//internal modules
use super::{SimInfo,utils::*};

impl super::surface::Surface{
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
            cell.contents.temperature = ((sim_info.base_temp*light_angle_multiplier)-(glm::max2_scalar(cell.contents.height,0.0)*sim_info.lapse_rate));
        }
    }

    //handles the teconics on the planets surface
    pub fn tectonics(&mut self,years:f32,sim_info: &mut SimInfo){        
        //for every cell with plate info, move according to plate
        for cell in self.cells.iter_mut().filter(|c|c.plate.is_some()){
            //plate cell belongs too
            let plate = &self.plates[cell.plate.unwrap()];
            //translate according to plate
            cell.position = glm::rotate_vec3(&cell.position, plate.speed*years, &plate.axis);
            //put cell pos into cell data
            cell.contents.position=cell.position.into();
        }
        //update plates internal translation matrices aswell
        for plate in self.plates.iter_mut(){
            plate.translation = glm::rotate(&plate.translation, plate.speed*years,  &plate.axis);
        }

        //update counter, check if exceeds interval
        if sim_info.triangulation_interval > self.since_triangulation{
           self.since_triangulation +=years;
            return;
        }
        self.since_triangulation = 0.0;
        
        //different types of boundary triangles
        let mut divergent:Vec<u32> = Vec::with_capacity(self.triangles.len()/3);
        let mut convergent:Vec<u32> = Vec::with_capacity(self.triangles.len()/3);
        let mut transform:Vec<u32> = Vec::with_capacity(self.triangles.len()/3);

        //filter boundary triangles out of mesh, record them
        self.triangles = self.triangles.chunks(3)
            .filter(|t| {
                //filter out triangles that contain cells in more than one plate, store seperately
                let plate = self.cells[t[0] as usize].plate;
                if t.iter().all(|i| self.cells[*i as usize].plate==plate){
                    true
                }else{
                    //evaluate boundry type of triangle and store appropriately
                    //use perimeter^2 of triangle to determine boundry type
                    let sqr_perim:f32 = t.iter().zip(t.iter().skip(1))
                    .map(|e| (self.cells[*e.0 as usize].position-self.cells[*e.1 as usize].position).magnitude_squared())
                    .sum();

                    if sqr_perim/3.0 < self.cell_distance*self.cell_distance*0.5{
                        t.iter().for_each(|x| convergent.push(*x));
                    }
                    else if sqr_perim/3.0 > self.cell_distance*self.cell_distance*1.25{
                        t.iter().for_each(|x| divergent.push(*x));
                    }
                    else{
                        t.iter().for_each(|x| transform.push(*x));
                    }
                    false
                }
            })
            .flatten()
            .map(|x| *x)
            .collect();

        //act on boundary triangles based what they are catigorized as
        //println!("\nConverging:{:?}\nTransform:{:?}\nDivergent:{:?}",convergent.len(),transform.len(),divergent.len());
        //remove cells in converging
        for tri in convergent.chunks(3){
            //remove most dense cell
            self.bank.insert(tri[0] as usize);
            
        }
        //records cells already added to prevent duplicates
        let mut newly_added:HashSet<u32> = HashSet::new();
        //get cells that can be added to mesh
        let mut bank_cells:Vec<usize> = self.bank.drain().collect();
        //add new cells according to base mesh
        for tri in divergent.chunks(3){
            //get index of cell to be added to mesh
            let cell_to_add = if let Some(cell) = bank_cells.pop(){
                cell
            }else{
                //no free space for new cell
                continue;
            };
            //find edge with two points of the same plate in tri to be used create new one
            let shared_plate = tri.iter().zip(tri.iter().skip(1))
                .find(|edge| self.cells[*edge.0 as usize].plate == self.cells[*edge.1 as usize].plate);
            if let Some(edge) = shared_plate{
                //add new cell that corrosponds to the next point in the virtual mesh's 
                self.add_cell((*edge.0 as usize,*edge.1 as usize), cell_to_add);
            }else{
                //no shared plate found
                continue;
            }
        }

        //turn triangles into polygons

        //triangulate polygons

        //add remaining unused cells back to bank
        self.bank = HashSet::from_iter(bank_cells.into_iter());

        //triangulate new boundary triangles, insert into mesh
        self.triangles.append(&mut transform);
    }
}