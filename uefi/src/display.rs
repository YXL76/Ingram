use uefi::{proto::console::gop::GraphicsOutput, table::boot::BootServices};

pub(crate) fn init(bs: &BootServices) {
    const RESOLUTION: (usize, usize) = (1024, 768);

    let output = bs.locate_protocol::<GraphicsOutput>().unwrap();
    let gop = unsafe { &mut *output.get() };
    let mode = gop
        .modes()
        .find(|m| m.info().resolution() == RESOLUTION)
        .unwrap();
    info!("mode size: {:?}", mode.info_size());
    info!("mode info: {:?}", mode.info());
    gop.set_mode(&mode).unwrap();
    info!("{:?}", gop.current_mode_info());
}
