use std::f64::consts::PI;
use std::os::raw::c_void;
use nalgebra::{Matrix4, Vector3};
use opencv::core::{Mat, MatTraitConst};
use opencv::imgproc::{COLOR_RGB2BGR, cvt_color};

pub type V3d = Vector3<f64>;

pub(crate) fn get_view_matrix(eye_pos: V3d) -> Matrix4<f64> {
    /*  implement your code here  */
    let mut t_view: Matrix4<f64> = Matrix4::identity();
    t_view.m14 = -eye_pos[0];
    t_view.m24 = -eye_pos[1];
    t_view.m34 = -eye_pos[2];
    let g = Vector3::new(0., 0., -1.);
    let t = Vector3::new(1., 0., 0.);
    let cross = g.cross(&t);
    let mut r_view: Matrix4<f64> = Matrix4::identity();
    r_view.m11 = cross[0];
    r_view.m21 = cross[1];
    r_view.m31 = cross[2];
    r_view.m12 = t[0];
    r_view.m22 = t[1];
    r_view.m32 = t[2];
    r_view.m13 = -g[0];
    r_view.m23 = -g[1];
    r_view.m33 = -g[2];
    r_view = r_view.transpose();
    let view = r_view * t_view;
    view
}

pub(crate) fn get_model_matrix(mut rotation_angle: f64) -> Matrix4<f64> {
    let mut model: Matrix4<f64> = Matrix4::identity();
    /*  implement your code here  */
    rotation_angle = rotation_angle / 180. * PI;
    model[(0,0)] = rotation_angle.cos();
    model[(0,1)] = -rotation_angle.sin();
    model[(1,0)] = rotation_angle.sin();
    model[(1,1)] = rotation_angle.cos();
    model
}

pub(crate) fn get_projection_matrix(eye_fov: f64, aspect_ratio: f64, z_near: f64, z_far: f64) -> Matrix4<f64> {
    let mut projection: Matrix4<f64> = Matrix4::identity();
    /*  implement your code here  */

    projection
}

pub(crate) fn frame_buffer2cv_mat(frame_buffer: &Vec<V3d>) -> opencv::core::Mat {
    let mut image = unsafe {
        Mat::new_rows_cols_with_data(
            700, 700,
            opencv::core::CV_64FC3,
            frame_buffer.as_ptr() as *mut c_void,
            opencv::core::Mat_AUTO_STEP,
        ).unwrap()
    };
    let mut img = Mat::copy(&image).unwrap();
    image.convert_to(&mut img, opencv::core::CV_8UC3, 1.0, 1.0).expect("panic message");
    cvt_color(&img, &mut image, COLOR_RGB2BGR, 0).unwrap();
    image
}