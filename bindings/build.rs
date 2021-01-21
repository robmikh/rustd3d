fn main() {
    windows::build!(
        windows::win32::system_services::{
            DXGI_ERROR_UNSUPPORTED,
        }
        windows::win32::direct3d11::{
            D3D11CreateDevice,
            D3D_DRIVER_TYPE,
            D3D11_CREATE_DEVICE_FLAG,
            D3D11_SDK_VERSION,
            ID3D11Device,
        }
    );
}
