use lazy_static::lazy_static;
use limine::{Framebuffer, NonNullPtr};

static FRAMEBUFFER_REQUEST: limine::FramebufferRequest = limine::FramebufferRequest::new(0);

pub struct FrameBuffer {
    framebuffer: &'static NonNullPtr<Framebuffer>,
}

lazy_static! {
    pub static ref FRAME_BUFFER: FrameBuffer = FrameBuffer::new();
}

impl FrameBuffer {
    pub fn new() -> FrameBuffer {
        let framebuffer_response = FRAMEBUFFER_REQUEST.get_response().get().unwrap();
        let framebuffer = &framebuffer_response.framebuffers()[0];

        FrameBuffer { framebuffer }
    }

    // TODO: Realizing at this point that these put/get functions only work with the 32-bit bpp
    // But haven't looked into framebuffer stuff enough so I guess sure find whatever
    pub fn put_pixel(&self, pixel_offset: usize, value: u32) {
        unsafe {
            *(self
                .framebuffer
                .address
                .as_ptr()
                .unwrap()
                .offset(pixel_offset as isize) as *mut u32) = value;
        }
    }

    pub fn get_pixel(&self, pixel_offset: usize) -> u32 {
        unsafe {
            return *((self
                .framebuffer
                .address
                .as_ptr()
                .unwrap()
                .offset(pixel_offset as isize) as *mut u32)
                .as_ref()
                .unwrap());
        }
    }

    pub fn clear_pixel(&self, pixel_offset: usize) {
        self.put_pixel(pixel_offset, 0x00000000)
    }

    pub fn clear_range(&self, pixel_start: usize, range: usize) {
        for index in 0..range {
            self.clear_pixel(pixel_start + index * self.bytes_per_pixel())
        }
    }

    pub fn move_range(&self, pixel_start: usize, range: usize, offset: usize) {
        for index in 0..range {
            let pixel_value =
                self.get_pixel(pixel_start + ((index + offset) * self.bytes_per_pixel()));

            self.put_pixel(pixel_start + (index * self.bytes_per_pixel()), pixel_value);
        }

        self.clear_range(pixel_start + (range * self.bytes_per_pixel()), offset)
    }

    pub fn clear_screen(&self) {
        for i in 0..(self.framebuffer.height * self.framebuffer.pitch) {
            self.put_pixel((i as usize * self.bytes_per_pixel()), 0x00000000)
        }
    }

    pub fn pitch(&self) -> u64 {
        self.framebuffer.pitch
    }

    pub fn width(&self) -> u64 {
        self.framebuffer.width
    }

    pub fn height(&self) -> u64 {
        self.framebuffer.height
    }

    pub fn bytes_per_pixel(&self) -> usize {
        self.framebuffer.bpp as usize / 8
    }
}
