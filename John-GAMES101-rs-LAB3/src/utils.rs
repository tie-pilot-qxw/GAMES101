use crate::shader::{FragmentShaderPayload, VertexShaderPayload};
use crate::texture::Texture;
use crate::triangle::Triangle;
use nalgebra::{Matrix3, Matrix4, Vector3, Vector4};
use opencv::core::{Mat, MatTraitConst};
use opencv::imgproc::{cvt_color, COLOR_RGB2BGR};
use opencv::stitching::Detail_MatchesInfoTraitConst;
use std::f64::consts::PI;
use std::os::raw::c_void;

type V3f = Vector3<f64>;
type M4f = Matrix4<f64>;

pub(crate) fn get_view_matrix(eye_pos: V3f) -> M4f {
    let mut view: M4f = Matrix4::identity();
    view[(0, 3)] = -eye_pos[0];
    view[(1, 3)] = -eye_pos[1];
    view[(2, 3)] = -eye_pos[2];

    view
}

pub(crate) fn get_model_matrix(rotation_angle: f64) -> M4f {
    let mut model: M4f = Matrix4::identity();
    let rad = rotation_angle.to_radians();
    model[(0, 0)] = rad.cos();
    model[(2, 2)] = model[(0, 0)];
    model[(0, 2)] = rad.sin();
    model[(2, 0)] = -model[(0, 2)];
    let mut scale: M4f = Matrix4::identity();
    scale[(0, 0)] = 2.5;
    scale[(1, 1)] = 2.5;
    scale[(2, 2)] = 2.5;
    model * scale
}

pub(crate) fn get_projection_matrix(
    mut eye_fov: f64,
    aspect_ratio: f64,
    z_near: f64,
    z_far: f64,
) -> M4f {
    // let mut persp2ortho: M4f = Matrix4::zeros();
    /*  Implement your code here  */
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
    // persp2ortho
}

pub(crate) fn frame_buffer2cv_mat(frame_buffer: &Vec<V3f>) -> Mat {
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

pub fn load_triangles(obj_file: &str) -> Vec<Triangle> {
    let (models, _) = tobj::load_obj(&obj_file, &tobj::LoadOptions::default()).unwrap();
    let mesh = &models[0].mesh;
    let n = mesh.indices.len() / 3;
    let mut triangles = vec![Triangle::default(); n];

    // 遍历模型的每个面
    for vtx in 0..n {
        let rg = vtx * 3..vtx * 3 + 3;
        let idx: Vec<_> = mesh.indices[rg.clone()]
            .iter()
            .map(|i| *i as usize)
            .collect();

        // 记录图形每个面中连续三个顶点（小三角形）
        for j in 0..3 {
            let v = &mesh.positions[3 * idx[j]..3 * idx[j] + 3];
            triangles[vtx].set_vertex(j, Vector4::new(v[0] as f64, v[1] as f64, v[2] as f64, 1.0));
            let ns = &mesh.normals[3 * idx[j]..3 * idx[j] + 3];
            triangles[vtx].set_normal(j, Vector3::new(ns[0] as f64, ns[1] as f64, ns[2] as f64));
            let tex = &mesh.texcoords[2 * idx[j]..2 * idx[j] + 2];
            triangles[vtx].set_tex_coord(j, tex[0] as f64, tex[1] as f64);
        }
    }
    triangles
}

// 选择对应的Shader
pub fn choose_shader_texture(
    method: &str,
    obj_path: &str,
) -> (fn(&FragmentShaderPayload) -> Vector3<f64>, Option<Texture>) {
    let mut active_shader: fn(&FragmentShaderPayload) -> Vector3<f64> = phong_fragment_shader;
    let mut tex = None;
    if method == "normal" {
        println!("Rasterizing using the normal shader");
        active_shader = normal_fragment_shader;
    } else if method == "texture" {
        println!("Rasterizing using the normal shader");
        active_shader = texture_fragment_shader;
        tex = Some(Texture::new(&(obj_path.to_owned() + "spot_texture.png")));
    } else if method == "phong" {
        println!("Rasterizing using the phong shader");
        active_shader = phong_fragment_shader;
    } else if method == "bump" {
        println!("Rasterizing using the bump shader");
        active_shader = bump_fragment_shader;
    } else if method == "displacement" {
        println!("Rasterizing using the displacement shader");
        active_shader = displacement_fragment_shader;
    }
    (active_shader, tex)
}

pub fn vertex_shader(payload: &VertexShaderPayload) -> V3f {
    payload.position
}

#[derive(Default)]
struct Light {
    pub position: V3f,
    pub intensity: V3f,
}

pub fn normal_fragment_shader(payload: &FragmentShaderPayload) -> V3f {
    let result_color = (payload.normal.xyz().normalize() + Vector3::new(1.0, 1.0, 1.0)) / 2.0;
    result_color * 255.0
}

pub fn phong_fragment_shader(payload: &FragmentShaderPayload) -> V3f {
    // 泛光、漫反射、高光系数
    let ka = Vector3::new(0.005, 0.005, 0.005);
    let kd = payload.color;
    let ks = Vector3::new(0.7937, 0.7937, 0.7937);

    // 灯光位置和强度
    let l1 = Light {
        position: Vector3::new(20.0, 20.0, 20.0),
        intensity: Vector3::new(500.0, 500.0, 500.0),
    };
    let l2 = Light {
        position: Vector3::new(-20.0, 20.0, 0.0),
        intensity: Vector3::new(500.0, 500.0, 500.0),
    };
    let lights = vec![l1, l2];
    let amb_light_intensity = Vector3::new(10.0, 10.0, 10.0);
    let eye_pos = Vector3::new(0.0, 0.0, 10.0);

    let p = 150.0;

    // ping point的信息
    let normal = payload.normal.normalize();
    let point = payload.view_pos;
    let color = payload.color;

    let mut result_color = Vector3::zeros(); // 保存光照结果

    // <遍历每一束光>
    for light in lights {
        // TODO: For each light source in the code, calculate what the *ambient*, *diffuse*, and *specular*
        // components are. Then, accumulate that result on the *result_color* object.

        let l = light.position - point;
        let len_s = l.norm_squared();
        let cosine = 0_f64.max(normal.dot(&l.normalize()));

        // Lambertian Term
        result_color += kd.component_mul(&(light.intensity / len_s)) * cosine;

        // Specular Term
        let v = eye_pos - point;
        let h = (v + l).normalize();
        let cosine = 0_f64.max(normal.dot(&h)).powf(p);
        result_color += ks.component_mul(&(light.intensity / len_s)) * cosine;
    }

    // Ambient Term
    result_color += ka.component_mul(&amb_light_intensity);

    result_color * 255.0
}

pub fn texture_fragment_shader(payload: &FragmentShaderPayload) -> V3f {
    let ka = Vector3::new(0.005, 0.005, 0.005);
    let texture_color: Vector3<f64> = match &payload.texture {
        // TODO: Get the texture value at the texture coordinates of the current fragment
        // <获取材质颜色信息>
        None => Vector3::new(0.0, 0.0, 0.0),
        Some(texture) => payload.texture.as_ref().unwrap().getColorBilinear(payload.tex_coords[0], payload.tex_coords[1]), // Do modification here
    };
    let kd = texture_color / 255.0; // 材质颜色影响漫反射系数
    let ks = Vector3::new(0.7937, 0.7937, 0.7937);

    let l1 = Light {
        position: Vector3::new(20.0, 20.0, 20.0),
        intensity: Vector3::new(500.0, 500.0, 500.0),
    };
    let l2 = Light {
        position: Vector3::new(-20.0, 20.0, 0.0),
        intensity: Vector3::new(500.0, 500.0, 500.0),
    };
    let lights = vec![l1, l2];
    let amb_light_intensity = Vector3::new(10.0, 10.0, 10.0);
    let eye_pos = Vector3::new(0.0, 0.0, 10.0);

    let p = 150.0;

    let color = texture_color;
    let point = payload.view_pos;
    let normal = payload.normal;

    let mut result_color = Vector3::zeros();

    for light in lights {
        // TODO: For each light source in the code, calculate what the *ambient*, *diffuse*, and *specular*
        // components are. Then, accumulate that result on the *result_color* object.

        let l = light.position - point;
        let len_s = l.norm_squared();
        let cosine = 0_f64.max(normal.normalize().dot(&l.normalize()));

        // Lambertian Term
        result_color += kd.component_mul(&(light.intensity / len_s)) * cosine;

        // Specular Term
        let v = eye_pos - point;
        let h = (v + l).normalize();
        let cosine = 0_f64.max(normal.normalize().dot(&h)).powf(p);
        result_color += ks.component_mul(&(light.intensity / len_s)) * cosine;
        
        // Ambient Term
        result_color += ka.component_mul(&amb_light_intensity);
    }

    result_color * 255.0
}

pub fn bump_fragment_shader(payload: &FragmentShaderPayload) -> V3f {
    let ka = Vector3::new(0.005, 0.005, 0.005);
    let kd = payload.color;
    let ks = Vector3::new(0.7937, 0.7937, 0.7937);

    let l1 = Light {
        position: Vector3::new(20.0, 20.0, 20.0),
        intensity: Vector3::new(500.0, 500.0, 500.0),
    };
    let l2 = Light {
        position: Vector3::new(-20.0, 20.0, 0.0),
        intensity: Vector3::new(500.0, 500.0, 500.0),
    };
    let lights = vec![l1, l2];
    let amb_light_intensity = Vector3::new(10.0, 10.0, 10.0);
    let eye_pos = Vector3::new(0.0, 0.0, 10.0);

    let p = 150.0;

    let normal = payload.normal.normalize();
    let point = payload.view_pos;
    let color = payload.color;

    let (kh, kn) = (0.2, 0.1);

    // TODO: Implement bump mapping here
    // Let n = normal = (x, y, z)
    // Vector t = (x*y/sqrt(x*x+z*z),sqrt(x*x+z*z),z*y/sqrt(x*x+z*z))
    // Vector b = n cross product t
    // Matrix TBN = [t b n]
    // dU = kh * kn * (h(u+1/w,v)-h(u,v))
    // dV = kh * kn * (h(u,v+1/h)-h(u,v))
    // Vector ln = (-dU, -dV, 1)
    // Normal n = normalize(TBN * ln)

    let (u,v) = (payload.tex_coords[0], payload.tex_coords[1]);
    let (w,h) = (payload.texture.as_ref().unwrap().width,payload.texture.as_ref().unwrap().height);

    let (x,y,z) = (normal.x, normal.y, normal.z);
    let t = Vector3::new(x*y/(x*x+z*z).sqrt(), (x*x+z*z).sqrt(),z*y/(x*x+z*z).sqrt());
    let b = normal.cross(&t);
    let mut TBN = Matrix3::zeros();
    TBN.m11 = t[0];TBN.m21 = t[1];TBN.m31 = t[2];
    TBN.m12 = b[0];TBN.m22 = b[1];TBN.m32 = b[2];
    TBN.m13 = normal[0];TBN.m23 = normal[1];TBN.m33 = normal[2];
    let du = kh * kn * (payload.texture.as_ref().unwrap().getColorBilinear(u + 1./w as f64, v).norm() - payload.texture.as_ref().unwrap().getColorBilinear(u, v).norm());
    let dv = kh * kn * (payload.texture.as_ref().unwrap().getColorBilinear(u, v + 1. / h as f64).norm() - payload.texture.as_ref().unwrap().getColorBilinear(u, v).norm());
    let ln = Vector3::new(-du, -dv, 1.);
    let n = (TBN * ln).normalize();

    let mut result_color = Vector3::zeros();
    result_color = n;

    result_color * 255.0
}

pub fn displacement_fragment_shader(payload: &FragmentShaderPayload) -> V3f {
    let ka = Vector3::new(0.005, 0.005, 0.005);
    let kd = payload.color;
    let ks = Vector3::new(0.7937, 0.7937, 0.7937);

    let l1 = Light {
        position: Vector3::new(20.0, 20.0, 20.0),
        intensity: Vector3::new(500.0, 500.0, 500.0),
    };
    let l2 = Light {
        position: Vector3::new(-20.0, 20.0, 0.0),
        intensity: Vector3::new(500.0, 500.0, 500.0),
    };
    let lights = vec![l1, l2];
    let amb_light_intensity = Vector3::new(10.0, 10.0, 10.0);
    let eye_pos = Vector3::new(0.0, 0.0, 10.0);

    let p = 150.0;

    let normal = payload.normal;
    let point = payload.view_pos;
    let color = payload.color;

    let (kh, kn) = (0.2, 0.1);

    // TODO: Implement displacement mapping here
    // Let n = normal = (x, y, z)
    // Vector t = (x*y/sqrt(x*x+z*z),sqrt(x*x+z*z),z*y/sqrt(x*x+z*z))
    // Vector b = n cross product t
    // Matrix TBN = [t b n]
    // dU = kh * kn * (h(u+1/w,v)-h(u,v))
    // dV = kh * kn * (h(u,v+1/h)-h(u,v))
    // Vector ln = (-dU, -dV, 1)
    // Position p = p + kn * n * h(u,v)
    // Normal n = normalize(TBN * ln)
    let (u,v) = (payload.tex_coords[0], payload.tex_coords[1]);
    let (w,h) = (payload.texture.as_ref().unwrap().width,payload.texture.as_ref().unwrap().height);

    let (x,y,z) = (normal.x, normal.y, normal.z);
    let t = Vector3::new(x*y/(x*x+z*z).sqrt(), (x*x+z*z).sqrt(),z*y/(x*x+z*z).sqrt());
    let b = normal.cross(&t);
    let mut TBN = Matrix3::zeros();
    TBN.m11 = t[0];TBN.m21 = t[1];TBN.m31 = t[2];
    TBN.m12 = b[0];TBN.m22 = b[1];TBN.m32 = b[2];
    TBN.m13 = normal[0];TBN.m23 = normal[1];TBN.m33 = normal[2];
    let du = kh * kn * (payload.texture.as_ref().unwrap().getColorBilinear(u + 1./w as f64, v).norm() - payload.texture.as_ref().unwrap().getColorBilinear(u, v).norm());
    let dv = kh * kn * (payload.texture.as_ref().unwrap().getColorBilinear(u, v + 1. / h as f64).norm() - payload.texture.as_ref().unwrap().getColorBilinear(u, v).norm());
    let ln = Vector3::new(-du, -dv, 1.);
    let point = point + kn * normal * payload.texture.as_ref().unwrap().getColorBilinear(u, v).norm();
    let normal = (TBN * ln).normalize();

    let mut result_color = Vector3::zeros();
    for light in lights {
        // TODO: For each light source in the code, calculate what the *ambient*, *diffuse*, and *specular*
        // components are. Then, accumulate that result on the *result_color* object.

        let l = light.position - point;
        let len_s = l.norm_squared();
        let cosine = 0_f64.max(normal.normalize().dot(&l.normalize()));

        // Lambertian Term
        result_color += kd.component_mul(&(light.intensity / len_s)) * cosine;

        // Specular Term
        let v = eye_pos - point;
        let h = (v + l).normalize();
        let cosine = 0_f64.max(normal.normalize().dot(&h)).powf(p);
        result_color += ks.component_mul(&(light.intensity / len_s)) * cosine;
        
        // Ambient Term
        result_color += ka.component_mul(&amb_light_intensity);
    }

    result_color * 255.0
}
