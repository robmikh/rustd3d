use bindings::windows::{
    ErrorCode,
    win32::{
        system_services::{
            DXGI_ERROR_UNSUPPORTED,
        },
        direct3d11::{
            D3D11CreateDevice,
            D3D_DRIVER_TYPE,
            D3D11_CREATE_DEVICE_FLAG,
            D3D11_SDK_VERSION,
            ID3D11Device,
        },
    },
};

fn create_d3d_device_with_type(driver_type: D3D_DRIVER_TYPE, flags: u32, device: *mut Option<ID3D11Device>) -> ErrorCode {
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
    if hresult.0 == DXGI_ERROR_UNSUPPORTED as u32{
        hresult = create_d3d_device_with_type(
            D3D_DRIVER_TYPE::D3D_DRIVER_TYPE_WARP,
            D3D11_CREATE_DEVICE_FLAG::D3D11_CREATE_DEVICE_BGRA_SUPPORT.0 as u32,
            &mut device,
        );
    }
    hresult.ok()?;
    Ok(device.unwrap())
}

fn main() -> windows::Result<()> {
    let device = create_d3d_device()?;

    Ok(())
}
