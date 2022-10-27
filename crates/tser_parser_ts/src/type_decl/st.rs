use super::super::type_expr::parse_to_type_expr;
use crate::error::StructureError;

use crate::prop::parse_as_prop;
use swc_ecma_ast::{TsInterfaceBody, TsInterfaceDecl};
use tser_ir::type_decl::st::{Field, Struct};

pub fn parse_struct(ts_interface: &TsInterfaceDecl) -> Result<Struct, StructureError> {
    if let Some(type_params) = &ts_interface.type_params {
        return Err(type_params.span.into());
    }
    if let Some(extend) = ts_interface.extends.first() {
        return Err(extend.span.into());
    }
    Ok(Struct {
        name: ts_interface.id.sym.to_string(),
        fields: parse_ts_interface_body(&ts_interface.body)?,
    })
}

fn parse_ts_interface_body(
    ts_interface_body: &TsInterfaceBody,
) -> Result<Vec<Field>, StructureError> {
    ts_interface_body
        .body
        .as_slice()
        .iter()
        .map(|type_elemnnt| {
            let prop = parse_as_prop(type_elemnnt)?;
            Ok(Field {
                name: prop.name,
                optional: prop.optional,
                ty: parse_to_type_expr(prop.ts_type)?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::super::test_utils::parse_src_as_decl;
    use super::*;
    use assert_matches::assert_matches;
    use swc_ecma_ast::Decl;
    use tser_ir::type_expr::primitive::Primitive;
    use tser_ir::type_expr::{TypeExpr, TypeExprKind};

    fn parse_src_as_struct(src: &str) -> Result<Struct, StructureError> {
        let decl = parse_src_as_decl(src);
        let ts_interface =
            assert_matches!(&decl, Decl::TsInterface(ts_interface) => ts_interface.as_ref());
        parse_struct(ts_interface)
    }

    #[test]
    fn test_ts_interface_struct() {
        assert_eq!(
            parse_src_as_struct("interface Hello { foo: string }").unwrap(),
            Struct {
                name: "Hello".to_string(),
                fields: vec![Field {
                    name: "foo".to_string(),
                    ty: TypeExpr {
                        nullable: false,
                        kind: TypeExprKind::Primitive(Primitive::String),
                    },
                    optional: false
                }]
            }
        );
    }

    #[test]
    fn test_ts_interface_struct_optional() {
        assert_eq!(
            parse_src_as_struct(r"interface Hello { foo?: string }")
                .unwrap()
                .fields
                .first()
                .unwrap()
                .optional,
            true
        );
    }
}
