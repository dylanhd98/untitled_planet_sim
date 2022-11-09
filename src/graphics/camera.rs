use nalgebra_glm as glm;

pub struct Camera{
    //matrices
    pub perspective:glm::Mat4,
    pub view:glm::Mat4,
    //vectors
    pub pos: glm::Vec3,
    pub target: glm::Vec3,
    pub up: glm::Vec3

}
impl Camera{
    //only uses these parameters as that fits my general use
    pub fn new(ratio:f32, pos: glm::Vec3, target: glm::Vec3, up: glm::Vec3)->Camera{
        Camera{
            //aspect ratio, fov, near field, far field
            perspective : glm::perspective(ratio, 3.14 / 4.0, 0.01, 1024.0),
            view: glm::look_at(
                &pos,//eye position
                &target,//looking at
                &up),//up

            pos,
            target,
            up,
        }
    }
    
    //updates matrices
    pub fn update_view(&mut self){
        self.view = glm::look_at(
            &self.pos,//eye position
            &self.target,//looking at
            &self.up);//up
    }
}