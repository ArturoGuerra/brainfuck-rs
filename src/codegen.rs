use crate::ast::{Ast, Operator};
use inkwell::execution_engine::JitFunction;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, PointerValue};
use inkwell::OptimizationLevel;
use inkwell::{builder::Builder, context::Context, module::Linkage};
use inkwell::{AddressSpace, IntPredicate};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub struct IRCodegen<'a, 'ctx> {
    ast: &'a Ast,
    funcs: HashMap<String, FunctionValue<'ctx>>,
    context: Context,
}

impl<'a, 'ctx> From<&'a Ast> for IRCodegen<'a, 'ctx> {
    fn from(source: &'a Ast) -> IRCodegen<'a, 'ctx> {
        IRCodegen {
            ast: source,
            funcs: HashMap::new(),
            context: Context::create(),
        }
    }
}

impl<'a, 'ctx> IRCodegen<'a, 'ctx> {
    pub fn jit(&self) {
        let module = self.build_module();
        let exec_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        unsafe {
            type Func = unsafe extern "C" fn() -> i32;
            let f: JitFunction<Func> = exec_engine.get_function("main").unwrap();
            f.call();
        }
    }

    pub fn compile(&self, name: &str) {
        std::fs::create_dir_all("./out").expect("unable to create output dir");
        let path = Path::new("./out");
        let ir = self.build_module().to_string();
        let ir_filepath = path.join("output.ll");

        let mut file = File::create(&ir_filepath).expect("unable to create IR file");
        file.write_all(ir.as_bytes())
            .expect("unable to write IR to file");

        let output = path.join(name);
        let status = Command::new("clang")
            .arg("-o")
            .arg(output)
            .arg(ir_filepath)
            .status()
            .expect("failed to execute clang");

        println!("{:?}", status);
    }

    pub fn build_module(&self) -> Module {
        let module = self.context.create_module("brainfuck_rs");
        let builder = self.context.create_builder();

        // Include putchar function
        let putchar_fn_type = self
            .context
            .i32_type()
            .fn_type(&[self.context.i32_type().into()], false);
        let putchar_fn = module.add_function("putchar", putchar_fn_type, Some(Linkage::External));

        // Include getchar function
        let getchar_fn_type = self.context.i32_type().fn_type(&[], false);
        let getchar_fn = module.add_function("getchar", getchar_fn_type, Some(Linkage::External));

        // Setup Memory and get a pointer to its first element
        let memory = self.context.i8_type().array_type(30000);
        let memory_global = module.add_global(memory.const_zero().get_type(), None, "memory");
        memory_global.set_initializer(&memory.const_zero());

        // Function Header Setup
        let main_fn_type = self.context.i8_type().fn_type(&[], false);
        let main_fn = module.add_function("main", main_fn_type, Some(Linkage::External));
        let basic_block = self.context.append_basic_block(main_fn, "entry");
        builder.position_at_end(basic_block);

        let ptr = builder
            .build_alloca(self.context.ptr_type(AddressSpace::default()), "ptr")
            .unwrap();

        let mem_ptr = unsafe {
            builder
                .build_gep(
                    self.context.ptr_type(AddressSpace::default()),
                    memory_global.as_pointer_value(),
                    &[self.context.i8_type().const_zero()],
                    "mem_ptr_gep",
                )
                .expect("unable to get pointer to memory")
        };
        builder.build_store(ptr, mem_ptr).unwrap();

        let mut ircode = IRCodegen::from(self.ast);
        ircode.funcs.insert("putchar".to_string(), putchar_fn);
        ircode.funcs.insert("getchar".to_string(), getchar_fn);
        ircode.funcs.insert("main".to_string(), main_fn);

        ircode.build(&self.context, &builder, &ptr, ircode.ast);

        let ret = self.context.i8_type().const_zero();

        builder.build_return(Some(&ret)).unwrap();

        module
    }

    fn build(
        &self,
        context: &'a Context,
        builder: &'a Builder<'a>,
        ptr: &'a PointerValue<'a>,
        ast: &'a Ast,
    ) {
        for op in ast.inner() {
            match op {
                Operator::IncPtr => self.build_move(context, builder, ptr, 1),
                Operator::DecPtr => self.build_move(context, builder, ptr, -1),
                Operator::Inc => self.build_inc(context, builder, ptr),
                Operator::Dec => self.build_dec(context, builder, ptr),
                Operator::In => self.build_in(context, builder, ptr),
                Operator::Out => self.build_out(context, builder, ptr),
                Operator::Loop(ast) => self.build_loop(context, builder, ptr, ast),
            }
        }
    }

    // Implements [ and ]
    fn build_loop(
        &self,
        context: &'a Context,
        builder: &'a Builder,
        ptr: &'a PointerValue,
        ast: &'a Ast,
    ) {
        let start_block =
            context.append_basic_block(*self.funcs.get("main").unwrap(), "loop_start");
        let body_block = context.append_basic_block(*self.funcs.get("main").unwrap(), "loop_body");
        let end_block = context.append_basic_block(*self.funcs.get("main").unwrap(), "loop_end");

        builder.build_unconditional_branch(start_block).unwrap();
        builder.position_at_end(start_block);

        let mem_ptr = builder
            .build_load(context.ptr_type(AddressSpace::default()), *ptr, "ptr_load")
            .unwrap()
            .into_pointer_value();
        let value = builder
            .build_load(context.i8_type(), mem_ptr, "mem_ptr_load")
            .unwrap()
            .into_int_value();

        let cmp = builder
            .build_int_compare(
                IntPredicate::NE,
                value,
                context.i8_type().const_zero(),
                "loop_cond",
            )
            .unwrap();

        builder
            .build_conditional_branch(cmp, body_block, end_block)
            .unwrap();
        builder.position_at_end(body_block);

        self.build(context, builder, ptr, ast);

        builder.build_unconditional_branch(start_block).unwrap();
        builder.position_at_end(end_block);
    }

    // Implements < and >
    fn build_move(
        &self,
        context: &'a Context,
        builder: &'a Builder,
        ptr: &'a PointerValue,
        offset: i64,
    ) {
        let mem_ptr = builder
            .build_load(context.ptr_type(AddressSpace::default()), *ptr, "ptr_load")
            .unwrap()
            .into_pointer_value();

        let mem_ptr = unsafe {
            builder
                .build_gep(
                    context.ptr_type(AddressSpace::default()),
                    mem_ptr,
                    &[context.i8_type().const_int(offset as u64, false)],
                    "mem_ptr_gep",
                )
                .unwrap()
        };
        builder.build_store(*ptr, mem_ptr).unwrap();
    }

    // Implements +
    fn build_inc(&self, context: &'a Context, builder: &'a Builder, ptr: &'a PointerValue) {
        let mem_ptr = builder
            .build_load(context.ptr_type(AddressSpace::default()), *ptr, "ptr_load")
            .unwrap()
            .into_pointer_value();

        let value = builder
            .build_load(context.i8_type(), mem_ptr, "mem_ptr_load")
            .unwrap()
            .into_int_value();

        let value = builder
            .build_int_add(value, context.i8_type().const_int(1, false), "inc_data")
            .unwrap();

        builder.build_store(mem_ptr, value).unwrap();
    }

    // Implements +
    fn build_dec(&self, context: &'a Context, builder: &'a Builder, ptr: &'a PointerValue) {
        let mem_ptr = builder
            .build_load(context.ptr_type(AddressSpace::default()), *ptr, "ptr_load")
            .unwrap()
            .into_pointer_value();

        let value = builder
            .build_load(context.i8_type(), mem_ptr, "mem_ptr_load")
            .unwrap()
            .into_int_value();

        let value = builder
            .build_int_sub(value, context.i8_type().const_int(1, false), "dec_data")
            .unwrap();

        builder.build_store(mem_ptr, value).unwrap();
    }

    fn build_out(&self, context: &'a Context, builder: &'a Builder, ptr: &'a PointerValue) {
        let mem_ptr = builder
            .build_load(context.ptr_type(AddressSpace::default()), *ptr, "ptr_load")
            .unwrap()
            .into_pointer_value();

        let value = builder
            .build_load(context.i8_type(), mem_ptr, "mem_ptr_load")
            .unwrap()
            .into_int_value();

        let s = builder
            .build_int_s_extend(value, context.i32_type(), "putchar s extend")
            .unwrap();

        let func = self.funcs.get("putchar").unwrap();

        builder
            .build_call(*func, &[s.into()], "putchar call")
            .unwrap();
    }

    fn build_in(&self, context: &'a Context, builder: &'a Builder, ptr: &'a PointerValue) {
        let getchar_call = builder
            .build_call(*self.funcs.get("getchar").unwrap(), &[], "getchar call")
            .unwrap();

        let getchar = getchar_call
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        let truncated = builder
            .build_int_truncate(getchar, context.i8_type(), "getchar truncate")
            .unwrap();

        let mem_ptr = builder
            .build_load(context.ptr_type(AddressSpace::default()), *ptr, "ptr_load")
            .unwrap()
            .into_pointer_value();

        builder.build_store(mem_ptr, truncated).unwrap();
    }
}
