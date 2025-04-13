use image::{RgbImage,Rgb};
use super::WINDOW_ID;

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
                    self.generate_rgb_image_rect();
                }
            });
            self.open = window_open;
        }
    }
    pub fn iris_gen(&self){
        let mut img = RgbImage::new(64,64);
        for x in 0..64 {
            for y in 0..64 {
                img.put_pixel(x, y, Rgb([255, 0, 0]));
            }
        }
        let _ = img.save("./created_images/color_rect.png");
    }
    pub fn generate_rgb_image_rect(&self) {
        let mut img = RgbImage::new(64,64);
        for x in 0..64 {
            let rgb = self.rgb_rect_y(x as f32 / 64.0);  

            let mut rgb_u8:[u8;3]=[0,0,0];

            rgb_u8[0] = (rgb[0] * 255.0) as u8;
            rgb_u8[1] = (rgb[2] * 255.0) as u8;
            rgb_u8[2] = (rgb[1] * 255.0) as u8;



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
                    println!("y : {} ----------Step {}-------",y,step);
                    println!("r: {}",rgb_u8[0]);
                    println!("g: {}",rgb_u8[1]);
                    println!("b: {}",rgb_u8[2]);
                }
                img.put_pixel(x, y, Rgb(rgb_u8));
            }
        }
        let _ = img.save("./created_images/rgb_rect.png");
    }
    fn rgb_rect_y(&self,x:f32) -> [f32;3] {
        let r = (-9.0*x.powf(2.0) + 3.0*x + 0.75).max(0.0);
        let g = (-9.0*x.powf(2.0) + 9.0*x - 1.25).max(0.0);
        let b = (-9.0*x.powf(2.0) + 15.0*x - 5.25).max(0.0);

        let b_2 = (-9.0*x.powf(2.0) -3.0*x + 0.75).max(0.0); 
        let r_2 = (-9.0*x.powf(2.0) + 21.0*x - 11.25).max(0.0);

        [r + r_2,g,b + b_2]
    }
}


