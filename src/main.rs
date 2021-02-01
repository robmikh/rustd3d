mod dispatcher_queue;
mod window;

use bindings::windows::{
    foundation::numerics::Vector2,
    ui::{composition::Compositor, Colors},
    win32::{
        direct3d11::{
            D3D11CreateDevice, ID3D11Device, ID3D11RenderTargetView, ID3D11Resource,
            ID3D11Texture2D, D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION, D3D_DRIVER_TYPE,
        },
        dxgi::{
            IDXGIAdapter, IDXGIDevice2, IDXGIFactory2, IDXGIObject, DXGI_ALPHA_MODE, DXGI_FORMAT,
            DXGI_PRESENT_PARAMETERS, DXGI_SAMPLE_DESC, DXGI_SCALING, DXGI_SWAP_CHAIN_DESC1,
            DXGI_SWAP_EFFECT, DXGI_USAGE_RENDER_TARGET_OUTPUT,
        },
        system_services::DXGI_ERROR_UNSUPPORTED,
        windows_and_messaging::{DispatchMessageA, GetMessageA, HWND, MSG},
        winrt::{ICompositorDesktopInterop, ICompositorInterop, RoInitialize, RO_INIT_TYPE},
    },
    ErrorCode, IUnknown,
};
use dispatcher_queue::create_dispatcher_queue_controller_for_current_thread;
use window::BasicWindow;
use windows::{Abi, Interface};

fn create_d3d_device_with_type(
    driver_type: D3D_DRIVER_TYPE,
    flags: u32,
    device: *mut Option<ID3D11Device>,
) -> ErrorCode {
    unsafe {
        D3D11CreateDevice(
            None,
            driver_type,
            0,
            flags,
            std::ptr::null(),
            0,
            D3D11_SDK_VERSION as u32,
            device,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    }
}

fn create_d3d_device() -> windows::Result<ID3D11Device> {
    let mut device = None;
    let mut hresult = create_d3d_device_with_type(
        D3D_DRIVER_TYPE::D3D_DRIVER_TYPE_HARDWARE,
        D3D11_CREATE_DEVICE_FLAG::D3D11_CREATE_DEVICE_BGRA_SUPPORT.0 as u32,
        &mut device,
    );
    if hresult.0 == DXGI_ERROR_UNSUPPORTED as u32 {
        hresult = create_d3d_device_with_type(
            D3D_DRIVER_TYPE::D3D_DRIVER_TYPE_WARP,
            D3D11_CREATE_DEVICE_FLAG::D3D11_CREATE_DEVICE_BGRA_SUPPORT.0 as u32,
            &mut device,
        );
    }
    hresult.ok()?;
    Ok(device.unwrap())
}

fn dxgi_object_get_parent<T: Interface + Abi>(object: &IDXGIObject) -> windows::Result<T> {
    let mut parent: Option<T> = None;
    object
        .GetParent(&T::IID, &mut parent as *mut _ as *mut _)
        .ok()?;
    Ok(parent.unwrap())
}

fn main() -> windows::Result<()> {
    unsafe {
        RoInitialize(RO_INIT_TYPE::RO_INIT_SINGLETHREADED).ok()?;
    }

    let _controller = create_dispatcher_queue_controller_for_current_thread()?;

    let window = BasicWindow::new(500, 600, "rustd3d.MainWindow", "rustd3d");

    let compositor = Compositor::new()?;
    let target = {
        let compositor_desktop: ICompositorDesktopInterop = compositor.cast()?;
        let mut target = None;
        compositor_desktop
            .CreateDesktopWindowTarget(window.handle(), false.into(), &mut target)
            .ok()?;
        target.unwrap()
    };
    let background = compositor.create_sprite_visual()?;
    background.set_relative_size_adjustment(Vector2 { x: 1.0, y: 1.0 })?;
    background.set_brush(compositor.create_color_brush_with_color(Colors::black()?)?)?;
    target.set_root(&background)?;

    let visual = compositor.create_sprite_visual()?;
    visual.set_relative_size_adjustment(Vector2 { x: 1.0, y: 1.0 })?;
    background.children()?.insert_at_top(&visual)?;
    let brush = compositor.create_surface_brush()?;
    visual.set_brush(&brush)?;

    let d3d_device = create_d3d_device()?;
    let d3d_context = {
        let mut context = None;
        d3d_device.GetImmediateContext(&mut context);
        context.unwrap()
    };

    let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
        width: 500,
        height: 600,
        format: DXGI_FORMAT::DXGI_FORMAT_B8G8R8A8_UNORM,
        stereo: false.into(),
        sample_desc: DXGI_SAMPLE_DESC {
            count: 1,
            quality: 0,
        },
        buffer_usage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        buffer_count: 2,
        scaling: DXGI_SCALING::DXGI_SCALING_STRETCH,
        swap_effect: DXGI_SWAP_EFFECT::DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
        alpha_mode: DXGI_ALPHA_MODE::DXGI_ALPHA_MODE_PREMULTIPLIED,
        flags: 0,
    };
    let dxgi_device: IDXGIDevice2 = d3d_device.cast()?;
    let dxgi_adapter = dxgi_object_get_parent::<IDXGIAdapter>(&dxgi_device.cast()?)?;
    let dxgi_factory = dxgi_object_get_parent::<IDXGIFactory2>(&dxgi_adapter.cast()?)?;
    let swap_chain = {
        let mut swap_chain = None;
        dxgi_factory
            .CreateSwapChainForComposition(&dxgi_device, &swap_chain_desc, None, &mut swap_chain)
            .ok()?;
        swap_chain.unwrap()
    };

    let surface = {
        let unknown: IUnknown = swap_chain.cast()?;
        let compositor_interop: ICompositorInterop = compositor.cast()?;
        let mut surface = None;
        compositor_interop
            .CreateCompositionSurfaceForSwapChain(Some(unknown), &mut surface)
            .ok()?;
        surface.unwrap()
    };
    brush.set_surface(surface)?;

    let back_buffer: ID3D11Texture2D = {
        let mut buffer = None;
        swap_chain
            .GetBuffer(0, &ID3D11Texture2D::IID, &mut buffer as *mut _ as _)
            .ok()?;
        buffer.unwrap()
    };

    let render_target_view = {
        let resource: ID3D11Resource = back_buffer.cast()?;
        let mut render_target_view = None;
        d3d_device
            .CreateRenderTargetView(Some(resource), std::ptr::null(), &mut render_target_view)
            .ok()?;
        render_target_view
            .unwrap()
            .cast::<ID3D11RenderTargetView>()?
    };

    d3d_context.ClearRenderTargetView(
        Some(render_target_view),
        &[1.0f32, 0.0, 0.0, 1.0] as *const f32,
    );

    let present_params = DXGI_PRESENT_PARAMETERS::default();
    swap_chain.Present1(1, 0, &present_params).ok()?;

    unsafe {
        let mut message = MSG::default();
        while GetMessageA(&mut message, HWND(0), 0, 0).into() {
            DispatchMessageA(&mut message);
        }
    }

    Ok(())
}
