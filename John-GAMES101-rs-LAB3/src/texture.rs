use nalgebra::Vector3;
use opencv::core::{MatTraitConst, VecN};
use opencv::imgcodecs::{imread, IMREAD_COLOR};

pub struct Texture {
    pub img_data: opencv::core::Mat,
    pub width: usize,
    pub height: usize,
}

impl Texture {
    pub fn new(name: &str) -> Self {
        let img_data = imread(name, IMREAD_COLOR).expect("Image reading error!");
        let width = img_data.cols() as usize;
        let height = img_data.rows() as usize;
        Texture {
            img_data,
            width,
            height,
        }
    }

    pub fn get_color(&self, mut u: f64, mut v: f64) -> Vector3<f64> {
        if u < 0.0 {
            u = 0.0;
        }
        if u > 1.0 {
            u = 1.0;
        }
        if v < 0.0 {
            v = 0.0;
        }
        if v > 1.0 {
            v = 1.0;
        }

        let u_img = u * self.width as f64;
        let v_img = (1.0 - v) * self.height as f64;
        let color: &VecN<u8, 3> = self.img_data.at_2d(v_img as i32, u_img as i32).unwrap();

        Vector3::new(color[2] as f64, color[1] as f64, color[0] as f64)
    }

    fn read_color(&self, u_img: i32, v_img: i32) -> Vector3<f64> {
        let color: &VecN<u8, 3> = self.img_data.at_2d(v_img, u_img).unwrap();

        Vector3::new(color[2] as f64, color[1] as f64, color[0] as f64)
    }

    pub fn getColorBilinear(&self, mut u: f64, mut v: f64) -> Vector3<f64> {
        // 在此实现双线性插值函数, 并替换掉get_color
        if u < 0.0 {
            u = 0.0;
        }
        if u > 1.0 {
            u = 1.0;
        }
        if v < 0.0 {
            v = 0.0;
        }
        if v > 1.0 {
            v = 1.0;
        }

        let u_img = u * self.width as f64;
        let v_img = (1.0 - v) * self.height as f64;
        let s_line: i32;
        let b_line: i32;
        if u_img < 0.5 || v_img < 0.5 || u_img + 0.5 > self.width as f64 || v_img + 0.5 > self.height as f64 {
            return self.get_color(u, v);
        }
        if u_img-u_img.floor() < 0.5 {
            b_line = u_img.floor() as i32;
            s_line = (b_line - 1);
        } else {
            s_line = u_img.floor() as i32;
            b_line = s_line + 1;
        }
        let s_col: i32;
        let b_col: i32;
        if v_img-v_img.floor() < 0.5 {
            b_col = v_img.floor() as i32;
            s_col = b_col - 1;
        } else {
            s_col = v_img.floor() as i32;
            b_col = s_col + 1;
        }
        let t = u_img - s_line as f64 + 0.5;
        let s = v_img - s_col as f64 + 0.5;

        let u0 = Self::lerp(s, self.read_color(s_line, s_col), self.read_color(b_line, s_col));
        let u1 = Self::lerp(s, self.read_color(s_line, b_col), self.read_color(b_line, b_col));

        Self::lerp(t, u0, u1)
    }

    fn lerp(x: f64, v0:Vector3<f64>, v1:Vector3<f64>) -> Vector3<f64> {
        v0 + x * (v1-v0)
    }
}
