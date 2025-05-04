use std::fmt::Debug;
use nalgebra::{Matrix3, Vector3};
use std::fmt;
use egui::Color32;
use egui::ColorImage;
use egui::Vec2;
use egui::Widget;
use image::{Pixel, Rgb};

use super::WINDOW_ID;
use super::iris_color;
use super::iris_image_creation as iic;

const OKLAB_TOLERANCE:f32 = 0.01;

#[derive(Debug,PartialEq)]
pub enum ColorSpace {
    Rgb,
    CieLab,
    OkLab,
}

pub fn rgb_distance(col_a:Rgb<u8>,col_b:Rgb<u8>) -> f32{
    let r_a = col_a.channels()[0] as f32;
    let g_a = col_a.channels()[1] as f32;
    let b_a = col_a.channels()[2] as f32;

    let r_b = col_b.channels()[0] as f32;
    let g_b = col_b.channels()[1] as f32;
    let b_b = col_b.channels()[2] as f32;

    let dist = f32::sqrt(f32::powf(r_b - r_a,2.0) + f32::powf(g_b - g_a,2.0) + f32::powf(b_b - b_a,2.0));
    dist
}

#[allow(dead_code)]
pub fn rgb_distance_squared(col_a:Rgb<u8>,col_b:Rgb<u8>) -> f32{
    let r_a = col_a.channels()[0] as f32;
    let g_a = col_a.channels()[1] as f32;
    let b_a = col_a.channels()[2] as f32;

    let r_b = col_b.channels()[0] as f32;
    let g_b = col_b.channels()[1] as f32;
    let b_b = col_b.channels()[2] as f32;

    let dist = f32::powf(r_b - r_a,2.0) + f32::powf(g_b - g_a,2.0) + f32::powf(b_b - b_a,2.0);
    dist
}

pub fn color_display(ui: &mut egui::Ui,color: &mut AvarageRgb) -> egui::Response {
    if let Some(texture) = &color.texture {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP),|ui|{
            ui.set_width(texture.size()[0] as f32);
            let img_widget = egui::Image::from_texture(texture); 
            let img_size = img_widget.size().unwrap();
            let response = img_widget.sense(egui::Sense::CLICK).ui(ui)
                .on_hover_text(format!("r:{}|g:{}|b:{}",color.r,color.g,color.b));
            let min = egui::pos2(response.rect.min.x + img_size[0]/2.0 + 5.0,response.rect.min.y + img_size[1]/2.0);
            let target = egui::Rect{max:response.rect.max,min};
            ui.put(target,egui::Checkbox::without_text(&mut color.marked));
            if response.clicked() {
                color.color_info_window_open = true;
            }
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::Image,ui.is_enabled(),color.color_info_window_open,"Display Color")
            });
        }).response
    }else {
        let (_,response) = ui.allocate_exact_size(egui::vec2(0.0,0.0), egui::Sense::click());
        response
    }
}
pub fn color_display_percent(ui: &mut egui::Ui,color: &mut AvarageRgb,percent:f32) -> egui::Response {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP),|ui|{
            let response = color_display(ui, color);
            ui.label(format!("{}|{}|{} |=> {}%",color.r,color.g,color.b,percent*100.0));
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::Image,ui.is_enabled(),color.color_info_window_open,"Display Color plus extra data")
            });
            response
        }).response
}

pub struct AvarageRgb {
    pub r:u8,
    pub g:u8,
    pub b:u8,
    pub color_n:u32,
    pub texture: Option<egui::TextureHandle>,
    pub colors:Vec<AvarageRgb>,
    pub color_info_window_open:bool,
    pub id:usize,
    pub img:iic::HSLRect,
    pub img_rect:Option<egui::TextureHandle>,
    pub img_bar:Option<egui::TextureHandle>,
    pub img_dispaly_generated:bool,
    pub marked:bool,
    pub mark_every_color:bool,
    pub position:[u32;2],
}

impl Clone for AvarageRgb {
    fn clone(&self) -> Self {
        WINDOW_ID.with(|thread_id|{
            let r = self.r.clone();
            let g = self.g.clone();
            let b = self.b.clone();
            
            let color_n = self.color_n.clone();
            let texture = self.texture.clone();
            let colors = self.colors.clone();
            let color_info_window_open = false;

            let id = thread_id.get();
            thread_id.set(id+1);

            let img = self.img.clone();
            let img_rect = self.img_rect.clone();
            let img_bar = self.img_bar.clone();
            let img_dispaly_generated = self.img_dispaly_generated.clone();
            let marked = self.marked;
            let position = self.position;

            Self { 
                r,
                g,
                b,
                color_n,
                texture,
                colors,
                color_info_window_open,
                id,
                img,
                img_rect,
                img_bar,
                img_dispaly_generated,
                marked,
                mark_every_color:false,
                position,
            }
        })
    }
}

impl Debug for AvarageRgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       write!(f,"|r: {}|g: {}|b: {} |=> color_n: {}",self.r,self.g,self.b,self.color_n) 
    }
}


impl AvarageRgb {
    pub fn to_rgb(&self) -> Rgb<u8>{
        Rgb::from([self.r,self.g,self.b])
    }
    pub fn from_rgb(rgb:Rgb<u8>,position:[u32;2]) -> Self{

        WINDOW_ID.with(|thread_id|{

            let r = rgb.channels()[0];
            let g = rgb.channels()[1];
            let b = rgb.channels()[2];

            let id = thread_id.get();
            thread_id.set(id+1);
            let img = iic::HSLRect::new([128,64],iris_color::HSL::from_rgb(&rgb).h);
            AvarageRgb {
                r,
                g,
                b,
                color_n:1,
                texture:None,
                colors:vec![],
                color_info_window_open:false,
                id,
                img,
                img_bar: None,
                img_rect: None,
                img_dispaly_generated: false,
                marked: false,
                mark_every_color:false,
                position,
            }
        })
    }
    pub fn switch_to_most_saturated_color(&mut self,ui: &mut egui::Ui){
        let mut old_main = Self::from_rgb(Rgb::from([self.r,self.g,self.b]),self.position); 
        old_main.generate_texture(ui);
        self.colors.push(old_main);  
        self.colors.sort_by(|a,b|{
            let hsl_a = HSL::from_rgb(&a.to_rgb());
            let hsl_b = HSL::from_rgb(&b.to_rgb());

            if hsl_a.s == hsl_b.s {
                (0.5 - hsl_b.l).abs().partial_cmp(&(0.5 - hsl_a.l).abs()).unwrap()
            }else{
                hsl_a.s.partial_cmp(&hsl_b.s).unwrap()
            }
        });
        let new_main = self.colors.pop().unwrap();
        self.colors.append(&mut new_main.colors.clone());
        self.color_n = self.colors.len() as u32;
        self.r = new_main.r;
        self.g = new_main.g;
        self.b = new_main.b;
        self.generate_texture(ui);
        self.generate_color_display();

    }
    pub fn generate_color_display(&mut self) {
        let rgb = Rgb::from([self.r,self.g,self.b]);
        let marker = iic::RGBMarker::new(rgb,5,2);
        self.img.obj.push(marker);
        self.img.generate_h_bar();
        self.img.generate_sl_rect();
        self.img_dispaly_generated = true;
    }
    pub fn generate_texture(&mut self,ui: &mut egui::Ui) {
        self.texture = Some(ui.ctx().load_texture("color_text",ColorImage::new([32,32],Color32::from_rgb(self.r,self.g,self.b)),Default::default()));
    }

    pub fn _avarage(&mut self,comp: &AvarageRgb){
        self.color_n += comp.color_n;
        self.r = self.r.checked_add((comp.r as u32/self.color_n.max(1)).try_into().unwrap_or(255_u8)).unwrap_or(255);
        self.g = self.g.checked_add((comp.g as u32/self.color_n.max(1)).try_into().unwrap_or(255_u8)).unwrap_or(255);
        self.b = self.b.checked_add((comp.b as u32/self.color_n.max(1)).try_into().unwrap_or(255_u8)).unwrap_or(255);
        self.colors.push(comp.clone());
    }

    pub fn avarage_with_rgb(&mut self,comp: &Rgb<u8>,position:[u32;2]){

        let new_r = comp.channels()[0] as u32;
        let new_g = comp.channels()[1] as u32;
        let new_b = comp.channels()[2] as u32;

        let r = (self.r as u32).pow(2) * self.color_n; 
        let g = (self.g as u32).pow(2) * self.color_n; 
        let b = (self.b as u32).pow(2) * self.color_n; 

        self.r = 254.min(((r + new_r.pow(2))/(self.color_n+1)).isqrt())as u8;
        self.g = 254.min(((g + new_g.pow(2))/(self.color_n+1)).isqrt())as u8;
        self.b = 254.min(((b + new_b.pow(2))/(self.color_n+1)).isqrt())as u8;

        let  x = self.position[0] * self.color_n;
        let  y = self.position[1] * self.color_n;

        self.position[0] = (x + position[0])/(self.color_n+1);
        self.position[1] = (y + position[0])/(self.color_n+1);

        let difference = self.colors.contains(&AvarageRgb::from_rgb(*comp,position));
        if !difference {
            self.colors.push(AvarageRgb::from_rgb(*comp,position));
        }
        self.color_n += 1;

    }
    pub fn color_info_window_show(&mut self,ctx:&egui::Context){
        if self.color_info_window_open {
            if self.img_bar.is_none() && self.img_dispaly_generated{
                self.img_bar = Some(ctx.load_texture("img_bar",ColorImage::from_rgb([self.img.size[0].try_into().unwrap(),(self.img.size[1]/4).try_into().unwrap()],&self.img.img_bar),Default::default()));
            }
            if self.img_rect.is_none() && self.img_dispaly_generated{
                self.img_rect = Some(ctx.load_texture("img_rect",ColorImage::from_rgb([self.img.size[0].try_into().unwrap(),self.img.size[1].try_into().unwrap()],&self.img.img_rect),Default::default()));
            }
            let mut window_open = self.color_info_window_open;
            egui::Window::new(format!("{}|{}|{}",self.r,self.g,self.b)).id(egui::Id::new(self.id)).open(&mut window_open).show(ctx, |ui| {
                color_display(ui, self);
                if self.img_rect.is_none() && self.img_bar.is_none() {
                    self.generate_color_display();
                }
                ui.label(format!("Image position : {} / {}",self.position[0],self.position[1]));
                let rgb = Rgb::from([self.r,self.g,self.b]);
                ui.label(format!("RGB : {},{},{}",self.r,self.g,self.b));
                let hsl = HSL::from_rgb(&rgb);
                ui.label(format!("HSL : {:.2},{:.2},{:.2}",hsl.h,hsl.s,hsl.l));
                let ok_lab = OkLab::from_rgb(&rgb);
                let cie_lab = CieLab::from_rgb(rgb);
                ui.label(format!("OkLab : {:.2},{:.2},{:.2}",ok_lab.l,ok_lab.a,ok_lab.b));
                ui.label(format!("CieLab : {:.2},{:.2},{:.2}",cie_lab.l,cie_lab.a,cie_lab.b));
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT),|ui|{
                    if let Some(rect) = &self.img_rect {
                        ui.add(
                            egui::Image::from_texture(rect)
                        );
                    }else {
                        ui.label("No Color Display Texture found");
                    };
                    if let Some(bar) = &self.img_bar {
                        ui.add(
                            egui::Image::from_texture(bar)
                        );
                    }else {
                        ui.label("No Color Display Texture found");
                        if ui.button("Generate").clicked() {
                            self.generate_color_display();
                        }
                    };
                });
                if ui.checkbox(&mut self.mark_every_color,"Select every color").clicked(){
                    for c in &mut self.colors.iter_mut(){
                       c.marked = self.mark_every_color; 
                    }
                };
                egui::CollapsingHeader::new("Colors").show(ui,|ui|{
                    if self.colors.len() > 0 {
                        egui::ScrollArea::vertical().max_height(100.0).auto_shrink([false,true]).show(ui, |ui| {
                            let aw = ui.available_width();
                            egui::Grid::new("Colors").spacing(Vec2::new(0.0,3.0)).show(ui,|ui|{
                                let mut column_count = 0;
                                for c in &mut self.colors{
                                    color_display(ui, c);
                                    column_count += 1;
                                    if column_count > (aw/(ui.available_width()+3.0)) as i32 {
                                        ui.end_row();
                                        column_count = 0;
                                    }
                                }
                            });
                        });
                    }
                });
            });
            self.color_info_window_open = window_open;
            for w in &mut self.colors{
                w.color_info_window_show(ctx);
            }
        }
    }
}

impl fmt::Display for AvarageRgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"({},{},{})",self.r,self.g,self.b)
    }
}

impl PartialEq for AvarageRgb {
    fn eq(&self, other: &Self) -> bool {
        rgb_distance(self.to_rgb(),other.to_rgb()) <= 1.0
    }
}

pub struct XYZ {
    x:f32,
    y:f32,
    z:f32,
}

impl XYZ {
    pub fn from_rgb(rgb:&Rgb<u8>) -> Self {
        let mut r = rgb.channels()[0] as f32 /255.0;
        let mut g = rgb.channels()[1] as f32 /255.0;
        let mut b = rgb.channels()[2] as f32 /255.0;

        r = if r > 0.04045 {((r + 0.055)/1.055).powf(2.4)} else {r/12.92};
        g = if g > 0.04045 {((g + 0.055)/1.055).powf(2.4)} else {g/12.92};
        b = if b > 0.04045 {((b + 0.055)/1.055).powf(2.4)} else {b/12.92};

        r = r*100.0;
        g = g*100.0;
        b = b*100.0;

        let x = r * 0.4124 + g * 0.3576 + b * 0.1805;
        let y = r * 0.2126 + g * 0.7152 + b * 0.0722;
        let z = r * 0.0193 + g * 0.1192 + b * 0.9505;
        Self{x,y,z}
    }
}

pub const XYZ_D65:XYZ = XYZ{x:95.047,y:100.0,z:108.883};

pub struct CieLab {
    l:f32,
    a:f32,
    b:f32,
}

impl CieLab {
    pub fn new(l:f32,a:f32,b:f32)-> Self{
        Self{l,a,b}
    } 

    #[allow(dead_code)]
    pub fn distance_to_lab_squared(&self,comp:&CieLab) -> f32 {
        (self.l - comp.l).powf(2.0)+(self.a - comp.a).powf(2.0)+(self.b - comp.b).powf(2.0)
    }
    pub fn distance_to_lab(&self,comp:&CieLab) -> f32 {
        ((self.l - comp.l).powf(2.0)+(self.a - comp.a).powf(2.0)+(self.b - comp.b).powf(2.0)).sqrt()
    }

    pub fn from_xyz(xyz:&XYZ) -> Self{
        let mut var_x = xyz.x/XYZ_D65.x;
        let mut var_y = xyz.y/XYZ_D65.y;
        let mut var_z = xyz.z/XYZ_D65.z;

        var_x = if var_x > 0.008856 {var_x.powf(1.0/3.0)} else {(7.787 * var_x)+(16.0/116.0)};
        var_y = if var_y > 0.008856 {var_y.powf(1.0/3.0)} else {(7.787 * var_y)+(16.0/116.0)};
        var_z = if var_z > 0.008856 {var_z.powf(1.0/3.0)} else {(7.787 * var_z)+(16.0/116.0)};

        let cie_l = (116.0 * var_y) - 16.0;
        let cie_a = 500.0 *(var_x - var_y);
        let cie_b = 200.0 *(var_y - var_z);

        Self::new(cie_l,cie_a,cie_b)
    }
    pub fn from_rgb(rgb:Rgb<u8>)-> Self{
        Self::from_xyz(&XYZ::from_rgb(&rgb))
    }
}

impl fmt::Display for CieLab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"({},{},{})",self.l,self.a,self.b)
    }
}
#[derive(Clone)]
pub struct OkLab{
    pub l:f32,
    pub a:f32,
    pub b:f32,
}

impl PartialEq for OkLab {
  fn eq(&self, comp: &Self) -> bool {
        let delta = ((self.l - comp.l).powf(2.0)+(self.a - comp.a).powf(2.0)+(self.b - comp.b).powf(2.0)).sqrt();
        delta <= OKLAB_TOLERANCE
  }  
}
fn gamma_expand(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
fn gamma_compress(c: f64) -> f64 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}
impl OkLab {
    pub fn to_u32(&self) -> [u32;3] {
        [(self.l * 100.0) as u32,((self.a + 1.0) * 100.0) as u32, (self.b * 100.0) as u32]
    }
    #[allow(dead_code)]
    pub fn to_xyz(&self) -> XYZ {
        let mut l = 0.99999999845051981432 * self.l + 0.39633779217376785678 * self.a + 0.21580375806075880339 * self.b; 
        let mut m = 1.0000000088817607767 * self.l - 0.1055613423236563494  * self.a - 0.063854174771705903402 * self.b;
        let mut s = 1.0000000546724109177 * self.l - 0.089484182094965759684 * self.a - 1.2914855378640917399 * self.b;

        l = l.powi(3);
        m = m.powi(3);
        s = s.powi(3);

        let x = 1.227013851103521026 * l + -0.5577999806518222383 * m + 0.28125614896646780758 * s;
        let y = -0.040580178423280593977 * l + 1.1122568696168301049 * m + -0.071676678665601200577 * s;
        let z = -0.07638128450570689287 * l + -0.42148197841801273054 * m + 1.5861632204407947575 * s;

        XYZ{x,y,z}
    }
    pub fn to_rgb(&self) -> [u8;3] {
    let m2_inv = Matrix3::new(
             1.0000000,  0.3963378,  0.2158038,
             1.0000000, -0.1055613, -0.0638542,
             1.0000000, -0.0894842, -1.2914855,
        );

        let m1_inv = Matrix3::new(
             1.2270139, -0.5577999,  0.2812561,
            -0.0405802,  1.1122569, -0.0716767,
            -0.0763813, -0.4214819,  1.5861632,
        );

        let lab = Vector3::new(self.l, self.a, self.b);
        let lms_cbrt = m2_inv * lab;

        let lms = lms_cbrt.map(|v| v * v * v);

        let xyz = m1_inv * lms;

        let xyz_to_rgb = Matrix3::new(
             3.2404542, -1.5371385, -0.4985314,
            -0.9692660,  1.8760108,  0.0415560,
             0.0556434, -0.2040259,  1.0572252,
        );
        let rgb_lin = xyz_to_rgb * xyz;

        let r = gamma_compress(rgb_lin.x.into()).clamp(0.0, 1.0);
        let g = gamma_compress(rgb_lin.y.into()).clamp(0.0, 1.0);
        let b = gamma_compress(rgb_lin.z.into()).clamp(0.0, 1.0);

        [(r * 255.0).round() as u8,
         (g * 255.0).round() as u8,
         (b * 255.0).round() as u8]

    }
    pub fn from_u32(value:[u32;3]) -> Self {
        Self::new(value[0] as f32/100.0, value[1] as f32/100.0 -1.0,value[2] as f32 / 100.0)
    }
    pub fn new(l:f32,a:f32,b:f32) -> Self {
        OkLab{l,a,b}
    }
    pub fn add(&mut self, other:&Self){
        self.l += other.l;
        self.a += other.a;
        self.b += other.b;
    }
    pub fn diff(&mut self, diff:f32){
        self.l /= diff;
        self.a /= diff;
        self.b /= diff;
    }
    #[allow(dead_code)]
    pub fn from_xyz(xyz:&XYZ) -> Self{
        let m1 = Matrix3::new(
            0.8189330101, 0.3618667424, -0.1288597137,
            0.0329845436, 0.9293118715,  0.0361456387,
            0.0482003018, 0.2643662691,  0.6338517070,
        );

        let m2 = Matrix3::new(
             0.2104542553,  0.7936177850, -0.0040720468,
             1.9779984951, -2.4285922050,  0.4505937099,
             0.0259040371,  0.7827717662, -0.8086757660,
        );

        let xyz_mat = Vector3::new(xyz.x,xyz.y, xyz.z); // Example input

        let lms = m1 * xyz_mat;

        let lms_cbrt = lms.map(|v| v.cbrt());

        let lab = m2 * lms_cbrt;
        Self::new(lab.x, lab.y, lab.z)
    }
    pub fn from_rgb(rgb:&Rgb<u8>) -> Self {
        
        let r = rgb.0[0] as f64 / 255.0;
        let g = rgb.0[1] as f64 / 255.0;
        let b = rgb.0[2] as f64 / 255.0;

        let r_lin = gamma_expand(r);
        let g_lin = gamma_expand(g);
        let b_lin = gamma_expand(b);

        let rgb_to_xyz = Matrix3::new(
            0.4124564, 0.3575761, 0.1804375,
            0.2126729, 0.7151522, 0.0721750,
            0.0193339, 0.1191920, 0.9503041,
        );
        let xyz = rgb_to_xyz * Vector3::new(r_lin, g_lin, b_lin);

        let m1 = Matrix3::new(
            0.8189330101, 0.3618667424, -0.1288597137,
            0.0329845436, 0.9293118715,  0.0361456387,
            0.0482003018, 0.2643662691,  0.6338517070,
        );

        let m2 = Matrix3::new(
             0.2104542553,  0.7936177850, -0.0040720468,
             1.9779984951, -2.4285922050,  0.4505937099,
             0.0259040371,  0.7827717662, -0.8086757660,
        );

        let lms = m1 * xyz;
        let lms_cbrt = lms.map(|v| v.cbrt());
        let lab = m2 * lms_cbrt;

        Self::new(lab.x as f32, lab.y as f32, lab.z as f32)
    }
    #[allow(dead_code)]
    pub fn distance_to_lab_squared(&self,comp:&OkLab) -> f32 {
        (self.l - comp.l).powf(2.0)+(self.a - comp.a).powf(2.0)+(self.b - comp.b).powf(2.0)
    }
    pub fn distance_to_lab(&self,comp:&OkLab) -> f32 {
        ((self.l - comp.l).powf(2.0)+(self.a - comp.a).powf(2.0)+(self.b - comp.b).powf(2.0)).sqrt()
    }
}

impl fmt::Display for OkLab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"({},{},{})",self.l,self.a,self.b)
    }
}
#[derive(Clone)]
pub struct HSL {
    pub h:f32,
    pub s:f32,
    pub l:f32,
}

#[derive(Debug,PartialEq)]
pub enum HSLDistanceType {
    #[allow(dead_code)]
    Hue,
    #[allow(dead_code)]
    Saturation,
    #[allow(dead_code)]
    Lightness,
}

impl HSL {
    pub fn new(h:f32,s:f32,l:f32) -> Self {
        Self{h,s,l}
    }
    pub fn from_rgb(rgb:&Rgb<u8>) -> Self {
        let r = rgb.channels()[0] as f32 /255.0;
        let g = rgb.channels()[1] as f32 /255.0;
        let b = rgb.channels()[2] as f32 /255.0;

        let v_max = r.max(g.max(b));
        let v_min = r.min(g.min(b));

        let delta = v_max - v_min;

        let mut h:f32 = 0.0;

        if delta.abs() <= 0.0001 {
            h = 0.0;
        }else{
            if (v_max - r).abs() <= 0.0001{
                h = 60.0 * (((g-b)/delta)%6.0);
                if h < 0.0 {
                    h = h + 360.0;
                }
            }

            if (v_max - g).abs() <= 0.0001{
                h = 60.0 * (((b-r)/delta)+2.0);
            }

            if (v_max - b).abs() <= 0.0001{
                h = 60.0 * (((r-g)/delta)+4.0);
            }
        }

        let l = (v_max+v_min) / 2.0;

        let s:f32;

        if delta.abs() <= 0.0001{
            s = 0.0;
        }else {
            s = delta/(1.0-(2.0*l-1.0).abs());
        }
        
        
        Self{h,s,l}
    }
    pub fn to_rgb(&self) -> Rgb<u8> {
        let c = (1.0-(2.0*self.l-1.0).abs()) * self.s;
        let x = c * (1.0-(self.h/60.0%2.0-1.0).abs());
        let m = self.l - c/2.0;

        let (r,g,b) = match (self.h/60.0).floor() {
            0.0 => (c,x,0.0),
            1.0 => (x,c,0.0),
            2.0 => (0.0,c,x),
            3.0 => (0.0,x,c),
            4.0 => (x,0.0,c),
            _   => (c,0.0,x),
        };

        Rgb::from([
                    ((r+m) * 255.0) as u8,
                    ((g+m) * 255.0) as u8,
                    ((b+m) * 255.0) as u8
        ])
    }
    #[allow(dead_code)]
    pub fn hue_distance(&self,hsl:&HSL) -> f32 {
        (self.h.powf(2.0) - hsl.h.powf(2.0)).sqrt() 
    }
    #[allow(dead_code)]
    pub fn saturation_distance(&self,hsl:&HSL) -> f32 {
        (self.s.powf(2.0) - hsl.s.powf(2.0)).sqrt() 
    }
    #[allow(dead_code)]
    pub fn lightness_distance(&self,hsl:&HSL) -> f32 {
        (self.l.powf(2.0) - hsl.l.powf(2.0)).sqrt() 
    }
    
}

impl fmt::Display for HSL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"({},{},{})",self.h,self.s,self.l)
    }
}
