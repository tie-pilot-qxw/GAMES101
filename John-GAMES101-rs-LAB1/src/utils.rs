use nalgebra::{Matrix3, Matrix4, Vector3};
use opencv::core::{Mat, MatTraitConst};
use opencv::imgproc::{cvt_color, COLOR_RGB2BGR};
use std::f64::consts::PI;
use std::os::raw::c_void;

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
    model[(0, 0)] = rotation_angle.cos();
    model[(0, 1)] = -rotation_angle.sin();
    model[(1, 0)] = rotation_angle.sin();
    model[(1, 1)] = rotation_angle.cos();
    model
}

pub(crate) fn get_projection_matrix(
    mut eye_fov: f64,
    aspect_ratio: f64,
    z_near: f64,
    z_far: f64,
) -> Matrix4<f64> {
    /*  implement your code here  */
    eye_fov = eye_fov / 360. * PI;
    let l = -eye_fov.tan() * z_near;
    let r = -l;
    let b = l * aspect_ratio;
    let t = r * aspect_ratio;
    let mut temp: Matrix4<f64> = Matrix4::identity();
    temp.m11 = 2. / (r - l);
    temp.m22 = 2. / (t - b);
    temp.m33 = 2. / (z_far - z_near);
    let mut m_ortho: Matrix4<f64> = Matrix4::identity();
    m_ortho.m14 = -(r + l) / 2.;
    m_ortho.m24 = -(t + b) / 2.;
    m_ortho.m34 = -(z_near + z_far) / 2.;
    m_ortho = temp * m_ortho;
    let mut m_trans: Matrix4<f64> = Matrix4::zeros();
    m_trans.m11 = z_near;
    m_trans.m22 = z_near;
    m_trans.m33 = z_near + z_far;
    m_trans.m34 = -z_near * z_far;
    m_trans.m43 = 1.;
    let projection: Matrix4<f64> = m_ortho * m_trans;
    projection
}

pub(crate) fn frame_buffer2cv_mat(frame_buffer: &Vec<V3d>) -> opencv::core::Mat {
    let mut image = unsafe {
        Mat::new_rows_cols_with_data(
            700,
            700,
            opencv::core::CV_64FC3,
            frame_buffer.as_ptr() as *mut c_void,
            opencv::core::Mat_AUTO_STEP,
        )
        .unwrap()
    };
    let mut img = Mat::copy(&image).unwrap();
    image
        .convert_to(&mut img, opencv::core::CV_8UC3, 1.0, 1.0)
        .expect("panic message");
    cvt_color(&img, &mut image, COLOR_RGB2BGR, 0).unwrap();
    image
}

pub(crate) fn get_arbitrary_rotation(n: V3d, mut angle: f64) -> Matrix4<f64> {
    angle = angle / 180. * PI;
    let mut rotation: Matrix3<f64> = angle.cos() * Matrix3::identity();
    rotation += (1. - angle.cos()) * n * n.transpose();
    let mut temp: Matrix3<f64> = Matrix3::zeros();
    temp.m21 = n.z;
    temp.m31 = -n.y;
    temp.m12 = -n.z;
    temp.m13 = n.y;
    temp.m23 = -n.x;
    temp.m32 = n.x;
    rotation += angle.sin() * temp;
    let mut ans: Matrix4<f64> = Matrix4::zeros();
    for i in 0..2 {
        for j in 0..2 {
            ans[(i, j)] = rotation[(i, j)];
        }
    }
    ans[(3, 3)] = 1.;
    ans
}
