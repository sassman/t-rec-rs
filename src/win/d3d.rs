use crate::windows::graphics::directx::direct3d11::{IDirect3DDevice, IDirect3DSurface};
use winapi::{
    shared::{
        dxgi::{IDXGIDeviceVtbl, IDXGISurfaceVtbl},
        winerror::DXGI_ERROR_UNSUPPORTED,
    },
    um::d3d11::{
        D3D11CreateDevice, ID3D11DeviceContextVtbl, ID3D11DeviceVtbl, ID3D11Resource,
        ID3D11Texture2DVtbl, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAP, D3D11_MAPPED_SUBRESOURCE,
        D3D11_SDK_VERSION, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC,
    },
};
use winrt::{Guid, RuntimeType, TryInto};

#[repr(C)]
pub struct abi_IDirect3DDxgiInterfaceAccess {
    __base: [usize; 3],
    get_interface: extern "system" fn(
        *const *const abi_IDirect3DDxgiInterfaceAccess,
        &Guid,
        *mut winrt::RawPtr,
    ) -> winrt::ErrorCode,
}

unsafe impl winrt::ComInterface for Direct3DDxgiInterfaceAccess {
    type VTable = abi_IDirect3DDxgiInterfaceAccess;
    const IID: winrt::Guid = winrt::Guid::from_values(
        2847133714,
        15858,
        20195,
        [184, 209, 134, 149, 244, 87, 211, 193],
    );
}

#[repr(transparent)]
#[derive(Default, Clone)]
pub struct Direct3DDxgiInterfaceAccess {
    ptr: winrt::ComPtr<Direct3DDxgiInterfaceAccess>,
}

impl Direct3DDxgiInterfaceAccess {
    pub fn get_interface<Into: winrt::ComInterface>(&self) -> winrt::Result<Into> {
        let this = self.ptr.abi();
        if this.is_null() {
            panic!("`this` was null");
        }
        unsafe {
            let mut result: Into = std::mem::zeroed();

            ((*(*(this))).get_interface)(this, &Into::IID, &mut result as *mut _ as _).ok()?;
            Ok(result)
        }
    }
}

#[repr(transparent)]
#[derive(Default, Clone)]
pub struct D3D11Texture2D {
    ptr: winrt::ComPtr<D3D11Texture2D>,
}

unsafe impl winrt::ComInterface for D3D11Texture2D {
    type VTable = ID3D11Texture2DVtbl;
    const IID: winrt::Guid = winrt::Guid::from_values(
        1863690994,
        53768,
        20105,
        [154, 180, 72, 149, 53, 211, 79, 156],
    );
}

#[repr(transparent)]
#[derive(Default, Clone)]
pub struct D3D11Device {
    ptr: winrt::ComPtr<D3D11Device>,
}

unsafe impl winrt::ComInterface for D3D11Device {
    type VTable = ID3D11DeviceVtbl;
    const IID: winrt::Guid = winrt::Guid::from_values(
        3681512923,
        44151,
        20104,
        [130, 83, 129, 157, 249, 187, 241, 64],
    );
}

#[repr(transparent)]
#[derive(Default, Clone)]
pub struct D3D11DeviceContext {
    ptr: winrt::ComPtr<D3D11DeviceContext>,
}

unsafe impl winrt::ComInterface for D3D11DeviceContext {
    type VTable = ID3D11DeviceContextVtbl;
    const IID: winrt::Guid = winrt::Guid::from_values(
        3233786220,
        57481,
        17659,
        [142, 175, 38, 248, 121, 97, 144, 218],
    );
}

#[repr(transparent)]
#[derive(Default, Clone)]
pub struct DXGISurface {
    ptr: winrt::ComPtr<DXGISurface>,
}

unsafe impl winrt::ComInterface for DXGISurface {
    type VTable = IDXGISurfaceVtbl;
    const IID: winrt::Guid = winrt::Guid::from_values(
        3405559148,
        27331,
        18569,
        [191, 71, 158, 35, 187, 210, 96, 236],
    );
}

#[repr(transparent)]
#[derive(Default, Clone)]
pub struct DXGIDevice {
    ptr: winrt::ComPtr<DXGIDevice>,
}

unsafe impl winrt::ComInterface for DXGIDevice {
    type VTable = IDXGIDeviceVtbl;
    const IID: winrt::Guid = winrt::Guid::from_values(
        1424783354,
        4983,
        17638,
        [140, 50, 136, 253, 95, 68, 200, 76],
    );
}

pub trait D3D11Resource {
    fn get_d3d_resource(&self) -> *mut ID3D11Resource;
}

impl D3D11Resource for D3D11Texture2D {
    fn get_d3d_resource(&self) -> *mut ID3D11Resource {
        let this = self.ptr.abi();
        this as *mut _
    }
}

#[link(name = "d3d11")]
extern "stdcall" {
    fn CreateDirect3D11DeviceFromDXGIDevice(
        device: winrt::RawComPtr<DXGIDevice>,
        graphics_device: *mut <IDirect3DDevice as RuntimeType>::Abi,
    ) -> winrt::ErrorCode;
}

#[link(name = "d3d11")]
extern "stdcall" {
    fn CreateDirect3D11SurfaceFromDXGISurface(
        surface: winrt::RawComPtr<DXGISurface>,
        graphics_surface: *mut <IDirect3DSurface as RuntimeType>::Abi,
    ) -> winrt::ErrorCode;
}

#[allow(dead_code)]
#[repr(u32)]
pub enum D3DDriverType {
    Unknown = 0,
    Hardware = 1,
    Reference = 2,
    Null = 3,
    Software = 4,
    Warp = 5,
}

impl D3D11Device {
    pub fn from_direct3d_device(device: &IDirect3DDevice) -> winrt::Result<Self> {
        let access: Direct3DDxgiInterfaceAccess = device.try_into()?;
        access.get_interface()
    }

    pub fn new_of_type(driver_type: D3DDriverType) -> winrt::Result<Self> {
        let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;
        let device = unsafe {
            let mut device = winrt::IUnknown::default();
            winrt::ErrorCode(D3D11CreateDevice(
                std::ptr::null_mut(),
                driver_type as u32,
                std::ptr::null_mut(),
                flags,
                std::ptr::null(),
                0,
                D3D11_SDK_VERSION,
                device.set_abi() as *mut *mut _,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ))
            .ok()?;
            std::mem::transmute(device)
        };
        Ok(Self { ptr: device })
    }

    pub fn new() -> winrt::Result<Self> {
        let result = Self::new_of_type(D3DDriverType::Hardware);
        match result {
            Ok(device) => Ok(device),
            Err(error) => {
                if error.code().0 == DXGI_ERROR_UNSUPPORTED {
                    Self::new_of_type(D3DDriverType::Warp)
                } else {
                    Err(error)
                }
            }
        }
    }

    pub fn to_direct3d_device(&self) -> winrt::Result<IDirect3DDevice> {
        let this = self.ptr.abi();
        if this.is_null() {
            panic!("`this` was null");
        }

        unsafe {
            let dxgi_device: DXGIDevice = self.try_into()?;
            let mut result: IDirect3DDevice = std::mem::zeroed();
            CreateDirect3D11DeviceFromDXGIDevice(dxgi_device.ptr.abi(), result.set_abi()).ok()?;
            Ok(result)
        }
    }

    pub fn create_texture_2d(
        &self,
        desc: &D3D11_TEXTURE2D_DESC,
        initial_data: Option<&D3D11_SUBRESOURCE_DATA>,
    ) -> winrt::Result<D3D11Texture2D> {
        let this = self.ptr.abi();
        if this.is_null() {
            panic!("`this` was null");
        }

        unsafe {
            let mut result: D3D11Texture2D = std::mem::zeroed();

            let initial_data = match initial_data {
                Some(data) => data,
                None => std::ptr::null(),
            };

            winrt::ErrorCode(((*(*(this))).CreateTexture2D)(
                this as *mut _,
                desc,
                initial_data,
                &mut result as *mut _ as _,
            ))
            .ok()?;
            Ok(result)
        }
    }

    pub fn get_immediate_context(&self) -> D3D11DeviceContext {
        let this = self.ptr.abi();
        if this.is_null() {
            panic!("`this` was null");
        }

        unsafe {
            let mut result: D3D11DeviceContext = std::mem::zeroed();

            ((*(*(this))).GetImmediateContext)(this as *mut _, &mut result as *mut _ as _);
            result
        }
    }
}

impl D3D11DeviceContext {
    pub fn map(
        &self,
        resource: &dyn D3D11Resource,
        subresource: u32,
        map_type: D3D11_MAP,
        map_flags: u32,
        mapped_resource: &mut D3D11_MAPPED_SUBRESOURCE,
    ) -> winrt::Result<()> {
        let this = self.ptr.abi();
        if this.is_null() {
            panic!("`this` was null");
        }

        let resource = resource.get_d3d_resource();
        if resource.is_null() {
            panic!("'resource' was null");
        }

        unsafe {
            winrt::ErrorCode(((*(*(this))).Map)(
                this as *mut _,
                resource,
                subresource,
                map_type,
                map_flags,
                mapped_resource as *mut _,
            ))
            .ok()?;
        };

        Ok(())
    }

    pub fn unmap(&self, resource: &dyn D3D11Resource, subresource: u32) {
        let this = self.ptr.abi();
        if this.is_null() {
            panic!("`this` was null");
        }

        let resource = resource.get_d3d_resource();
        if resource.is_null() {
            panic!("'resource' was null");
        }

        unsafe {
            ((*(*(this))).Unmap)(this as *mut _, resource, subresource);
        };
    }

    pub fn copy_resource(&self, dest: &dyn D3D11Resource, src: &dyn D3D11Resource) {
        let this = self.ptr.abi();
        if this.is_null() {
            panic!("`this` was null");
        }

        let dest = dest.get_d3d_resource();
        if dest.is_null() {
            panic!("'dest' was null");
        }

        let src = src.get_d3d_resource();
        if src.is_null() {
            panic!("'src' was null");
        }

        unsafe {
            ((*(*(this))).CopyResource)(this as *mut _, dest, src);
        };
    }
}

impl D3D11Texture2D {
    pub fn from_direct3d_surface(surface: &IDirect3DSurface) -> winrt::Result<Self> {
        let access: Direct3DDxgiInterfaceAccess = surface.try_into()?;
        access.get_interface()
    }

    pub fn to_direct3d_surface(&self) -> winrt::Result<IDirect3DSurface> {
        let this = self.ptr.abi();
        if this.is_null() {
            panic!("`this` was null");
        }

        unsafe {
            let dxgi_surface: DXGISurface = self.try_into()?;
            let mut result: IDirect3DSurface = std::mem::zeroed();
            CreateDirect3D11SurfaceFromDXGISurface(dxgi_surface.ptr.abi(), result.set_abi())
                .ok()?;
            Ok(result)
        }
    }

    pub fn get_desc(&self) -> D3D11_TEXTURE2D_DESC {
        let this = self.ptr.abi();
        if this.is_null() {
            panic!("`this` was null");
        }

        unsafe {
            let mut result = D3D11_TEXTURE2D_DESC::default();
            ((*(*(this))).GetDesc)(this as *mut _, &mut result as *mut _);
            result
        }
    }
}

#[test]
fn test_d3d_device() -> winrt::Result<()> {
    use crate::windows::graphics::capture::Direct3D11CaptureFramePool;
    use crate::windows::graphics::directx::DirectXPixelFormat;
    use crate::windows::graphics::SizeInt32;

    let d3d_device = D3D11Device::new()?;
    let device = d3d_device.to_direct3d_device()?;

    // The frame pool will create d3d textures
    let _frame_pool = Direct3D11CaptureFramePool::create_free_threaded(
        &device,
        DirectXPixelFormat::B8G8R8A8UIntNormalized,
        1,
        SizeInt32 {
            width: 100,
            height: 100,
        },
    )?;

    Ok(())
}

#[test]
fn test_d3d_texture_2d() -> winrt::Result<()> {
    use winapi::{
        shared::{dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM, dxgitype::DXGI_SAMPLE_DESC},
        um::d3d11::{
            D3D11_CPU_ACCESS_READ, D3D11_MAP_READ, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC,
            D3D11_USAGE_STAGING,
        },
    };

    let d3d_device = D3D11Device::new()?;
    let d3d_context = d3d_device.get_immediate_context();

    // Create and fill our texture with red
    let width = 200u32;
    let height = 200u32;
    let desc = D3D11_TEXTURE2D_DESC {
        Width: width,
        Height: height,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_STAGING,
        BindFlags: 0,
        CPUAccessFlags: D3D11_CPU_ACCESS_READ,
        MiscFlags: 0,
    };
    let mut data = Vec::new();
    for _ in 0..width * height {
        data.push(0u8);
        data.push(0u8);
        data.push(255u8);
        data.push(255u8);
    }
    let data: &[u8] = &data;
    let subresource_data = D3D11_SUBRESOURCE_DATA {
        pSysMem: data.as_ptr() as *const _,
        SysMemPitch: width * 4,
        SysMemSlicePitch: 0,
    };
    let texture = d3d_device.create_texture_2d(&desc, Some(&subresource_data))?;

    // Lock and read the texture, verifying it has red pixels
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    d3d_context.map(&texture, 0, D3D11_MAP_READ, 0, &mut mapped)?;

    // Get a slice of bytes
    let slice: &[u8] = unsafe {
        std::slice::from_raw_parts(
            mapped.pData as *const _,
            (height * mapped.RowPitch) as usize,
        )
    };

    // Find the center of the texture
    let center_x = width / 2;
    let center_y = height / 2;
    let offset = ((center_y * mapped.RowPitch) + (center_x * 4)) as usize;

    // Test for red
    assert_eq!(slice[offset], 0);
    assert_eq!(slice[offset + 1], 0);
    assert_eq!(slice[offset + 2], 255);
    assert_eq!(slice[offset + 3], 255);

    d3d_context.unmap(&texture, 0);

    Ok(())
}
