use {
    boa_engine::{object::ObjectInitializer, Context, JsResult, JsValue},
    core::fmt::Debug,
    x86_64::instructions::port::{PortRead, PortReadOnly, PortWrite, PortWriteOnly},
};

fn port_in<T: PortRead + Into<i32>>(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let port = match args.get(0) {
        Some(port) => port.to_uint16(context),
        None => context.throw_type_error("missing port"),
    }?;

    let mut port = PortReadOnly::<T>::new(port);
    let value = unsafe { port.read() };

    Ok(JsValue::Integer(value.into()))
}

fn port_out<T: PortWrite + TryFrom<u32> + Default>(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue>
where
    T::Error: Debug,
{
    let port = match args.get(0) {
        Some(port) => port.to_uint16(context),
        None => context.throw_type_error("missing port"),
    }?;
    let value = match args.get(1) {
        Some(port) => port.to_u32(context),
        None => context.throw_type_error("missing value"),
    }?;

    let mut port = PortWriteOnly::<T>::new(port);
    unsafe { port.write(T::try_from(value).unwrap()) };

    Ok(JsValue::undefined())
}

pub fn init(obj: &mut ObjectInitializer) {
    obj.function(port_in::<u8>, "inb", 1)
        .function(port_out::<u8>, "outb", 2)
        .function(port_in::<u16>, "inw", 1)
        .function(port_out::<u16>, "outw", 2)
        .function(
            |_this: &JsValue, args: &[JsValue], context: &mut Context| {
                let port = match args.get(0) {
                    Some(port) => port.to_uint16(context),
                    None => context.throw_type_error("missing port"),
                }?;

                let mut port = PortReadOnly::<u32>::new(port);
                let value = unsafe { port.read() };

                Ok(JsValue::Rational(value.into()))
            },
            "inl",
            1,
        )
        .function(port_out::<u32>, "outl", 2);
}
