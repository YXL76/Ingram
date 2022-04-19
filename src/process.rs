use {
    alloc::{boxed::Box, collections::VecDeque},
    boa_engine::{
        object::{JsObject, ObjectData, ObjectInitializer},
        property::Attribute,
        vm::ReturnType,
        Context, JsResult, JsValue,
    },
    boa_gc::{unsafe_empty_trace, Finalize, Trace},
    core::{
        cell::RefCell,
        sync::atomic::{AtomicI32, Ordering},
    },
    spin::{Mutex, Once},
    x86_64::{
        instructions::segmentation::{Segment, DS},
        structures::gdt::SegmentSelector,
    },
};

pub const START_PID: i32 = 1;
static PID: AtomicI32 = AtomicI32::new(START_PID + 1);

#[derive(Finalize, Debug)]
pub struct Process {
    pub id: i32,
    /// `None` if the process is dead
    pub ctx: RefCell<Option<Context>>,
    pub microtasks: VecDeque<JsObject>,
}

unsafe impl Trace for Process {
    unsafe_empty_trace!();
}

impl Process {
    #[inline]
    fn try_new<S>(code: S) -> JsResult<(Self, Context)>
    where
        S: AsRef<[u8]>,
    {
        let mut context = Context::default();
        context.parse_and_compile(code)?;

        let id = PID.fetch_add(1, Ordering::SeqCst);

        let deno_obj = ObjectInitializer::new(&mut context)
            .property("pid", JsValue::Integer(id), Attribute::default())
            .build();

        context.register_global_property("deno", deno_obj, Attribute::default());

        Ok((
            Self {
                id,
                ctx: RefCell::new(None),
                microtasks: VecDeque::new(),
            },
            context,
        ))
    }
}

#[inline]
fn new_process<S>(code: S) -> JsResult<JsObject>
where
    S: AsRef<[u8]>,
{
    let (proc, mut context) = Process::try_new(code)?; // insert later

    let proc = {
        let mut proc = ObjectInitializer {
            context: &mut context,
            object: JsObject::from_proto_and_data(None, ObjectData::native_object(Box::new(proc))),
        };

        proc.function(
            |this, _args, _context| {
                const STEPS: usize = 512;

                let proc = this.as_object().unwrap().downcast_ref::<Process>().unwrap();
                let mut ctx = proc.ctx.borrow_mut();

                if let Some(context) = ctx.as_mut() {
                    let (_result, ret_type) = context.run_steps(STEPS)?;
                    if let ReturnType::Yield = ret_type {
                        return Ok(true.into());
                    }
                }

                Ok(false.into())
            },
            "steps",
            0,
        )
        .build()
    };

    let _ = proc
        .downcast_ref::<Process>()
        .unwrap()
        .ctx
        .borrow_mut()
        .insert(context); // put back

    Ok(proc)
}

pub static KERNEL_MICROTASKS: Once<Mutex<VecDeque<JsObject>>> = Once::new();

pub fn init(obj: &mut ObjectInitializer) {
    KERNEL_MICROTASKS.call_once(|| Mutex::new(VecDeque::with_capacity(8)));

    obj.function(
        |_this, args, context| {
            let obj = args
                .get(0)
                .and_then(|code| code.as_object())
                .ok_or(context.construct_type_error("missing code"))?
                .borrow();
            let code = obj
                .as_array_buffer()
                .ok_or(context.construct_type_error("expect ArrayBuffer"))?
                .array_buffer_data
                .as_ref()
                .unwrap();

            Ok(new_process(code)?.into())
        },
        "spawn",
        1,
    )
    .function(
        |_this, _args, _context| {
            let flag = DS::get_reg().0;
            if flag != 0 {
                unsafe { DS::set_reg(SegmentSelector(0)) };
                Ok(true.into())
            } else {
                Ok(false.into())
            }
        },
        "shouldSchedule",
        1,
    );
}
