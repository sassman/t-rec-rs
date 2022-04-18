use winapi::shared::windef::{HMONITOR, HWND};
use winapi::Interface;

use windows::core::{RawPtr, GUID, HRESULT};
use windows::Graphics::Capture::GraphicsCaptureItem;

#[repr(C)]
pub struct abi_IGraphicsCaptureItemInterop {
    __base: [usize; 3],
    create_for_window: extern "system" fn(
        *const *const abi_IGraphicsCaptureItemInterop,
        HWND,
        &GUID,
        *mut RawPtr,
    ) -> HRESULT,
    create_for_monitor: extern "system" fn(
        *const *const abi_IGraphicsCaptureItemInterop,
        HMONITOR,
        &GUID,
        *mut RawPtr,
    ) -> HRESULT,
}

unsafe impl Interface for GraphicsCaptureItemInterop {
    type VTable = abi_IGraphicsCaptureItemInterop;
    const IID: GUID =
        GUID::from_values(908650523, 15532, 19552, [183, 244, 35, 206, 14, 12, 51, 86]);
}

#[repr(transparent)]
#[derive(Default, Clone)]
pub struct GraphicsCaptureItemInterop {
    ptr: windows::core::ComPtr<GraphicsCaptureItemInterop>,
}

impl GraphicsCaptureItemInterop {
    // pub fn create_for_monitor(&self, monitor: HMONITOR) -> windows::core::Result<GraphicsCaptureItem> {
    //     let this = self.ptr.abi();
    //     if this.is_null() {
    //         panic!("`this` was null");
    //     }
    //     unsafe {
    //         let mut result: GraphicsCaptureItem = std::mem::zeroed();

    //         ((*(*(this))).create_for_monitor)(
    //             this,
    //             monitor,
    //             &GraphicsCaptureItem::IID,
    //             &mut result as *mut _ as _,
    //         )
    //         .ok()?;
    //         Ok(result)
    //     }
    // }

    pub fn create_for_window(&self, window: HWND) -> windows::core::Result<GraphicsCaptureItem> {
        let this = self.ptr.abi();
        if this.is_null() {
            panic!("`this` was null");
        }
        unsafe {
            let mut result: GraphicsCaptureItem = std::mem::zeroed();

            ((*(*(this))).create_for_window)(
                this,
                window,
                &GraphicsCaptureItem::IID,
                &mut result as *mut _ as _,
            )
            .ok()?;
            Ok(result)
        }
    }
}

// pub fn create_capture_item_for_monitor(monitor: HMONITOR) -> windows::core::Result<GraphicsCaptureItem> {
//     let factory = windows::core::activation::factory::<GraphicsCaptureItem, windows::core::IActivationFactory>()?;
//     let interop: GraphicsCaptureItemInterop = factory.try_into()?;
//     let item = interop.create_for_monitor(monitor)?;
//     Ok(item)
// }

pub fn create_capture_item_for_window(window: HWND) -> windows::core::Result<GraphicsCaptureItem> {
    let factory = windows::core::activation::factory::<
        GraphicsCaptureItem,
        windows::core::IActivationFactory,
    >()?;
    let interop: GraphicsCaptureItemInterop = factory.try_into()?;
    let item = interop.create_for_window(window)?;
    Ok(item)
}
