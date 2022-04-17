use {
    boa_engine::{object::ObjectInitializer, Context, JsValue},
    x86_64::instructions::port::{PortReadOnly, PortWriteOnly},
};

pub fn init(obj: &mut ObjectInitializer) {
    obj.function(
        |_this: &JsValue, args: &[JsValue], context: &mut Context| {
            let port = match args.get(0) {
                Some(port) => port.to_uint16(context),
                None => context.throw_type_error("missing port"),
            }?;

            let mut port = PortReadOnly::<u8>::new(port);
            let value = unsafe { port.read() };

            Ok(JsValue::Integer(value as i32))
        },
        "inb",
        1,
    )
    .function(
        |_this: &JsValue, args: &[JsValue], context: &mut Context| {
            let port = match args.get(0) {
                Some(port) => port.to_uint16(context),
                None => context.throw_type_error("missing port"),
            }?;
            let value = match args.get(1) {
                Some(port) => port.to_uint8(context),
                None => context.throw_type_error("missing value"),
            }?;

            let mut port = PortWriteOnly::<u8>::new(port);
            unsafe { port.write(value) };

            Ok(JsValue::undefined())
        },
        "outb",
        2,
    );
}
