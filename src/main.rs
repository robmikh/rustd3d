use bindings::windows::{
    win32::{
        direct3d11::{
            D3D11CreateDevice, ID3D11Device, ID3D11RenderTargetView, ID3D11Resource,
            D3D11_BIND_FLAG, D3D11_CREATE_DEVICE_FLAG, D3D11_RENDER_TARGET_VIEW_DESC,
            D3D11_RTV_DIMENSION, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE,
            D3D_DRIVER_TYPE,
        },
        dxgi::{DXGI_FORMAT, DXGI_SAMPLE_DESC},
        system_services::DXGI_ERROR_UNSUPPORTED,
    },
    ErrorCode,
};
use windows::Interface;

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

fn main() -> windows::Result<()> {
    println!("Creating device and context...");
    let device = create_d3d_device()?;
    let context = {
        let mut context = None;
        device.GetImmediateContext(&mut context);
        context.unwrap()
    };

    println!("Creating texture...");
    let texture_desc = D3D11_TEXTURE2D_DESC {
        width: 500,
        height: 600,
        mip_levels: 1,
        array_size: 1,
        format: DXGI_FORMAT::DXGI_FORMAT_B8G8R8A8_UNORM,
        sample_desc: DXGI_SAMPLE_DESC {
            count: 1,
            quality: 0,
        },
        usage: D3D11_USAGE::D3D11_USAGE_DEFAULT,
        bind_flags: D3D11_BIND_FLAG::D3D11_BIND_RENDER_TARGET.0 as u32,
        cpu_access_flags: 0,
        misc_flags: 0,
    };
    let texture = {
        let mut texture = None;
        device
            .CreateTexture2D(&texture_desc, std::ptr::null(), &mut texture)
            .ok()?;
        texture.unwrap()
    };

    println!("D3D11_TEXTURE2D_DESC size in bytes: {}", std::mem::size_of::<D3D11_TEXTURE2D_DESC>());

    println!("Testing GetDesc (custom vtable)...");
    #[repr(transparent)]
    #[allow(non_camel_case_types)]
    pub struct ID3D11Texture2DCustom(::windows::IUnknown);
    #[repr(C)]
    pub struct ID3D11Texture2DCustom_abi(
        pub  unsafe extern "system" fn(
            this: ::windows::RawPtr,
            iid: &::windows::Guid,
            interface: *mut ::windows::RawPtr,
        ) -> ::windows::ErrorCode,
        pub unsafe extern "system" fn(this: ::windows::RawPtr) -> u32,
        pub unsafe extern "system" fn(this: ::windows::RawPtr) -> u32,

        // ID3D11DeviceChild
        pub  unsafe extern "system" fn(
            this: ::windows::RawPtr,
            pp_device: *mut ::std::option::Option<ID3D11Device>,
        ),
        pub  unsafe extern "system" fn(
            this: ::windows::RawPtr,
            guid: *const ::windows::Guid,
            p_data_size: *mut u32,
            p_data: *mut ::std::ffi::c_void,
        ) -> ::windows::ErrorCode,
        pub  unsafe extern "system" fn(
            this: ::windows::RawPtr,
            guid: *const ::windows::Guid,
            data_size: u32,
            p_data: *const ::std::ffi::c_void,
        ) -> ::windows::ErrorCode,
        pub  unsafe extern "system" fn(
            this: ::windows::RawPtr,
            guid: *const ::windows::Guid,
            p_data: ::std::option::Option<::windows::IUnknown>,
        ) -> ::windows::ErrorCode,

        // ID3D11Resource
        pub  unsafe extern "system" fn(
            this: ::windows::RawPtr,
            p_resource_dimension: *mut bindings::windows::win32::direct3d11::D3D11_RESOURCE_DIMENSION,
        ),
        pub unsafe extern "system" fn(this: ::windows::RawPtr, eviction_priority: u32),
        pub unsafe extern "system" fn(this: ::windows::RawPtr) -> u32,

        // ID3D11Texture2D
        pub  unsafe extern "system" fn(
            this: ::windows::RawPtr,
            p_desc: *mut D3D11_TEXTURE2D_DESC,
        ),
    );
    unsafe impl windows::Interface for ID3D11Texture2DCustom {
        type Vtable = ID3D11Texture2DCustom_abi;
        const IID: ::windows::Guid = ::windows::Guid::from_values(
            1863690994,
            53768,
            20105,
            [154, 180, 72, 149, 53, 211, 79, 156],
        );
    }
    #[allow(non_snake_case)]
    impl ID3D11Texture2DCustom {
        pub fn GetDesc(&self, p_desc: *mut D3D11_TEXTURE2D_DESC) {
            unsafe {
                (::windows::Interface::vtable(self).10)(::windows::Abi::abi(self), p_desc)
            }
        }
    }
    let mut desc = D3D11_TEXTURE2D_DESC::default();
    let custom_texture: ID3D11Texture2DCustom = unsafe { std::mem::transmute::<_, _>(texture.clone()) };
    custom_texture.GetDesc(&mut desc);
    assert_eq!(desc.width, texture_desc.width);
    assert_eq!(desc.height, texture_desc.height);
    assert_eq!(desc.mip_levels, texture_desc.mip_levels);
    assert_eq!(desc.array_size, texture_desc.array_size);
    assert_eq!(desc.format, texture_desc.format);
    assert_eq!(desc.sample_desc, texture_desc.sample_desc);
    assert_eq!(desc.usage, texture_desc.usage);
    assert_eq!(desc.bind_flags, texture_desc.bind_flags);
    assert_eq!(desc.cpu_access_flags, texture_desc.cpu_access_flags);
    assert_eq!(desc.misc_flags, texture_desc.misc_flags);

    println!("Testing GetDesc...");
    let mut desc = D3D11_TEXTURE2D_DESC::default();
    texture.GetDesc(&mut desc);
    assert_eq!(desc.width, texture_desc.width);
    assert_eq!(desc.height, texture_desc.height);
    assert_eq!(desc.mip_levels, texture_desc.mip_levels);
    assert_eq!(desc.array_size, texture_desc.array_size);
    assert_eq!(desc.format, texture_desc.format);
    assert_eq!(desc.sample_desc, texture_desc.sample_desc);
    assert_eq!(desc.usage, texture_desc.usage);
    assert_eq!(desc.bind_flags, texture_desc.bind_flags);
    assert_eq!(desc.cpu_access_flags, texture_desc.cpu_access_flags);
    assert_eq!(desc.misc_flags, texture_desc.misc_flags);

    // Doesn't work yet
    if false {
        println!("Creating render target view...");
        let render_target_view_desc = D3D11_RENDER_TARGET_VIEW_DESC {
            format: texture_desc.format,
            view_dimension: D3D11_RTV_DIMENSION::D3D11_RTV_DIMENSION_TEXTURE2D,
            // TODO: D3D11_TEX2D_RTV
            anonymous: false,
        };
        let render_target_view = {
            let resource: ID3D11Resource = texture.cast()?;
            let mut render_target_view = None;
            device
                .CreateRenderTargetView(
                    Some(resource),
                    &render_target_view_desc,
                    &mut render_target_view,
                )
                .ok()?;
            render_target_view
                .unwrap()
                .cast::<ID3D11RenderTargetView>()?
        };

        println!("Clearing render target view...");
        context.ClearRenderTargetView(
            Some(render_target_view),
            &[1.0f32, 0.0, 0.0, 1.0] as *const f32,
        );
    }
    

    println!("Done!");
    Ok(())
}
