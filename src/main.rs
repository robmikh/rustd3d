use bindings::windows::{
    win32::{
        direct3d11::{
            D3D11CreateDevice, ID3D11Device, ID3D11RenderTargetView, ID3D11Resource,
            D3D11_BIND_FLAG, D3D11_CPU_ACCESS_FLAG, D3D11_CREATE_DEVICE_FLAG, D3D11_MAP,
            D3D11_MAPPED_SUBRESOURCE, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE,
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

    println!(
        "D3D11_TEXTURE2D_DESC size in bytes: {}",
        std::mem::size_of::<D3D11_TEXTURE2D_DESC>()
    );

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

    println!("Creating render target view...");
    let render_target_view = {
        let resource: ID3D11Resource = texture.cast()?;
        let mut render_target_view = None;
        device
            .CreateRenderTargetView(Some(resource), std::ptr::null(), &mut render_target_view)
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

    // Check to see that the texture was properly cleared

    println!("Creating staging texture...");
    desc.usage = D3D11_USAGE::D3D11_USAGE_STAGING;
    desc.bind_flags = 0;
    desc.cpu_access_flags = D3D11_CPU_ACCESS_FLAG::D3D11_CPU_ACCESS_READ.0 as u32;
    desc.misc_flags = 0;
    let staging_texture = {
        let mut texture = None;
        device
            .CreateTexture2D(&desc, std::ptr::null(), &mut texture)
            .ok()?;
        texture.unwrap()
    };
    context.CopyResource(&staging_texture, &texture);

    // Map the staging texture and check the center pixel
    let resource: ID3D11Resource = staging_texture.cast()?;
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    context
        .Map(
            Some(resource.clone()),
            0,
            D3D11_MAP::D3D11_MAP_READ,
            0,
            &mut mapped as *mut _,
        )
        .ok()?;

    let slice: &[u8] = unsafe {
        std::slice::from_raw_parts(
            mapped.p_data as *const _,
            (desc.height * mapped.row_pitch) as usize,
        )
    };

    println!("Checking the center of the texture...");
    let width = desc.width;
    let height = desc.height;
    let x = width / 2;
    let y = height / 2;
    let bytes_per_pixel = 4;
    let offset = ((y * mapped.row_pitch) + (x * bytes_per_pixel)) as usize;

    // BGRA
    let blue = slice[offset + 0];
    let green = slice[offset + 1];
    let red = slice[offset + 2];
    let alpha = slice[offset + 3];

    assert_eq!(blue, 0);
    assert_eq!(green, 0);
    assert_eq!(red, 255);
    assert_eq!(alpha, 255);
    println!("Passed!");

    context.Unmap(Some(resource), 0);

    println!("Done!");
    Ok(())
}
