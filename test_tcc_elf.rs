use cranelift_codegen::settings::{self, Configurable};
use cranelift_native;
use cranelift_object::{ObjectBuilder, ObjectModule};
use cranelift_module::{Module, Linkage, default_libcall_names};
use cranelift_frontend::{FunctionBuilderContext, FunctionBuilder};
use cranelift_codegen::ir::{types, AbiParam, Signature, CallConv};
use std::fs;
use target_lexicon::Triple;
use std::str::FromStr;

fn main() {
    let mut flag_builder = settings::builder();
    flag_builder.set("opt_level", "speed").unwrap();
    let flags = settings::Flags::new(flag_builder);
    
    // Force ELF format on Windows by using a linux target
    let target = Triple::from_str("x86_64-unknown-linux-gnu").unwrap();
    let isa = cranelift_codegen::isa::lookup(target).unwrap().finish(flags).unwrap();
    
    let mut obj_builder = ObjectBuilder::new(isa, "test", default_libcall_names()).unwrap();
    let mut module = ObjectModule::new(obj_builder);
    
    let mut sig = Signature::new(CallConv::SystemV);
    sig.returns.push(AbiParam::new(types::I32));
    
    let func_id = module.declare_function("main", Linkage::Export, &sig).unwrap();
    
    let mut ctx = module.make_context();
    ctx.func.signature = sig;
    ctx.func.name = cranelift_codegen::ir::ExternalName::user(0, 0);
    
    let mut fb_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
    
    let block = builder.create_block();
    builder.switch_to_block(block);
    builder.append_block_params_for_function_params(block);
    
    let v = builder.ins().iconst(types::I32, 42);
    builder.ins().return_(&[v]);
    builder.seal_all_blocks();
    builder.finalize();
    
    module.define_function(func_id, &mut ctx).unwrap();
    let product = module.finish();
    fs::write("test.o", product.emit().unwrap()).unwrap();
}
