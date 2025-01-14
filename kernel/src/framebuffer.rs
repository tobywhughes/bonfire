use core::ptr;
use limine::framebuffer::{self, Framebuffer};
use limine::request::FramebufferRequest;

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

pub struct FrameBuffer<'a> {
    _framebuffer: Option<Framebuffer<'a>>,
}

impl FrameBuffer<'_> {
    pub fn new<'a>() -> FrameBuffer<'a> {
        let framebuffer_response = FRAMEBUFFER_REQUEST.get_response();

        match framebuffer_response {
            Some(framebuffer_response) => {
                let framebuffer = framebuffer_response.framebuffers().next();

                FrameBuffer {
                    _framebuffer: framebuffer,
                }
            }
            None => FrameBuffer { _framebuffer: None },
        }
    }

    fn with_famebuffer_ref<F, T>(&self, callback: F, default: T) -> T
    where
        F: FnOnce(&Framebuffer) -> T,
    {
        if let Some(framebuffer_ref) = self._framebuffer.as_ref() {
            return callback(framebuffer_ref);
        }

        default
    }

    pub fn put_pixel(&self, pixel_offset: usize, value: u32) {
        self.with_famebuffer_ref(
            |framebuffer_ref| {
                // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
                unsafe { *(framebuffer_ref.addr().add(pixel_offset as usize) as *mut u32) = value }
            },
            (),
        );
    }

    pub fn pitch(&self) -> Option<u64> {
        self.with_famebuffer_ref(
            |framebuffer_ref| {
                // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
                Some(framebuffer_ref.pitch())
            },
            None,
        )
    }

    pub fn height(&self) -> Option<u64> {
        self.with_famebuffer_ref(
            |framebuffer_ref| {
                // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
                Some(framebuffer_ref.height())
            },
            None,
        )
    }

    pub fn bpp(&self) -> Option<u16> {
        self.with_famebuffer_ref(
            |framebuffer_ref| {
                // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
                Some(framebuffer_ref.bpp())
            },
            None,
        )
    }

    fn framebuffer_size(&self) -> Option<u64> {
        self.with_famebuffer_ref(
            |framebuffer_ref| Some(framebuffer_ref.pitch() * (framebuffer_ref.height())),
            None,
        )
    }

    pub fn scroll_framebuffer(&self, scroll_height: usize) {
        self.with_famebuffer_ref(
            |framebuffer_ref| unsafe {
                let scroll_size = framebuffer_ref.pitch() as usize * scroll_height;
                let destination_addr = framebuffer_ref.addr();
                let src_addr = framebuffer_ref.addr().add(scroll_size);

                let framebuffer_size = self.framebuffer_size().unwrap_or(0);

                let buffer_size = framebuffer_size.saturating_sub(scroll_size as u64) as usize;

                ptr::copy(src_addr, destination_addr, buffer_size);

                let clear_addr = framebuffer_ref.addr().add(buffer_size);

                ptr::write_bytes(clear_addr, 0x00, scroll_size);
            },
            (),
        )
    }
}
