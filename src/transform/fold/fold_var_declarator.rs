use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::{
        ast::{BindingIdent, Expr, Ident, Pat, VarDeclarator},
        visit::FoldWith,
    },
};

use crate::{
    shared::{enums::ModuleCycle, utils::common::increase_ident_count},
    ModuleTransformVisitor,
};

impl<C> ModuleTransformVisitor<C>
where
    C: Comments,
{
    pub(crate) fn fold_var_declarator_impl(
        &mut self,
        mut var_declarator: VarDeclarator,
    ) -> VarDeclarator {
        // Get the declarations from the VarDecl struct
        // let var_declarator_id = var_declarator.clone().name.as_ident().unwrap().to_id();
        // let stylex_var_declarator = self.declaration.clone().unwrap();

        if self.cycle != ModuleCycle::Initializing && self.cycle != ModuleCycle::Processing {
            return var_declarator.fold_children_with(self);
        }

        if &var_declarator.init.is_some() == &true {
            match &*var_declarator.init.clone().unwrap() {
                //HERE
                Expr::Call(call) => {
                    let declaration_tuple = self.process_declaration(&call);

                    match &declaration_tuple {
                        Some(declaration) => {
                            let (declaration, member) = declaration;

                            println!("!!!!!declaration: {:?}", declaration);
                            println!("!!!!!self.declaration: {:?}", self.declaration);
                            println!("!!!!!member: {:?}", member);
                            if declaration.eq(&self.declaration.clone().unwrap()) {
                                let declaration_string = self.declaration.clone().unwrap().0.to_string();

                                if member.as_str() == "create" || member.as_str() == declaration_string {
                                    if self.cycle == ModuleCycle::Initializing {
                                        self.props_declaration =
                                            var_declarator.name.as_ident().map(|ident| {
                                                // increase_ident_count(
                                                //     &mut self.var_decl_count_map,
                                                //     &ident,
                                                // );

                                                ident.to_id()
                                            });
                                    } else {
                                        if !self.config.runtime_injection {
                                            var_declarator.name = Pat::Ident(BindingIdent {
                                                id: Ident {
                                                    span: DUMMY_SP,
                                                    optional: false,
                                                    sym: "_stylex$props".into(),
                                                },
                                                type_ann: None,
                                            })
                                        }
                                    }
                                }
                            }
                        }
                        None => {}
                    }
                }
                _ => {}
            }
        }

        // Call the fold_children_with method on the VarDecl struct
        var_declarator.fold_children_with(self)
    }
}
