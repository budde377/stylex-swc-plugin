use std::{
  any::type_name,
  collections::HashSet,
  fs,
  hash::{DefaultHasher, Hash, Hasher},
  ops::Deref,
  path::{Path, PathBuf},
};

use path_clean::PathClean;
use radix_fmt::radix;
use swc_core::{
  common::{FileName, Span, DUMMY_SP},
  ecma::ast::{
    ArrayLit, BinExpr, BinaryOp, BindingIdent, Bool, Decl, Expr, ExprOrSpread, Id, Ident,
    ImportDecl, ImportSpecifier, KeyValueProp, Lit, MemberExpr, Module, ModuleDecl,
    ModuleExportName, ModuleItem, Number, ObjectLit, Pat, Prop, PropName, PropOrSpread, Stmt, Str,
    Tpl, UnaryExpr, UnaryOp, VarDeclarator,
  },
};

use crate::shared::{
  constants::{self, messages::ILLEGAL_PROP_VALUE},
  enums::{TopLevelExpression, TopLevelExpressionKind, VarDeclAction},
  regex::{DASHIFY_REGEX, IDENT_PROP_REGEX},
  structures::{
    functions::{FunctionConfigType, FunctionMap, FunctionType},
    state_manager::{StateManager, EXTENSIONS},
  },
};

use super::{
  css::stylex::evaluate::{evaluate_cached, State},
  js::stylex::stylex_types::BaseCSSType,
};

pub fn prop_or_spread_expression_creator(key: &str, value: Box<Expr>) -> PropOrSpread {
  PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
    key: string_to_prop_name(key).unwrap(),
    value,
  })))
}

// NOTE: Tests only using this function
#[allow(dead_code)]
pub(crate) fn prop_or_spread_expr_creator(key: &str, values: Vec<PropOrSpread>) -> PropOrSpread {
  let object = ObjectLit {
    span: DUMMY_SP,
    props: values,
  };

  prop_or_spread_expression_creator(key, Box::new(Expr::Object(object)))
}

pub fn key_value_creator(key: &str, value: Expr) -> KeyValueProp {
  KeyValueProp {
    key: PropName::Ident(Ident::new(key.into(), DUMMY_SP)),
    value: Box::new(value),
  }
}

pub(crate) fn prop_or_spread_string_creator(key: &str, value: &str) -> PropOrSpread {
  let value = string_to_expression(value);

  match value {
    Some(value) => prop_or_spread_expression_creator(key, Box::new(value)),
    None => panic!("Value is not a string"),
  }
}

// NOTE: Tests only using this function
#[allow(dead_code)]
pub(crate) fn prop_or_spread_array_string_creator(key: &str, value: &[&str]) -> PropOrSpread {
  let array = ArrayLit {
    span: DUMMY_SP,
    elems: value
      .iter()
      .map(|v| Option::Some(expr_or_spread_string_expression_creator(v)))
      .collect::<Vec<Option<ExprOrSpread>>>(),
  };

  prop_or_spread_expression_creator(key, Box::new(Expr::Array(array)))
}

pub(crate) fn _prop_or_spread_boolean_creator(key: &str, value: Option<bool>) -> PropOrSpread {
  match value {
    Some(value) => prop_or_spread_expression_creator(
      key,
      Box::new(Expr::Lit(Lit::Bool(Bool {
        span: DUMMY_SP,
        value,
      }))),
    ),
    None => panic!("Value is not a boolean"),
  }
}

// Converts a string to an expression.
pub(crate) fn string_to_expression(value: &str) -> Option<Expr> {
  Option::Some(Expr::Lit(Lit::Str(value.into())))
}

// NOTE: Tests only using this function
#[allow(dead_code)]
fn array_fabric(values: &[Expr], spread: Option<Span>) -> Option<ArrayLit> {
  let array = ArrayLit {
    span: DUMMY_SP,
    elems: values
      .iter()
      .map(|value| {
        Some(ExprOrSpread {
          spread,
          expr: Box::new(value.clone()),
        })
      })
      .collect(),
  };

  Option::Some(array)
}

// NOTE: Tests only using this function
#[allow(dead_code)]
pub(crate) fn create_array(values: &[Expr]) -> Option<ArrayLit> {
  array_fabric(values, Option::None)
}

pub(crate) fn _create_spreaded_array(values: &[Expr]) -> Option<ArrayLit> {
  array_fabric(values, Option::Some(DUMMY_SP))
}

// Converts a string to an expression.
pub(crate) fn string_to_prop_name(value: &str) -> Option<PropName> {
  if IDENT_PROP_REGEX.is_match(value) && value.parse::<i64>().is_err() {
    Some(PropName::Ident(Ident::new(value.into(), DUMMY_SP)))
  } else {
    Some(PropName::Str(Str {
      span: DUMMY_SP,
      value: value.into(),
      raw: None,
    }))
  }
}

// Converts a number to an expression.
pub(crate) fn number_to_expression(value: f64) -> Option<Expr> {
  Option::Some(Expr::Lit(Lit::Num(Number {
    span: DUMMY_SP,
    value,
    // value: trancate_f64(value),
    raw: Option::None,
  })))
}

pub(crate) fn extract_filename_from_path(path: FileName) -> String {
  match path {
    FileName::Real(path_buf) => path_buf.file_stem().unwrap().to_str().unwrap().to_string(),
    _ => "UnknownFile".to_string(),
  }
}

pub(crate) fn extract_path(path: FileName) -> String {
  match path {
    FileName::Real(path_buf) => path_buf.to_str().unwrap().to_string(),
    _ => "UnknownFile".to_string(),
  }
}

pub(crate) fn extract_filename_with_ext_from_path(path: FileName) -> Option<String> {
  match path {
    FileName::Real(path_buf) => {
      Option::Some(path_buf.file_name().unwrap().to_str().unwrap().to_string())
    }
    _ => Option::None,
  }
}

pub fn create_hash(value: &str) -> String {
  radix(murmur2::murmur2(value.as_bytes(), 1), 36).to_string()
}

pub(crate) fn get_string_val_from_lit(value: &Lit) -> Option<String> {
  match value {
    Lit::Str(str) => Option::Some(format!("{}", str.value)),
    Lit::Num(num) => Option::Some(format!("{}", num.value)),
    Lit::BigInt(big_int) => Option::Some(format!("{}", big_int.value)),
    _ => Option::None, // _ => panic!("{}", ILLEGAL_PROP_VALUE),
  }
}

pub(crate) fn get_key_str(key_value: &KeyValueProp) -> String {
  let key = &key_value.key;
  let mut should_wrap_in_quotes = false;

  let key = match key {
    PropName::Ident(ident) => &*ident.sym,
    PropName::Str(str) => {
      should_wrap_in_quotes = false;

      &*str.value
    }
    _ => panic!("Key is not recognized"),
  };

  wrap_key_in_quotes(key, &should_wrap_in_quotes)
}

pub(crate) fn wrap_key_in_quotes(key: &str, should_wrap_in_quotes: &bool) -> String {
  if *should_wrap_in_quotes {
    format!("\"{}\"", key)
  } else {
    key.to_string()
  }
}

pub(crate) fn expr_or_spread_string_expression_creator(value: &str) -> ExprOrSpread {
  let expr = Box::new(string_to_expression(value).expect(constants::messages::NON_STATIC_VALUE));

  ExprOrSpread {
    expr,
    spread: Option::None,
  }
}

pub(crate) fn expr_or_spread_number_expression_creator(value: f64) -> ExprOrSpread {
  let expr = Box::new(number_to_expression(value).unwrap());

  ExprOrSpread {
    expr,
    spread: Option::None,
  }
}

pub fn reduce_ident_count<'a>(state: &'a mut StateManager, ident: &'a Ident) {
  *state.var_decl_count_map.entry(ident.to_id()).or_insert(0) -= 1;
}

pub fn increase_member_ident(state: &mut StateManager, member_obj: &MemberExpr) {
  if let Some(obj_ident) = member_obj.obj.as_ident() {
    increase_member_ident_count(state, &obj_ident.to_id());
  }
}

pub fn reduce_member_expression_count(state: &mut StateManager, member_expression: &MemberExpr) {
  if let Some(obj_ident) = member_expression.obj.as_ident() {
    reduce_member_ident_count(state, &obj_ident.to_id());
  }
}

pub fn reduce_member_ident_count(state: &mut StateManager, obj_ident: &Id) {
  *state
    .member_object_ident_count_map
    .entry(obj_ident.clone())
    .or_insert(0) -= 1;
}

pub fn increase_ident_count(state: &mut StateManager, ident: &Ident) {
  increase_ident_count_by_count(state, ident, 1);
}

pub fn increase_member_ident_count(state: &mut StateManager, obj_ident: &Id) {
  increase_member_ident_count_by_count(state, obj_ident, 1);
}

pub fn increase_ident_count_by_count(state: &mut StateManager, ident: &Ident, count: i8) {
  let ident_id = &ident.to_id();
  *state
    .var_decl_count_map
    .entry(ident_id.clone())
    .or_insert(0) += count;
}

pub fn increase_member_ident_count_by_count(state: &mut StateManager, obj_ident: &Id, count: i8) {
  *state
    .member_object_ident_count_map
    .entry(obj_ident.clone())
    .or_insert(0) += count;
}

pub fn get_var_decl_by_ident<'a>(
  ident: &'a Ident,
  state: &'a mut StateManager,
  functions: &'a FunctionMap,
  action: VarDeclAction,
) -> Option<VarDeclarator> {
  match action {
    VarDeclAction::Increase => increase_ident_count(state, ident),
    VarDeclAction::Reduce => reduce_ident_count(state, ident),
    VarDeclAction::None => {}
  };

  match get_var_decl_from(state, ident) {
    Some(var_decl) => Some(var_decl.clone()),
    None => {
      let func = functions.identifiers.get(&ident.to_id());

      match func {
        Some(func) => {
          let func = func.clone();
          match func.as_ref() {
            FunctionConfigType::Regular(func) => {
              match func.fn_ptr.clone() {
                FunctionType::Mapper(func) => {
                  // let arg = Expr::Ident(ident.clone());
                  let result = func();

                  // println!("!!!!! ident: {:?}, result: {:?}", ident, result);

                  let var_decl = VarDeclarator {
                    span: DUMMY_SP,
                    name: Pat::Ident(BindingIdent {
                      id: ident.clone(),
                      type_ann: Option::None,
                    }),
                    init: Option::Some(Box::new(result)), // Clone the result
                    definite: false,
                  };

                  let var_declarator = var_decl.clone();
                  Option::Some(var_declarator)
                }
                _ => panic!("Function type not supported"),
              }
            }
            FunctionConfigType::Map(_) => todo!("FunctionConfigType::Map"),
          }
        }
        None => Option::None,
      }
    }
  }
}

pub fn get_import_by_ident<'a>(
  ident: &'a Ident,
  state: &'a mut StateManager,
) -> Option<ImportDecl> {
  get_import_from(state, ident).cloned()
}

pub(crate) fn get_var_decl_from<'a>(
  state: &'a StateManager,
  ident: &'a Ident,
) -> Option<&'a VarDeclarator> {
  state.declarations.iter().find(|var_declarator| {
    if let Pat::Ident(binding_indent) = &var_declarator.name {
      return binding_indent.sym == ident.sym;
    }

    false
  })
}

pub(crate) fn get_import_from<'a>(
  state: &'a StateManager,
  ident: &'a Ident,
) -> Option<&'a ImportDecl> {
  state.top_imports.iter().find(|import| {
    import.specifiers.iter().any(|specifier| match specifier {
      ImportSpecifier::Named(named_import) => {
        named_import.local.sym == ident.sym || {
          if let Some(imported) = &named_import.imported {
            match imported {
              ModuleExportName::Ident(export_ident) => export_ident.sym == ident.sym,
              ModuleExportName::Str(str) => str.value == ident.sym,
            }
          } else {
            false
          }
        }
      }
      ImportSpecifier::Default(default_import) => default_import.local.sym == ident.sym,
      ImportSpecifier::Namespace(namespace_import) => namespace_import.local.sym == ident.sym,
    })
  })
}

pub(crate) fn get_var_decl_by_ident_or_member<'a>(
  state: &'a StateManager,
  ident: &'a Ident,
) -> Option<&'a VarDeclarator> {
  state.declarations.iter().find(|var_declarator| {
    if let Pat::Ident(binding_indent) = &var_declarator.name {
      if binding_indent.sym == ident.sym {
        return true;
      }
    }

    var_declarator
      .init
      .as_ref()
      .and_then(|init| init.as_call())
      .and_then(|call| call.callee.as_expr())
      .and_then(|callee| callee.as_member())
      .and_then(|member| member.prop.as_ident())
      .map_or(false, |member_ident| member_ident.sym == ident.sym)
  })
}

pub fn get_expr_from_var_decl(var_decl: &VarDeclarator) -> Expr {
  match &var_decl.init {
    Some(var_decl_init) => *var_decl_init.clone(),
    None => panic!("Variable declaration is not an expression"),
  }
}

pub fn expr_to_num(expr_num: &Expr, traversal_state: &mut StateManager) -> f64 {
  match &expr_num {
    Expr::Ident(ident) => ident_to_number(ident, traversal_state, &FunctionMap::default()),
    Expr::Lit(lit) => lit_to_num(lit),
    Expr::Unary(unary) => unari_to_num(unary, traversal_state),
    Expr::Bin(lit) => {
      // dbg!(&traversal_state.var_decl_count_map);

      let mut state = Box::new(State::new(traversal_state));

      match binary_expr_to_num(lit, &mut state) {
        Some(result) => result,
        None => panic!("Binary expression is not a number"),
      }
    }
    _ => panic!("Expression in not a number {:?}", expr_num),
  }
}

fn ident_to_string(ident: &Ident, state: &mut StateManager, functions: &FunctionMap) -> String {
  let var_decl = get_var_decl_by_ident(ident, state, functions, VarDeclAction::Reduce);

  // println!("var_decl: {:?}, ident: {:?}", var_decl, ident);

  match &var_decl {
    Some(var_decl) => {
      let var_decl_expr = get_expr_from_var_decl(var_decl);

      match &var_decl_expr {
        Expr::Lit(lit) => get_string_val_from_lit(lit).expect(ILLEGAL_PROP_VALUE),
        Expr::Ident(ident) => ident_to_string(ident, state, functions),
        _ => panic!("{}", ILLEGAL_PROP_VALUE),
      }
    }
    None => panic!("{}", ILLEGAL_PROP_VALUE),
  }
}

pub fn expr_to_str(
  expr_string: &Expr,
  state: &mut StateManager,
  functions: &FunctionMap,
) -> String {
  match &expr_string {
    Expr::Ident(ident) => ident_to_string(ident, state, functions),
    Expr::Lit(lit) => get_string_val_from_lit(lit).expect("Value is not a string"),
    _ => panic!("Expression in not a string {:?}", expr_string),
  }
}

pub fn unari_to_num(unary_expr: &UnaryExpr, state: &mut StateManager) -> f64 {
  let arg = unary_expr.arg.as_ref();
  let op = unary_expr.op;

  match &op {
    UnaryOp::Minus => expr_to_num(arg, state) * -1.0,
    UnaryOp::Plus => expr_to_num(arg, state),
    _ => panic!("Union operation '{}' is invalid", op),
  }
}

pub fn binary_expr_to_num(binary_expr: &BinExpr, state: &mut State) -> Option<f64> {
  let binary_expr = binary_expr.clone();

  let op = binary_expr.op;
  let Some(left) = evaluate_cached(&binary_expr.left, state) else {
    // dbg!(binary_expr.left);

    if !state.confident {
      return Option::None;
    }

    panic!("Left expression is not a number")
  };

  let Some(right) = evaluate_cached(&binary_expr.right, state) else {
    // dbg!(binary_expr.right);

    if !state.confident {
      return Option::None;
    }

    panic!("Right expression is not a number")
  };

  // dbg!(&left, &right, &op);

  let result = match &op {
    BinaryOp::Add => {
      expr_to_num(left.as_expr()?, &mut state.traversal_state)
        + expr_to_num(right.as_expr()?, &mut state.traversal_state)
    }
    BinaryOp::Sub => {
      expr_to_num(left.as_expr()?, &mut state.traversal_state)
        - expr_to_num(right.as_expr()?, &mut state.traversal_state)
    }
    BinaryOp::Mul => {
      expr_to_num(left.as_expr()?, &mut state.traversal_state)
        * expr_to_num(right.as_expr()?, &mut state.traversal_state)
    }
    BinaryOp::Div => {
      expr_to_num(left.as_expr()?, &mut state.traversal_state)
        / expr_to_num(right.as_expr()?, &mut state.traversal_state)
    }
    BinaryOp::Mod => {
      expr_to_num(left.as_expr()?, &mut state.traversal_state)
        % expr_to_num(right.as_expr()?, &mut state.traversal_state)
    }
    BinaryOp::Exp => expr_to_num(left.as_expr()?, &mut state.traversal_state)
      .powf(expr_to_num(right.as_expr()?, &mut state.traversal_state)),
    BinaryOp::RShift => {
      ((expr_to_num(left.as_expr()?, &mut state.traversal_state) as i32)
        >> expr_to_num(right.as_expr()?, &mut state.traversal_state) as i32) as f64
    }
    BinaryOp::LShift => {
      ((expr_to_num(left.as_expr()?, &mut state.traversal_state) as i32)
        << expr_to_num(right.as_expr()?, &mut state.traversal_state) as i32) as f64
    }
    BinaryOp::BitAnd => {
      ((expr_to_num(left.as_expr()?, &mut state.traversal_state) as i32)
        & expr_to_num(right.as_expr()?, &mut state.traversal_state) as i32) as f64
    }
    BinaryOp::BitOr => {
      ((expr_to_num(left.as_expr()?, &mut state.traversal_state) as i32)
        | expr_to_num(right.as_expr()?, &mut state.traversal_state) as i32) as f64
    }
    BinaryOp::BitXor => {
      ((expr_to_num(left.as_expr()?, &mut state.traversal_state) as i32)
        ^ expr_to_num(right.as_expr()?, &mut state.traversal_state) as i32) as f64
    }
    BinaryOp::In => {
      if expr_to_num(right.as_expr()?, &mut state.traversal_state) == 0.0 {
        1.0
      } else {
        0.0
      }
    }
    BinaryOp::InstanceOf => {
      if expr_to_num(right.as_expr()?, &mut state.traversal_state) == 0.0 {
        1.0
      } else {
        0.0
      }
    }
    BinaryOp::EqEq => {
      if expr_to_num(left.as_expr()?, &mut state.traversal_state)
        == expr_to_num(right.as_expr()?, &mut state.traversal_state)
      {
        1.0
      } else {
        0.0
      }
    }
    BinaryOp::NotEq => {
      if expr_to_num(left.as_expr()?, &mut state.traversal_state)
        != expr_to_num(right.as_expr()?, &mut state.traversal_state)
      {
        1.0
      } else {
        0.0
      }
    }
    BinaryOp::EqEqEq => {
      if expr_to_num(left.as_expr()?, &mut state.traversal_state)
        == expr_to_num(right.as_expr()?, &mut state.traversal_state)
      {
        1.0
      } else {
        0.0
      }
    }
    BinaryOp::NotEqEq => {
      if expr_to_num(left.as_expr()?, &mut state.traversal_state)
        != expr_to_num(right.as_expr()?, &mut state.traversal_state)
      {
        1.0
      } else {
        0.0
      }
    }
    BinaryOp::Lt => {
      if expr_to_num(left.as_expr()?, &mut state.traversal_state)
        < expr_to_num(right.as_expr()?, &mut state.traversal_state)
      {
        1.0
      } else {
        0.0
      }
    }
    BinaryOp::LtEq => {
      if expr_to_num(left.as_expr()?, &mut state.traversal_state)
        <= expr_to_num(right.as_expr()?, &mut state.traversal_state)
      {
        1.0
      } else {
        0.0
      }
    }
    BinaryOp::Gt => {
      if expr_to_num(left.as_expr()?, &mut state.traversal_state)
        > expr_to_num(right.as_expr()?, &mut state.traversal_state)
      {
        1.0
      } else {
        0.0
      }
    }
    BinaryOp::GtEq => {
      if expr_to_num(left.as_expr()?, &mut state.traversal_state)
        >= expr_to_num(right.as_expr()?, &mut state.traversal_state)
      {
        1.0
      } else {
        0.0
      }
    }
    // #region Logical
    BinaryOp::LogicalOr => {
      let was_confident = state.confident;

      let result = evaluate_cached(&Box::new(left.as_expr()?.clone()), state);

      let left = result.unwrap();
      let left = left.as_expr().unwrap();

      let left_confident = state.confident;

      state.confident = was_confident;

      let result = evaluate_cached(&Box::new(right.as_expr()?.clone()), state);

      let right = result.unwrap();
      let right = right.as_expr().unwrap();
      let right_confident = state.confident;

      let left = expr_to_num(left, &mut state.traversal_state);
      let right = expr_to_num(right, &mut state.traversal_state);

      state.confident = left_confident && (left != 0.0 || right_confident);
      // println!("!!!!__ state.confident44444: {:#?}", state.confident);

      if !state.confident {
        return Option::None;
      }

      if left != 0.0 {
        left
      } else {
        right
      }
    }
    BinaryOp::LogicalAnd => {
      let was_confident = state.confident;

      let result = evaluate_cached(&Box::new(left.as_expr()?.clone()), state);

      let left = result.unwrap();
      let left = left.as_expr().unwrap();

      let left_confident = state.confident;

      state.confident = was_confident;

      let result = evaluate_cached(&Box::new(right.as_expr()?.clone()), state);

      let right = result.unwrap();
      let right = right.as_expr().unwrap();
      let right_confident = state.confident;

      let left = expr_to_num(left, &mut state.traversal_state);
      let right = expr_to_num(right, &mut state.traversal_state);

      state.confident = left_confident && (left == 0.0 || right_confident);

      if !state.confident {
        return Option::None;
      }

      if left != 0.0 {
        right
      } else {
        left
      }
    }
    BinaryOp::NullishCoalescing => {
      let was_confident = state.confident;

      let result = evaluate_cached(&Box::new(left.as_expr()?.clone()), state);

      let left = result.unwrap();
      let left = left.as_expr().unwrap();

      let left_confident = state.confident;

      state.confident = was_confident;

      let result = evaluate_cached(&Box::new(right.as_expr()?.clone()), state);

      let right = result.unwrap();
      let right = right.as_expr().unwrap();
      let right_confident = state.confident;

      let left = expr_to_num(left, &mut state.traversal_state);
      let right = expr_to_num(right, &mut state.traversal_state);

      state.confident = left_confident && !!(left == 0.0 || right_confident);

      if !state.confident {
        return Option::None;
      }

      if left == 0.0 {
        right
      } else {
        left
      }
    }
    // #endregion Logical
    BinaryOp::ZeroFillRShift => {
      ((expr_to_num(left.as_expr()?, &mut state.traversal_state) as i32)
        >> expr_to_num(right.as_expr()?, &mut state.traversal_state) as i32) as f64
    }
  };

  // Option::Some(trancate_f64(result))
  Option::Some(result)
}

pub fn ident_to_number(
  ident: &Ident,
  traveral_state: &mut StateManager,
  functions: &FunctionMap,
) -> f64 {
  // 1. Get the variable declaration
  let var_decl = get_var_decl_by_ident(ident, traveral_state, functions, VarDeclAction::Reduce);

  // 2. Check if it is a variable
  match &var_decl {
    Some(var_decl) => {
      // 3. Do the correct conversion according to the expression
      let var_decl_expr = get_expr_from_var_decl(var_decl);

      let mut state: State = State::new(traveral_state);

      match &var_decl_expr {
        Expr::Bin(bin_expr) => match binary_expr_to_num(bin_expr, &mut state) {
          Some(result) => result,
          None => panic!("Binary expression is not a number"),
        },
        Expr::Unary(unary_expr) => unari_to_num(unary_expr, traveral_state),
        Expr::Lit(lit) => lit_to_num(lit),
        _ => panic!("Varable {:?} is not a number", var_decl_expr),
      }
    }
    None => panic!("Variable {} is not declared", ident.sym),
  }
}

pub fn lit_to_num(lit_num: &Lit) -> f64 {
  match &lit_num {
    Lit::Bool(Bool { value, .. }) => {
      if value == &true {
        1.0
      } else {
        0.0
      }
    }
    Lit::Num(num) => num.value as f64,
    Lit::Str(str) => {
      let Result::Ok(num) = str.value.parse::<f64>() else {
        panic!("Value in not a number");
      };

      num
    }
    _ => {
      panic!("Value in not a number");
    }
  }
}

pub fn handle_tpl_to_expression(
  tpl: &swc_core::ecma::ast::Tpl,
  state: &mut StateManager,
  functions: &FunctionMap,
) -> Expr {
  // Clone the template, so we can work on it
  let mut tpl = tpl.clone();

  // Loop through each expression in the template
  for expr in tpl.exprs.iter_mut() {
    // Check if the expression is an identifier
    if let Expr::Ident(ident) = expr.as_ref() {
      // Find the variable declaration for this identifier in the AST
      let var_decl = get_var_decl_by_ident(ident, state, functions, VarDeclAction::Reduce);

      // If a variable declaration was found
      match &var_decl {
        Some(var_decl) => {
          // Swap the placeholder expression in the template with the variable declaration's initializer
          std::mem::swap(
            expr,
            &mut var_decl
              .init
              .clone()
              .expect("Variable declaration has no initializer"),
          );
        }
        None => {}
      }
    };
  }

  Expr::Tpl(tpl.clone())
}

pub fn expr_tpl_to_string(tpl: &Tpl, state: &mut StateManager, functions: &FunctionMap) -> String {
  let mut tpl_str: String = String::new();

  for (i, quasi) in tpl.quasis.iter().enumerate() {
    tpl_str.push_str(quasi.raw.as_ref());

    if i < tpl.exprs.len() {
      match &tpl.exprs[i].as_ref() {
        Expr::Ident(ident) => {
          let ident = get_var_decl_by_ident(ident, state, functions, VarDeclAction::Reduce);

          match ident {
            Some(var_decl) => {
              let var_decl_expr = get_expr_from_var_decl(&var_decl);

              let value = match &var_decl_expr {
                Expr::Lit(lit) => {
                  get_string_val_from_lit(lit).expect(constants::messages::ILLEGAL_PROP_VALUE)
                }
                _ => panic!("{}", constants::messages::ILLEGAL_PROP_VALUE),
              };

              tpl_str.push_str(value.as_str());
            }
            None => panic!("{}", constants::messages::NON_STATIC_VALUE),
          }
        }
        Expr::Bin(bin) => tpl_str.push_str(
          transform_bin_expr_to_number(bin, state)
            .to_string()
            .as_str(),
        ),
        Expr::Lit(lit) => tpl_str
          .push_str(&get_string_val_from_lit(lit).expect(constants::messages::ILLEGAL_PROP_VALUE)),
        _ => panic!("Value not suppported"), // Handle other expression types as needed
      }
    }
  }

  tpl_str
}

pub fn evaluate_bin_expr(op: BinaryOp, left: f64, right: f64) -> f64 {
  match &op {
    BinaryOp::Add => left + right,
    BinaryOp::Sub => left - right,
    BinaryOp::Mul => left * right,
    BinaryOp::Div => left / right,
    _ => panic!("Operator '{}' is not supported", op),
  }
}

pub fn transform_bin_expr_to_number(bin: &BinExpr, traversal_state: &mut StateManager) -> f64 {
  let mut state = Box::new(State::new(traversal_state));
  let op = bin.op;
  let Some(left) = evaluate_cached(&bin.left, &mut state) else {
    panic!("Left expression is not a number")
  };

  let Some(right) = evaluate_cached(&bin.right, &mut state) else {
    panic!("Left expression is not a number")
  };
  let left = expr_to_num(left.as_expr().unwrap(), traversal_state);
  let right = expr_to_num(right.as_expr().unwrap(), traversal_state);

  evaluate_bin_expr(op, left, right)
}

pub(crate) fn type_of<T>(_: T) -> &'static str {
  type_name::<T>()
}

fn prop_name_eq(a: &PropName, b: &PropName) -> bool {
  match (a, b) {
    (PropName::Ident(a), PropName::Ident(b)) => a.sym == b.sym,
    (PropName::Str(a), PropName::Str(b)) => a.value == b.value,
    (PropName::Num(a), PropName::Num(b)) => (a.value - b.value).abs() < std::f64::EPSILON,

    (PropName::BigInt(a), PropName::BigInt(b)) => a.value == b.value,
    // Add more cases as needed
    _ => false,
  }
}

pub(crate) fn remove_duplicates(props: Vec<PropOrSpread>) -> Vec<PropOrSpread> {
  let mut set = HashSet::new();
  let mut result = vec![];

  for prop in props.into_iter().rev() {
    let key = match &prop {
      PropOrSpread::Prop(prop) => match prop.as_ref().clone() {
        Prop::Shorthand(ident) => ident.sym.clone(),
        Prop::KeyValue(kv) => match kv.clone().key {
          PropName::Ident(ident) => ident.sym.clone(),
          PropName::Str(str_) => str_.value.clone(),
          _ => continue,
        },
        _ => continue,
      },
      _ => continue,
    };

    if set.insert(key) {
      result.push(prop);
    }
  }

  result.reverse();

  result
}

pub(crate) fn deep_merge_props(
  old_props: Vec<PropOrSpread>,
  mut new_props: Vec<PropOrSpread>,
) -> Vec<PropOrSpread> {
  for prop in old_props {
    match prop {
      PropOrSpread::Prop(prop) => match *prop {
        Prop::KeyValue(mut kv) => {
          if new_props.iter().any(|p| match p {
            PropOrSpread::Prop(p) => match **p {
              Prop::KeyValue(ref existing_kv) => prop_name_eq(&kv.key, &existing_kv.key),
              _ => false,
            },
            _ => false,
          }) {
            if let Expr::Object(ref mut obj1) = *kv.value {
              new_props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: kv.key.clone(),
                value: Box::new(Expr::Object(ObjectLit {
                  span: DUMMY_SP,
                  props: deep_merge_props(obj1.props.clone(), obj1.props.clone()),
                })),
              }))));
            }
          } else {
            new_props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(kv))));
          }
        }
        _ => new_props.push(PropOrSpread::Prop(Box::new(*prop))),
      },
      _ => new_props.push(prop),
    }
  }

  remove_duplicates(new_props.into_iter().rev().collect())
}

pub(crate) fn get_css_value(key_value: KeyValueProp) -> (Box<Expr>, Option<BaseCSSType>) {
  let Some(obj) = key_value.value.as_object() else {
    return (key_value.value, Option::None);
  };

  for prop in obj.props.clone().into_iter() {
    match prop {
      PropOrSpread::Spread(_) => todo!("Spread in not supported"),
      PropOrSpread::Prop(mut prop) => {
        transform_shorthand_to_key_values(&mut prop);

        match prop.deref() {
          Prop::KeyValue(key_value) => {
            // dbg!(&key_value);

            if let Some(ident) = key_value.key.as_ident() {
              if ident.sym == "syntax" {
                let value = obj.props.iter().find(|prop| {
                  match prop {
                    PropOrSpread::Spread(_) => todo!("Spread in not supported"),
                    PropOrSpread::Prop(prop) => {
                      let mut prop = prop.clone();
                      transform_shorthand_to_key_values(&mut prop);

                      match prop.as_ref() {
                        Prop::KeyValue(key_value) => {
                          if let Some(ident) = key_value.key.as_ident() {
                            return ident.sym == "value";
                          }
                        }
                        _ => todo!(),
                      }
                    }
                  }

                  false
                });
                // dbg!(&value);

                if let Some(value) = value {
                  // dbg!(&key_value);
                  let result_key_value = value.as_prop().unwrap().clone().key_value().unwrap();

                  // let value = value.value.object().unwrap().props.first().unwrap().clone();

                  // let value = value.as_prop().unwrap().clone().key_value().unwrap();

                  return (result_key_value.value, Option::Some(obj.clone().into()));
                }
              }
            }
          }
          _ => todo!(),
        }
      }
    }
  }

  (key_value.value, Option::None)
}

pub(crate) fn get_key_values_from_object(object: &ObjectLit) -> Vec<KeyValueProp> {
  let mut key_values = vec![];

  for prop in object.props.iter() {
    assert!(prop.is_prop(), "Spread in not supported");

    match prop {
      PropOrSpread::Spread(_) => todo!("Spread in not supported"),
      PropOrSpread::Prop(prop) => {
        let mut prop = prop.clone();

        transform_shorthand_to_key_values(&mut prop);
        // dbg!(&prop);

        match prop.as_ref() {
          Prop::KeyValue(key_value) => {
            key_values.push(key_value.clone());
          }
          _ => panic!("{}", constants::messages::ILLEGAL_PROP_VALUE),
        }
      }
    }
  }
  key_values
}

pub(crate) fn dashify(s: &str) -> String {
  let after = DASHIFY_REGEX.replace_all(s, "$1-$2");
  after.to_lowercase()
}

pub(crate) fn fill_top_level_expressions(module: &Module, state: &mut StateManager) {
  module.clone().body.iter().for_each(|item| match &item {
    ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) => {
      if let Decl::Var(decl_var) = &export_decl.decl {
        for decl in &decl_var.decls {
          if let Some(decl_init) = decl.init.as_ref() {
            state.top_level_expressions.push(TopLevelExpression(
              TopLevelExpressionKind::NamedExport,
              *decl_init.clone(),
              Option::Some(decl.name.as_ident().unwrap().to_id()),
            ));
            state.declarations.push(decl.clone());
          }
        }
      }
    }
    ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(export_decl)) => {
      if let Some(paren) = export_decl.expr.as_paren() {
        state.top_level_expressions.push(TopLevelExpression(
          TopLevelExpressionKind::DefaultExport,
          *paren.expr.clone(),
          None,
        ));
      } else {
        state.top_level_expressions.push(TopLevelExpression(
          TopLevelExpressionKind::DefaultExport,
          *export_decl.expr.clone(),
          None,
        ));
      }
    }
    ModuleItem::Stmt(Stmt::Decl(Decl::Var(var))) => {
      for decl in &var.decls {
        if let Some(decl_init) = decl.init.as_ref() {
          state.top_level_expressions.push(TopLevelExpression(
            TopLevelExpressionKind::Stmt,
            *decl_init.clone(),
            Option::Some(decl.name.as_ident().unwrap().to_id()),
          ));
          state.declarations.push(decl.clone());
        }
      }
    }
    _ => {}
  });
}

pub(crate) fn gen_file_based_identifier(
  file_name: &str,
  export_name: &str,
  key: Option<&str>,
) -> String {
  let key = key.map_or(String::new(), |k| format!(".{}", k));
  // dbg!(&file_name);

  format!("{}//{}{}", file_name, export_name, key)
}

pub(crate) fn hash_f64(value: f64) -> u64 {
  let bits = value.to_bits();
  let mut hasher = DefaultHasher::new();
  bits.hash(&mut hasher);
  hasher.finish()
}

pub(crate) fn round_f64(value: f64, decimal_places: u32) -> f64 {
  let multiplier = 10f64.powi(decimal_places as i32);
  (value * multiplier).round() / multiplier
}

pub(crate) fn _resolve_node_package_path(package_name: &str) -> Result<PathBuf, String> {
  match node_resolve::Resolver::default()
    .with_basedir(PathBuf::from("./cwd"))
    .preserve_symlinks(true)
    .with_extensions([".ts", ".tsx", ".js", ".jsx", ".json"])
    .with_main_fields(vec![String::from("main"), String::from("module")])
    .resolve(package_name)
  {
    Ok(path) => Ok(path),
    Err(error) => Err(format!(
      "Error resolving package {}: {:?}",
      package_name, error
    )),
  }
}

pub(crate) fn resolve_file_path(
  import_path_str: &str,
  source_file_path: &str,
  ext: &str,
  root_path: &str,
) -> std::io::Result<PathBuf> {
  let source_dir = Path::new(source_file_path).parent().unwrap();

  let mut resolved_file_path = (if import_path_str.starts_with("./") {
    source_dir
      .join(import_path_str)
      .strip_prefix(root_path)
      .unwrap()
      .to_path_buf()
  } else if import_path_str.starts_with('/') {
    Path::new(root_path).join(import_path_str)
  } else {
    Path::new("node_modules").join(import_path_str)
  })
  .clean();

  if let Some(extension) = resolved_file_path.extension() {
    let subpath = extension.to_string_lossy();

    if EXTENSIONS.iter().all(|ext| {
      let res = !ext.ends_with(subpath.as_ref());
      // println!("!!! subpath: {}, ext: {}, res: {}", subpath, ext, res);
      res
    }) {
      resolved_file_path.set_extension(format!("{}{}", subpath, ext));
    }
  } else {
    resolved_file_path.set_extension(ext);
  }

  let resolved_file_path = resolved_file_path.clean();

  let path_to_check = Path::new("./cwd").join(&resolved_file_path);

  if fs::metadata(path_to_check).is_ok() {
    Ok(resolved_file_path.to_path_buf())
  } else {
    Err(std::io::Error::new(
      std::io::ErrorKind::NotFound,
      "File not found",
    ))
  }
}

pub(crate) fn normalize_expr(expr: &Expr) -> &Expr {
  match expr {
    Expr::Paren(paren) => normalize_expr(paren.expr.as_ref()),
    _ => expr,
  }
}

pub(crate) fn transform_shorthand_to_key_values(prop: &mut Box<Prop>) {
  if let Some(ident) = prop.as_shorthand() {
    *prop = Box::new(Prop::KeyValue(KeyValueProp {
      key: PropName::Ident(ident.clone()),
      value: Box::new(Expr::Ident(ident.clone())),
    }));
  }
}

// pub(crate) fn trancate_f32(number: f32) -> f32 {
//   f32::trunc(number  * 100.0) / 100.0
// }

// pub(crate) fn trancate_f64(number: f64) -> f64 {
//   let a = f64::trunc(number * 100.0) / 100.0;

//   if a == 4.0 {
//     println!("Origin: {}, trancated: {}", &number, &a);
//   }

//   number
// }
