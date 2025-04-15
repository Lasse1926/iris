use image::{Pixel, Rgb, RgbImage};

use super::WINDOW_ID;
use super::iris_color;

#[derive(Default)]
pub struct ImageCreator{
    id:usize,
    pub open:bool,
}

impl ImageCreator {
    pub fn new() -> Self {
        WINDOW_ID.with(|thread_id|{

            let id = thread_id.get();
            thread_id.set(id+1);
            
            Self{
                id,
                open:true,
            }
        })
    }
    pub fn show(&mut self,ctx:&egui::Context){
        if self.open {
            let mut window_open = self.open;
            egui::Window::new("ImageCreator").id(egui::Id::new(self.id)).open(&mut window_open).show(ctx,|ui|{
                if ui.add(egui::Button::new("gen")).clicked() {
                    let mut rect = HSLRect::new([640,640],313.0); 
                    let mark = RGBMarker::new(Rgb::from([129,50,112]),20,5);
                    let mark1 = RGBMarker::new(Rgb::from([255, 122, 226]),20,5);
                    let mark2 = RGBMarker::new(Rgb::from([255, 0, 200]),20,5);
                    rect.obj.push(mark);
                    rect.obj.push(mark1);
                    rect.obj.push(mark2);
                    rect.generate_sl_rect();
                }
            });
            self.open = window_open;
        }
    }
}

pub struct RGBRect;

impl RGBRect {
    
    pub fn rgb_rect_x(x:f32) -> [f32;3] {
        let r = (-9.0*x.powf(2.0) + 3.0*x + 0.75).max(0.0);
        let g = (-9.0*x.powf(2.0) + 9.0*x - 1.25).max(0.0);
        let b = (-9.0*x.powf(2.0) + 15.0*x - 5.25).max(0.0);

        let b_2 = (-9.0*x.powf(2.0) -3.0*x + 0.75).max(0.0); 
        let r_2 = (-9.0*x.powf(2.0) + 21.0*x - 11.25).max(0.0);

        [r + r_2,g,b + b_2]
    }
    pub fn generate_image() {
        let mut img = RgbImage::new(64,64);
        for x in 0..64 {
            let rgb = Self::rgb_rect_x(x as f32 / 64.0);  

            let mut rgb_u8:[u8;3]=[0,0,0];

            for y in 0..64 {

                rgb_u8[0] = (rgb[0] * 255.0) as u8;
                rgb_u8[1] = (rgb[2] * 255.0) as u8;
                rgb_u8[2] = (rgb[1] * 255.0) as u8;

                if y > 32 {
                    let step:f32  = 1.0 - ((y as f32 - 32.0)/32.0);
                    rgb_u8[0] = (rgb_u8[0] as f32 * step) as u8;
                    rgb_u8[1] = (rgb_u8[1] as f32 * step) as u8;
                    rgb_u8[2] = (rgb_u8[2] as f32 * step) as u8;
                }
                if y < 32 {
                    let step:f32  = f32::abs((y as f32 - 32.0)/32.0);
                    rgb_u8[0] = (rgb_u8[0] as f32 + (255 - rgb_u8[0])as f32 * step) as u8;
                    rgb_u8[1] = (rgb_u8[1] as f32 + (255 - rgb_u8[1])as f32 * step) as u8;
                    rgb_u8[2] = (rgb_u8[2] as f32 + (255 - rgb_u8[2])as f32 * step) as u8;
                }
                img.put_pixel(x, y, Rgb(rgb_u8));
            }
        }
        let _ = img.save("./created_images/rgb_rect.png");
    }
}
pub trait Draw {
    fn draw_rect(&mut self,parent:&mut HSLRect); 
    fn draw_bar(&mut self,parent:&mut HSLRect); 
}

pub struct HSLRect{
    img:RgbImage,
    size:[u32;2],
    obj: Vec<RGBMarker>,
    hue:f32,
}

impl HSLRect {
    pub fn new(size:[u32;2],hue:f32) -> Self {
        let img = RgbImage::new(size[0],size[1]);
        HSLRect{size,obj:vec![],hue,img}
    }
    pub fn generate_sl_rect(&mut self){
        for x in 0..self.size[0] {
            for y in 0..self.size[1]{
                self.img.put_pixel(x, y,self.pos_to_rgb_rect([x,y]));
            }
        }
        let mut clone_obj = self.obj.clone();
        for m in clone_obj.iter_mut() {
            m.draw_rect(self);
        }
        let _ = self.img.save("./created_images/HSL_saturation_lightness_rect.png");
    }
    pub fn pos_to_rgb_rect(&self,pos:[u32;2]) -> Rgb<u8> {
        let s = pos[0] as f32/self.size[0] as f32;
        let l = pos[1] as f32/self.size[1] as f32;
        let hsl = iris_color::HSL::new(self.hue,s,l);
        hsl.to_rgb()
    }
    pub fn rgb_color_to_position_rect(&self,rgb:&Rgb<u8>) -> [u32;2] {
        let hsl = iris_color::HSL::from_rgb(rgb); 
        let x = hsl.s * self.size[0]as f32;
        let y = hsl.l * self.size[1]as f32;
        [x as u32,y as u32]
    }
    pub fn generate_h_bar(&self){
        let mut img = RgbImage::new(self.size[0],self.size[1]/4);
        for x in 0..self.size[0] {
            for y in 0..self.size[1]/4{
                img.put_pixel(x, y,self.pos_to_rgb_bar(x as f32));
            }
        }
        let _ = img.save("./created_images/HSL_hue_rect.png");
    }
    pub fn pos_to_rgb_bar(&self,x:f32) -> Rgb<u8> {
        let h = (360.0/self.size[0] as f32) * x;
        let hsl = iris_color::HSL::new(h, 1.0,0.5);
        hsl.to_rgb()
    }
    pub fn rgb_color_to_position_bar(&self,rgb:&Rgb<u8>) -> u32{
        let hsl = iris_color::HSL::from_rgb(rgb); 
        let x = hsl.h / (360.0/self.size[0] as f32);
        x as u32
    }
}
#[derive(Clone)]
pub struct RGBMarker {
    rgb:Rgb<u8>,
    size:u32,
    border_size:u32,
}

impl RGBMarker {
    pub fn new(rgb:Rgb<u8>,size:u32,border_size:u32) -> Self{
        Self{rgb,size,border_size} 
    } 
}

impl Draw for RGBMarker{
    fn draw_rect(&mut self,parent: &mut HSLRect) {
        let rgb_pos = parent.rgb_color_to_position_rect(&self.rgb);
        let x_start = if let Some(val) = rgb_pos[0].checked_sub(self.size) {val} else {0};
        let y_start = if let Some(val) = rgb_pos[1].checked_sub(self.size) {val} else {0};
        let x_end = if let Some(val) = rgb_pos[0].checked_add(self.size) {val} else {u32::MAX};
        let y_end = if let Some(val) = rgb_pos[1].checked_add(self.size) {val} else {u32::MAX};
        for x in x_start..x_end{
            for y in y_start..y_end{
                let dist = ((x as f32 - rgb_pos[0]as f32).powf(2.0) + (y as f32 - rgb_pos[1] as f32).powf(2.0)).sqrt();
                if x < parent.size[0] && y < parent.size[1] {
                    if dist < (self.size/2) as f32 {
                        parent.img.put_pixel(x, y,self.rgb);
                    }
                    if dist >= (self.size/2) as f32 && dist <= (self.size/2) as f32 + (self.border_size as f32/2.0) {
                        parent.img.put_pixel(x, y,Rgb::from([255,255,255]));
                    }
                    if dist > (self.size/2) as f32 + (self.border_size as f32/2.0) && dist <= (self.size/2) as f32 + self.border_size as f32 {
                        parent.img.put_pixel(x, y,Rgb::from([0,0,0]));
                    }
                }
            }
        }
    }    
    fn draw_bar(&mut self, _parent: &mut HSLRect) {
        
    }
}
