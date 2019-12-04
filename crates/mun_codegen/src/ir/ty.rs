use super::try_convert_any_to_basic;
use crate::IrDatabase;
use hir::{ApplicationTy, CallableDef, Ty, TypeCtor};
use inkwell::types::{AnyTypeEnum, BasicType, BasicTypeEnum};

/// Given a mun type, construct an LLVM IR type
pub(crate) fn ir_query(db: &impl IrDatabase, ty: Ty) -> AnyTypeEnum {
    let context = db.context();
    match ty {
        Ty::Empty => AnyTypeEnum::StructType(context.struct_type(&[], false)),
        Ty::Apply(ApplicationTy { ctor, .. }) => match ctor {
            TypeCtor::Float => AnyTypeEnum::FloatType(context.f64_type()),
            TypeCtor::Int => AnyTypeEnum::IntType(context.i64_type()),
            TypeCtor::Bool => AnyTypeEnum::IntType(context.bool_type()),
            TypeCtor::FnDef(def @ CallableDef::Function(_)) => {
                let ty = db.callable_sig(def);
                let params: Vec<BasicTypeEnum> = ty
                    .params()
                    .iter()
                    .map(|p| try_convert_any_to_basic(db.type_ir(p.clone())).unwrap())
                    .collect();

                let fn_type = match ty.ret() {
                    Ty::Empty => context.void_type().fn_type(&params, false),
                    ty => try_convert_any_to_basic(db.type_ir(ty.clone()))
                        .expect("could not convert return value")
                        .fn_type(&params, false),
                };

                AnyTypeEnum::FunctionType(fn_type)
            }
            TypeCtor::FnDef(CallableDef::Struct(s)) | TypeCtor::Struct(s) => {
                let name = s.name(db).to_string();
                context.opaque_struct_type(&name).into()
            }
            _ => unreachable!(),
        },
        _ => unreachable!("unknown type can not be converted"),
    }
}
