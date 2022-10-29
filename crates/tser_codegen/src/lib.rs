pub mod rust;
pub mod swift;

use tser_block::{block, flatten, Block};
use tser_ir::type_decl::enum_::EnumKind;
use tser_ir::type_decl::{
    enum_::Enum as IrEnum, struct_::Struct as IrStruct, union::Union as IrUnion,
    union::UnionKind as IrUnionKind,
};
use tser_ir::type_expr::{primitive::Primitive, TypeExpr, TypeExprKind};
use tser_ir::File;

pub enum EnumValueType {
    Integer,
    String,
}

pub struct Struct {
    pub name: String,
    pub fields: Vec<(String, String)>,
}
impl Struct {
    fn from_ir(ir_struct: &IrStruct, code_gen: &dyn CodeGen) -> Self {
        Self {
            name: ir_struct.name.to_string(),
            fields: ir_struct
                .fields
                .iter()
                .map(|ir_field| {
                    let ty = type_expr_to_string(&ir_field.ty, code_gen);
                    (ir_field.name.clone(), ty)
                })
                .collect(),
        }
    }
}

fn type_expr_to_string(type_expr: &TypeExpr, code_gen: &dyn CodeGen) -> String {
    let unwrapped = match &type_expr.kind {
        TypeExprKind::Primitive(primitive) => code_gen.primitive_expr(*primitive),
        TypeExprKind::ArrayOf(element) => {
            let element_string = type_expr_to_string(element.as_ref(), code_gen);
            code_gen.array_expr(element_string.as_str())
        }
        TypeExprKind::Identifier(id) => code_gen.identifier_expr(id),
    };
    if type_expr.nullable {
        code_gen.optional_expr(unwrapped.as_str())
    } else {
        unwrapped
    }
}

pub struct Enum {
    pub name: String,
    pub value_type: EnumValueType,
    pub values: Vec<(String, String)>,
}
impl Enum {
    fn from_ir(ir_enum: &IrEnum, _code_gen: &dyn CodeGen) -> Self {
        let (value_type, values): (EnumValueType, Vec<(String, String)>) = match &ir_enum.kind {
            EnumKind::Integers(integers) => (
                EnumValueType::Integer,
                integers
                    .iter()
                    .map(|val| (val.name.to_string(), val.value.to_string()))
                    .collect(),
            ),
            EnumKind::Strings(strings) => (
                EnumValueType::String,
                strings
                    .iter()
                    .map(|val| (val.name.to_string(), val.value.clone()))
                    .collect(),
            ),
        };
        Self {
            name: ir_enum.name.to_string(),
            value_type,
            values,
        }
    }
}

pub enum UnionKind {
    ExternallyTagged(Vec<(String, String)>),
    InternallyTagged {
        tag_field: String,
        variants: Vec<Struct>,
    }
}
impl UnionKind {
    fn from_ir(ir_union_kind: &IrUnionKind, code_gen: &dyn CodeGen) -> Self {
        match ir_union_kind {
            IrUnionKind::ExternallyTagged(variants) => Self::ExternallyTagged(
                variants
                    .iter()
                    .map(|(name, type_expr)| {
                        (name.clone(), type_expr_to_string(type_expr, code_gen))
                    })
                    .collect(),
            ),
            IrUnionKind::InternallyTagged(internally_tagged) => Self::InternallyTagged {
                tag_field: internally_tagged.tag_field.clone(),
                variants: internally_tagged.variants
                    .iter()
                    .map(|ir_struct| Struct::from_ir(ir_struct, code_gen))
                    .collect(),
            },
        }
    }
}

pub struct Union {
    pub name: String,
    pub kind: UnionKind,
}
impl Union {
    fn from_ir(ir_union: &IrUnion, code_gen: &dyn CodeGen) -> Self {
        Self {
            name: ir_union.name.to_string(),
            kind: UnionKind::from_ir(&ir_union.kind, code_gen),
        }
    }
}

pub trait CodeGen {
    fn head(&self) -> Block;
    fn identifier_expr(&self, id: &str) -> String;
    fn primitive_expr(&self, primitive: Primitive) -> String;
    fn array_expr(&self, elem: &str) -> String;
    fn optional_expr(&self, unwrapped: &str) -> String;

    fn struct_decl(&self, struct_: Struct) -> Block;
    fn enum_decl(&self, enum_: Enum) -> Block;
    fn union_decl(&self, union: Union) -> Block;
}

pub fn generate(ir_file: &File, code_gen: &dyn CodeGen) -> String {
    use tser_ir::type_decl::TypeDecl;
    use tser_ir::Item;
    let head = code_gen.head();
    let item_blocks = ir_file.items.iter().map(|item| {
        let item_block = match item {
            Item::Service(_) => unimplemented!(),
            Item::TypeDecl(type_decl) => match type_decl {
                TypeDecl::Struct(ir_struct) => {
                    code_gen.struct_decl(Struct::from_ir(ir_struct, code_gen))
                }
                TypeDecl::Union(ir_union) => {
                    code_gen.union_decl(Union::from_ir(ir_union, code_gen))
                }
                TypeDecl::Enum(ir_enum) => code_gen.enum_decl(Enum::from_ir(ir_enum, code_gen)),
            },
        };
        flatten![flatten(item_block), ""]
    });

    let file_block = block![flatten(head), flatten(item_blocks)];

    file_block.string()
}
