use stylex_swc_plugin::{
  shared::structures::{plugin_pass::PluginPass, stylex_options::StyleXOptionsParams},
  ModuleTransformVisitor,
};
use swc_core::ecma::{
  parser::{Syntax, TsSyntax},
  transforms::testing::test,
};

test!(
  Syntax::Typescript(TsSyntax {
    tsx: true,
    ..Default::default()
  }),
  |tr| ModuleTransformVisitor::new_test_styles(
    tr.comments.clone(),
    &PluginPass::default(),
    Some(&mut StyleXOptionsParams {
      gen_conditional_classes: Some(true),
      ..StyleXOptionsParams::default()
    })
  ),
  stylex_call_with_conditions,
  r#"
        import stylex from 'stylex';
        const styles = stylex.create({
            default: {
                backgroundColor: 'red',
            },
            active: {
                color: 'blue',
            }
        });
        stylex.attrs([styles.default, isActive && styles.active]);
    "#
);

test!(
  Syntax::Typescript(TsSyntax {
    tsx: true,
    ..Default::default()
  }),
  |tr| ModuleTransformVisitor::new_test_styles(
    tr.comments.clone(),
    &PluginPass::default(),
    None
  ),
  stylex_call_with_conditions_skip_conditional,
  r#"
        import stylex from 'stylex';
        const styles = stylex.create({
            default: {
                backgroundColor: 'red',
            },
            active: {
                color: 'blue',
            }
        });
        stylex.attrs([styles.default, isActive && styles.active]);
    "#
);

test!(
  Syntax::Typescript(TsSyntax {
    tsx: true,
    ..Default::default()
  }),
  |tr| ModuleTransformVisitor::new_test_styles(
    tr.comments.clone(),
    &PluginPass::default(),
    None
  ),
  stylex_call_with_property_collisions,
  r#"
        import stylex from 'stylex';
        const styles = stylex.create({
            red: {
                color: 'red',
            },
            blue: {
                color: 'blue',
            }
        });
        stylex.attrs([styles.red, styles.blue]);
        stylex.attrs([styles.blue, styles.red]);
    "#
);

test!(
  Syntax::Typescript(TsSyntax {
    tsx: true,
    ..Default::default()
  }),
  |tr| ModuleTransformVisitor::new_test_styles(
    tr.comments.clone(),
    &PluginPass::default(),
    None
  ),
  stylex_call_with_reverting_by_null,
  r#"
        import stylex from 'stylex';
        const styles = stylex.create({
            red: {
                color: 'red',
            },
            revert: {
                color: null,
            }
        });
        stylex.attrs([styles.red, styles.revert]);
        stylex.attrs([styles.revert, styles.red]);
    "#
);

test!(
  Syntax::Typescript(TsSyntax {
    tsx: true,
    ..Default::default()
  }),
  |tr| ModuleTransformVisitor::new_test_styles(
    tr.comments.clone(),
    &PluginPass::default(),
    None
  ),
  stylex_call_with_short_form_property_collisions,
  r#"
        import stylex from 'stylex';
        const styles = stylex.create({
            foo: {
                padding: 5,
                paddingEnd: 10,
            },

            bar: {
                padding: 2,
                paddingStart: 10,
            },
        });
        stylex.attrs([styles.foo, styles.bar]);
    "#
);

test!(
  Syntax::Typescript(TsSyntax {
    tsx: true,
    ..Default::default()
  }),
  |tr| ModuleTransformVisitor::new_test_styles(
    tr.comments.clone(),
    &PluginPass::default(),
    None
  ),
  stylex_call_with_short_form_property_collisions_with_null,
  r#"
        import stylex from 'stylex';
        const styles = stylex.create({
            foo: {
                padding: 5,
                paddingEnd: 10,
            },

            bar: {
                padding: 2,
                paddingStart: null,
            },
        });
        stylex.attrs([styles.foo, styles.bar]);
    "#
);

test!(
  Syntax::Typescript(TsSyntax {
    tsx: true,
    ..Default::default()
  }),
  |tr| ModuleTransformVisitor::new_test_styles(
    tr.comments.clone(),
    &PluginPass::default(),
    Some(&mut StyleXOptionsParams {
      gen_conditional_classes: Some(true),
      ..StyleXOptionsParams::default()
    })
  ),
  stylex_call_with_conditions_and_collisions,
  r#"
        import stylex from 'stylex';
        const styles = stylex.create({
            red: {
                color: 'red',
            },
            blue: {
                color: 'blue',
            }
            });
        stylex.attrs([styles.red, isActive && styles.blue]);
    "#
);

test!(
  Syntax::Typescript(TsSyntax {
    tsx: true,
    ..Default::default()
  }),
  |tr| ModuleTransformVisitor::new_test_styles(
    tr.comments.clone(),
    &PluginPass::default(),
    None
  ),
  stylex_call_with_conditions_and_collisions_skip_conditional,
  r#"
        import stylex from 'stylex';
        const styles = stylex.create({
            red: {
                color: 'red',
            },
            blue: {
                color: 'blue',
            }
        });
        stylex.attrs([styles.red, isActive && styles.blue]);
    "#
);

test!(
  Syntax::Typescript(TsSyntax {
    tsx: true,
    ..Default::default()
  }),
  |tr| ModuleTransformVisitor::new_test_styles(
    tr.comments.clone(),
    &PluginPass::default(),
    Some(&mut StyleXOptionsParams {
      gen_conditional_classes: Some(true),
      ..StyleXOptionsParams::default()
    })
  ),
  stylex_call_with_conditions_and_null_collisions,
  r#"
        import stylex from 'stylex';
        const styles = stylex.create({
            red: {
                color: 'red',
            },
            blue: {
                color: null,
            }
        });
        stylex.attrs([styles.red, isActive && styles.blue]);
    "#
);

test!(
  Syntax::Typescript(TsSyntax {
    tsx: true,
    ..Default::default()
  }),
  |tr| ModuleTransformVisitor::new_test_styles(
    tr.comments.clone(),
    &PluginPass::default(),
    None
  ),
  stylex_call_with_conditions_and_null_collisions_skip_conditional,
  r#"
        import stylex from 'stylex';
        const styles = stylex.create({
            red: {
                color: 'red',
            },
            blue: {
                color: null,
            }
        });
        stylex.attrs([styles.red, isActive && styles.blue]);
    "#
);
