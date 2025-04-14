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
                    HSLRect::generate_sl_rect(313.0);
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

pub struct HSLRect;

impl HSLRect {
    pub fn generate_h_bar(){
        let mut img = RgbImage::new(64,16);
        for x in 0..64 {
            for y in 0..16{
                let h = (360.0/64.0) * x as f32;
                let hsl = iris_color::HSL::new(h, 1.0,0.5);
                let hsl_rgb = hsl.to_rgb();
                img.put_pixel(x, y,hsl_rgb);
            }
        }
        let _ = img.save("./created_images/HSL_hue_rect.png");
    }
    pub fn generate_sl_rect(h:f32){
        let mut img = RgbImage::new(64,64);
        for x in 0..64 {
            for y in 0..64{
                let s = x as f32/64.0;
                let l = (64.0-y as f32)/(64.0+s*64.0);
                let hsl = iris_color::HSL::new(h,s,l);
                let hsl_rgb = hsl.to_rgb();
                img.put_pixel(x, y,hsl_rgb);
            }
        }
        let _ = img.save("./created_images/HSL_saturation_lightness_rect.png");
    }
}

