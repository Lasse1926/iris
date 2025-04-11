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
                    self.iris_gen();
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
}

