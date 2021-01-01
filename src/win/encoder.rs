use super::d3d::{D3D11Device, D3D11Texture2D};
use crate::windows::graphics::directx::direct3d11::{IDirect3DDevice, IDirect3DSurface};
// use std::path::Path;

use winapi::{
    shared::dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM,
    um::d3d11::{
        D3D11_CPU_ACCESS_READ, D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ, D3D11_USAGE_STAGING,
    },
};


// pub fn save_d3d_surface<P>(
//     device: &IDirect3DDevice,
//     surface: &IDirect3DSurface,
//     path: P,
// ) -> winrt::Result<()>
// where
//     P: AsRef<Path>,
// {
//     let d3d_device = D3D11Device::from_direct3d_device(device)?;
//     let d3d_context = d3d_device.get_immediate_context();
//     let d3d_texture = D3D11Texture2D::from_direct3d_surface(surface)?;

//     // Make sure the surface is a pixel format we support
//     let desc = d3d_texture.get_desc();
//     let width = desc.Width as u32;
//     let height = desc.Height as u32;
//     let bytes_per_pixel = match desc.Format {
//         DXGI_FORMAT_B8G8R8A8_UNORM => 4,
//         _ => panic!("Unsupported format! {:?}", desc.Format),
//     };

//     // TODO: If the texture isn't marked for staging, make a copy
//     let d3d_texture = if desc.Usage == D3D11_USAGE_STAGING {
//         if (desc.CPUAccessFlags & D3D11_CPU_ACCESS_READ) == D3D11_CPU_ACCESS_READ {
//             d3d_texture
//         } else {
//             panic!("CPU read access required!");
//         }
//     } else {
//         panic!("Unsupported buffer type! Must be a staging buffer!");
//     };

//     // Map the texture
//     let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
//     d3d_context.map(&d3d_texture, 0, D3D11_MAP_READ, 0, &mut mapped)?;

//     // Get a slice of bytes
//     let slice: &[u8] = unsafe {
//         std::slice::from_raw_parts(
//             mapped.pData as *const _,
//             (height * mapped.RowPitch) as usize,
//         )
//     };

//     // Make a copy of the data
//     let mut data = vec![0u8; ((width * height) * bytes_per_pixel) as usize];
//     for row in 0..height {
//         let data_begin = (row * (width * bytes_per_pixel)) as usize;
//         let data_end = ((row + 1) * (width * bytes_per_pixel)) as usize;
//         let slice_begin = (row * mapped.RowPitch) as usize;
//         let slice_end = slice_begin + (width * bytes_per_pixel) as usize;
//         data[data_begin..data_end].copy_from_slice(&slice[slice_begin..slice_end]);
//     }

//     // Unmap the texture
//     d3d_context.unmap(&d3d_texture, 0);

//     // Save the bits
//     let image: image::ImageBuffer<image::Bgra<u8>, _> =
//         image::ImageBuffer::from_raw(width, height, data).unwrap();
//     // The image crate doesn't seem to support saving bgra8 :(
//     let dynamic_image = image::DynamicImage::ImageBgra8(image);
//     let dynamic_image = dynamic_image.to_rgba();
//     dynamic_image.save(path).unwrap();

//     Ok(())
// }

pub fn encode_d3d_surface(
    device: &IDirect3DDevice,
    surface: &IDirect3DSurface,
) -> winrt::Result<image::FlatSamples<Vec<u8>>>
{
    let d3d_device = D3D11Device::from_direct3d_device(device)?;
    let d3d_context = d3d_device.get_immediate_context();
    let d3d_texture = D3D11Texture2D::from_direct3d_surface(surface)?;

    // Make sure the surface is a pixel format we support
    let desc = d3d_texture.get_desc();
    let width = desc.Width as u32;
    let height = desc.Height as u32;
    let bytes_per_pixel = match desc.Format {
        DXGI_FORMAT_B8G8R8A8_UNORM => 4,
        _ => panic!("Unsupported format! {:?}", desc.Format),
    };

    // TODO: If the texture isn't marked for staging, make a copy
    let d3d_texture = if desc.Usage == D3D11_USAGE_STAGING {
        if (desc.CPUAccessFlags & D3D11_CPU_ACCESS_READ) == D3D11_CPU_ACCESS_READ {
            d3d_texture
        } else {
            panic!("CPU read access required!");
        }
    } else {
        panic!("Unsupported buffer type! Must be a staging buffer!");
    };

    // Map the texture
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    d3d_context.map(&d3d_texture, 0, D3D11_MAP_READ, 0, &mut mapped)?;

    // Get a slice of bytes
    let slice: &[u8] = unsafe {
        std::slice::from_raw_parts(
            mapped.pData as *const _,
            (height * mapped.RowPitch) as usize,
        )
    };

    // Make a copy of the data
    let mut data = vec![0u8; ((width * height) * bytes_per_pixel) as usize];
    for row in 0..height {
        let data_begin = (row * (width * bytes_per_pixel)) as usize;
        let data_end = ((row + 1) * (width * bytes_per_pixel)) as usize;
        let slice_begin = (row * mapped.RowPitch) as usize;
        let slice_end = slice_begin + (width * bytes_per_pixel) as usize;
        data[data_begin..data_end].copy_from_slice(&slice[slice_begin..slice_end]);
    }

    // Unmap the texture
    d3d_context.unmap(&d3d_texture, 0);

    // Save the bits
    let image: image::ImageBuffer<image::Bgra<u8>, _> =
        image::ImageBuffer::from_raw(width, height, data).unwrap();
    // // The image crate doesn't seem to support saving bgra8 :(
    // let dynamic_image = image::DynamicImage::ImageBgra8(image);
    // let dynamic_image = dynamic_image.to_rgba();
    // dynamic_image.save(path).unwrap();

    Ok(image.into_flat_samples())
}
