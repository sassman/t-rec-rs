use crate::windows::graphics::capture::GraphicsCaptureItem;
use winapi::shared::windef::{HMONITOR, HWND};
use winrt::{ComInterface, Guid, TryInto};

#[repr(C)]
pub struct abi_IGraphicsCaptureItemInterop {
    __base: [usize; 3],
    create_for_window: extern "system" fn(
        *const *const abi_IGraphicsCaptureItemInterop,
        HWND,
        &Guid,
        *mut winrt::RawPtr,
    ) -> winrt::ErrorCode,
    create_for_monitor: extern "system" fn(
        *const *const abi_IGraphicsCaptureItemInterop,
        HMONITOR,
        &Guid,
        *mut winrt::RawPtr,
    ) -> winrt::ErrorCode,
}

unsafe impl winrt::ComInterface for GraphicsCaptureItemInterop {
    type VTable = abi_IGraphicsCaptureItemInterop;
    const IID: winrt::Guid =
        winrt::Guid::from_values(908650523, 15532, 19552, [183, 244, 35, 206, 14, 12, 51, 86]);
}

#[repr(transparent)]
#[derive(Default, Clone)]
pub struct GraphicsCaptureItemInterop {
    ptr: winrt::ComPtr<GraphicsCaptureItemInterop>,
}

impl GraphicsCaptureItemInterop {
    // pub fn create_for_monitor(&self, monitor: HMONITOR) -> winrt::Result<GraphicsCaptureItem> {
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

    pub fn create_for_window(&self, window: HWND) -> winrt::Result<GraphicsCaptureItem> {
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

// pub fn create_capture_item_for_monitor(monitor: HMONITOR) -> winrt::Result<GraphicsCaptureItem> {
//     let factory = winrt::activation::factory::<GraphicsCaptureItem, winrt::IActivationFactory>()?;
//     let interop: GraphicsCaptureItemInterop = factory.try_into()?;
//     let item = interop.create_for_monitor(monitor)?;
//     Ok(item)
// }

pub fn create_capture_item_for_window(window: HWND) -> winrt::Result<GraphicsCaptureItem> {
    let factory = winrt::activation::factory::<GraphicsCaptureItem, winrt::IActivationFactory>()?;
    let interop: GraphicsCaptureItemInterop = factory.try_into()?;
    let item = interop.create_for_window(window)?;
    Ok(item)
}
