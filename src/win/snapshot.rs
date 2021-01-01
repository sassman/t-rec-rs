use super::d3d::{D3D11Device, D3D11Texture2D};
use crate::windows::graphics::capture::{
    Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession,
};
use crate::windows::graphics::directx::direct3d11::{IDirect3DDevice, IDirect3DSurface};
use crate::windows::graphics::directx::DirectXPixelFormat;
use std::sync::mpsc::{channel, Receiver, Sender};
use winapi::um::d3d11::{D3D11_CPU_ACCESS_READ, D3D11_USAGE_STAGING};

type FrameArrivedHandler =
    crate::windows::foundation::TypedEventHandler<Direct3D11CaptureFramePool, winrt::Object>;

pub struct CaptureSnapshot {
    _session: GraphicsCaptureSession,
    trigger_capture: Sender<()>,
    image_data: Receiver<D3D11Texture2D>,
}

impl CaptureSnapshot {
    pub fn create_session(
        device: &IDirect3DDevice,
        item: &GraphicsCaptureItem,
    ) -> winrt::Result<CaptureSnapshot> {
        let d3d_device = D3D11Device::from_direct3d_device(device)?;
        let d3d_context = d3d_device.get_immediate_context();
        let item_size = item.size()?;

        // Initialize the capture
        let frame_pool = Direct3D11CaptureFramePool::create_free_threaded(
            device,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            1,
            &item_size,
        )?;
        let session = frame_pool.create_capture_session(item)?;

        let (trigger_capture_sender, trigger_capture_receiver) = channel();
        let (image_data_sender, image_data_receiver) = channel();
        let frame_arrived = FrameArrivedHandler::new({
            let d3d_device = d3d_device.clone();
            let d3d_context = d3d_context.clone();
            let session = session.clone();
            move |frame_pool, _| {
                if let Ok(()) = trigger_capture_receiver.recv() {
                    let frame = frame_pool.try_get_next_frame()?;
                    let surface = frame.surface()?;

                    let frame_texture = D3D11Texture2D::from_direct3d_surface(&surface)?;

                    // Make a copy of the texture
                    let mut desc = frame_texture.get_desc();
                    // Make this a staging texture
                    desc.Usage = D3D11_USAGE_STAGING;
                    desc.BindFlags = 0;
                    desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ;
                    desc.MiscFlags = 0;
                    let copy_texture = d3d_device.create_texture_2d(&desc, None)?;
                    d3d_context.copy_resource(&copy_texture, &frame_texture);

                    // // End the capture
                    // session.close()?;
                    // frame_pool.close()?;

                    image_data_sender.send(copy_texture).unwrap();
                }
                Ok(())
            }
        });

        // Start the capture
        frame_pool.frame_arrived(frame_arrived)?;
        session.start_capture()?;
        Ok(CaptureSnapshot {
            _session: session,
            image_data: image_data_receiver,
            trigger_capture: trigger_capture_sender,
        })
    }

    pub fn take_session(&self) -> winrt::Result<IDirect3DSurface> {
        self.trigger_capture.send(()).unwrap();
        // Wait for our texture to come
        let texture = self.image_data.recv().unwrap();
        let surface = texture.to_direct3d_surface()?;

        Ok(surface)
    }

    // TODO: Allow to create non-staging textures
    // TODO: Allow specifying pixel format
    // TODO: Async?
    pub fn take(
        device: &IDirect3DDevice,
        item: &GraphicsCaptureItem,
    ) -> winrt::Result<IDirect3DSurface> {
        let d3d_device = D3D11Device::from_direct3d_device(device)?;
        let d3d_context = d3d_device.get_immediate_context();
        let item_size = item.size()?;

        // Initialize the capture
        let frame_pool = Direct3D11CaptureFramePool::create_free_threaded(
            device,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            1,
            &item_size,
        )?;
        let session = frame_pool.create_capture_session(item)?;

        // Setup the frame arrived handler
        let (sender, receiver) = channel();
        let frame_arrived = FrameArrivedHandler::new({
            let d3d_device = d3d_device.clone();
            let d3d_context = d3d_context.clone();
            let session = session.clone();
            move |frame_pool, _| {
                let frame = frame_pool.try_get_next_frame()?;
                let surface = frame.surface()?;

                let frame_texture = D3D11Texture2D::from_direct3d_surface(&surface)?;

                // Make a copy of the texture
                let mut desc = frame_texture.get_desc();
                // Make this a staging texture
                desc.Usage = D3D11_USAGE_STAGING;
                desc.BindFlags = 0;
                desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ;
                desc.MiscFlags = 0;
                let copy_texture = d3d_device.create_texture_2d(&desc, None)?;
                d3d_context.copy_resource(&copy_texture, &frame_texture);

                // End the capture
                session.close()?;
                frame_pool.close()?;

                sender.send(copy_texture).unwrap();

                Ok(())
            }
        });

        // Start the capture
        frame_pool.frame_arrived(frame_arrived)?;
        session.start_capture()?;

        // Wait for our texture to come
        let texture = receiver.recv().unwrap();
        let surface = texture.to_direct3d_surface()?;

        Ok(surface)
    }
}
