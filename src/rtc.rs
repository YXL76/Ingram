use boa_engine::{object::ObjectInitializer, property::Attribute, JsValue};

pub fn init(obj: &mut ObjectInitializer, century: u8) {
    obj.property(
        "RTC_CENTURY_REG",
        JsValue::Integer(century as i32),
        Attribute::default(),
    );
}
