use std::mem;
use std::path::Path;

use windows::Win32::Foundation::SIZE;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC,
    DeleteObject, GetDIBits, HBITMAP, HDC,
};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::UI::Shell::{
    IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_ICONONLY,
};
use windows::core::HSTRING;

const BYTES_PER_PIXEL: usize = 4;
const BITS_PER_PIXEL: u16 = 32;
const COLOUR_PLANE_COUNT: u16 = 1;
const FULLY_OPAQUE: u32 = 255;

#[derive(Debug, thiserror::Error)]
pub enum ShellIconError {
    #[error("Windows has no icon for it: {0}")]
    Unavailable(windows::core::Error),
    #[error("the pixels of the icon could not be read")]
    UnreadablePixels,
    #[error("the icon could not be turned into an image: {0}")]
    Encoding(#[from] png::EncodingError),
}

struct ComApartment {
    was_entered: bool,
}

impl ComApartment {
    #[allow(unsafe_code)]
    fn enter() -> Self {
        let outcome = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        Self {
            was_entered: outcome.is_ok(),
        }
    }
}

impl Drop for ComApartment {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        if self.was_entered {
            unsafe { CoUninitialize() };
        }
    }
}

pub fn read_icon_as_png(path: &Path, side_length: u16) -> Result<Vec<u8>, ShellIconError> {
    let _apartment = ComApartment::enter();
    let bitmap = read_icon_bitmap(path, side_length)?;
    let mut pixels = read_pixels(bitmap, side_length)?;
    make_straight_rgba(&mut pixels);
    encode_png(&pixels, side_length)
}

#[allow(unsafe_code)]
fn read_icon_bitmap(path: &Path, side_length: u16) -> Result<HBITMAP, ShellIconError> {
    let wanted_size = SIZE {
        cx: i32::from(side_length),
        cy: i32::from(side_length),
    };
    let shell_path = HSTRING::from(path.as_os_str());
    let image_factory: IShellItemImageFactory =
        unsafe { SHCreateItemFromParsingName(&shell_path, None) }
            .map_err(ShellIconError::Unavailable)?;
    unsafe { image_factory.GetImage(wanted_size, SIIGBF_ICONONLY) }
        .map_err(ShellIconError::Unavailable)
}

#[allow(unsafe_code)]
fn read_pixels(bitmap: HBITMAP, side_length: u16) -> Result<Vec<u8>, ShellIconError> {
    let side = usize::from(side_length);
    let mut pixels = vec![0_u8; side * side * BYTES_PER_PIXEL];
    let mut layout = top_down_layout(side_length);
    let device_context = unsafe { CreateCompatibleDC(HDC::default()) };
    let copied_row_count = unsafe {
        GetDIBits(
            device_context,
            bitmap,
            0,
            u32::from(side_length),
            Some(pixels.as_mut_ptr().cast()),
            &raw mut layout,
            DIB_RGB_COLORS,
        )
    };
    let was_released =
        unsafe { DeleteDC(device_context).as_bool() && DeleteObject(bitmap).as_bool() };
    if copied_row_count == i32::from(side_length) && was_released {
        Ok(pixels)
    } else {
        Err(ShellIconError::UnreadablePixels)
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "the bitmap header is forty bytes"
)]
fn top_down_layout(side_length: u16) -> BITMAPINFO {
    BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: i32::from(side_length),
            biHeight: -i32::from(side_length),
            biPlanes: COLOUR_PLANE_COUNT,
            biBitCount: BITS_PER_PIXEL,
            biCompression: BI_RGB.0,
            ..BITMAPINFOHEADER::default()
        },
        ..BITMAPINFO::default()
    }
}

fn make_straight_rgba(premultiplied_bgra: &mut [u8]) {
    for pixel in premultiplied_bgra.chunks_exact_mut(BYTES_PER_PIXEL) {
        let alpha = u32::from(pixel[3]);
        let [blue, green, red] = [pixel[0], pixel[1], pixel[2]].map(|channel| {
            let straight = u32::from(channel) * FULLY_OPAQUE / alpha.max(1);
            u8::try_from(straight.min(FULLY_OPAQUE)).unwrap_or(u8::MAX)
        });
        pixel[0] = red;
        pixel[1] = green;
        pixel[2] = blue;
    }
}

fn encode_png(rgba_pixels: &[u8], side_length: u16) -> Result<Vec<u8>, ShellIconError> {
    let mut png_bytes = Vec::new();
    {
        let side = u32::from(side_length);
        let mut encoder = png::Encoder::new(&mut png_bytes, side, side);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(rgba_pixels)?;
        writer.finish()?;
    }
    Ok(png_bytes)
}
