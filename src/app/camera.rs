use nalgebra_glm::{identity, look_at, perspective};
use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};
use std::f32::consts::PI;

pub struct Camera {
    z_near: f32,
    z_far: f32,
    fov: f32,

    pub perspective_mat: Mat4,
    pub view_mat: Mat4,
}

impl Camera {
    pub fn new(aspect: f32, z_near: f32, z_far: f32, fov: f32) -> Camera {
        let mut perspective_mat = perspective(aspect, PI / fov, z_near, z_far);
        perspective_mat = nalgebra_glm::scale(&perspective_mat, &Vec3::new(1.0, -1.0, 1.0));
        Camera {
            z_near: z_near,
            z_far: z_far,
            fov: fov,

            perspective_mat,
            view_mat: look_at(&Vec3::new(0.0, 0.0, 20.0), &Vec3::new(0.0, 0.0, 0.0), &Vec3::new(0.0, 1.0, 0.0)),
        }
    }

    pub fn look_vec(&self) -> Vec3 {
        Vec3::new(self.perspective_mat[(0, 2)], self.perspective_mat[(1, 2)], self.perspective_mat[(2, 2)])
    }
    pub fn up_vec(&self) -> Vec3 {
        Vec3::new(self.perspective_mat[(0, 1)], self.perspective_mat[(1, 1)], self.perspective_mat[(2, 1)])
    }
    pub fn look_cross_up_vec(&self) -> Vec3 {
        let lv = self.look_vec();
        let uv = self.up_vec();
        nalgebra_glm::cross::<f32, nalgebra_glm::U1>(&lv, &uv)
    }
    pub fn eye_pos_vec(&self) -> Vec3 {
        Vec3::new(self.perspective_mat[(3, 0)], self.perspective_mat[(3, 1)], self.perspective_mat[(3, 2)])
    }
    pub fn rotate(&mut self, radians: f32, axis: &Vec3) {
        let quaternion = nalgebra_glm::quat::<f32>(
            (radians / 2.0).cos(),
            axis.x * (radians / 2.0).sin(),
            axis.y * (radians / 2.0).sin(),
            axis.z * (radians / 2.0).sin(),
        );
        println!("{}", quaternion);
        self.view_mat = nalgebra_glm::quat_cast(&quaternion) * self.view_mat;
        // println!("{}", self.look_vec());
        // println!("{}", self.eye_pos_vec());
        let pos = self.eye_pos_vec();
        self.view_mat = nalgebra_glm::rotate(&nalgebra_glm::translate(&self.view_mat, &(-pos)), radians, axis);
        self.view_mat = nalgebra_glm::translate(&self.view_mat, &pos);
    }
    pub fn translate(&mut self, delta: &Vec3) {
        self.view_mat = nalgebra_glm::translate(&self.view_mat, delta)
    }
}
