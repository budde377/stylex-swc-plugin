use swc_core::{
  common::DUMMY_SP,
  css::{
    ast::{
      ComponentValue, Declaration, DeclarationName, DelimiterValue, Dimension, Function, Ident,
      ListOfComponentValues, Number, Stylesheet, Token,
    },
    visit::{Fold, FoldWith},
  },
};

use crate::shared::utils::common::dashify;
#[cfg(test)]
use crate::shared::utils::css::{stringify, swc_parse_css};

struct CssFolder;

impl Fold for CssFolder {
  fn fold_list_of_component_values(
    &mut self,
    list: ListOfComponentValues,
  ) -> ListOfComponentValues {
    list.fold_children_with(self)
  }

  // fn fold_simple_block(&mut self, mut simple_block: SimpleBlock) -> SimpleBlock {
  //     simple_block.value = whitespace_normalizer(&mut simple_block.value);

  //     simple_block.fold_children_with(self)
  // }

  // fn fold_token(&mut self,n:Token) -> Token {
  //     dbg!(&n);

  //     Token::LBrace
  // }

  fn fold_declaration(&mut self, mut declaration: Declaration) -> Declaration {
    let declaration = kebab_case_normalizer(&mut declaration).clone();

    // NOTE: Whitespace normalizer working out of the box with minify
    // declaration.value = whitespace_normalizer(declaration.value);

    declaration.fold_children_with(self)
  }
  fn fold_dimension(&mut self, mut dimension: Dimension) -> Dimension {
    let mut dimension = timing_normalizer(&mut dimension).clone();
    let dimension = zero_demention_normalizer(&mut dimension).clone();

    dimension.clone()
  }

  fn fold_function(&mut self, func: Function) -> Function {
    let mut fnc = func.clone();

    if let Some(last) = fnc.value.last_mut() {
      *last = last.clone().fold_with(self);
    }

    fnc
  }

  // NOTE: Whitespace normalizer working out of the box with minify
  // fn fold_function(&mut self, mut function: Function) -> Function {
  //     function.value = whitespace_normalizer(function.value);

  //     function
  // }

  // NOTE: Leding zero normalizer working out of the box with minify
  // fn fold_number(&mut self, mut number: Number) -> Number {
  //     leading_zero_normalizer(&mut number);

  //     number
  // }
}

fn _whitespace_normalizer(values: Vec<ComponentValue>) -> Vec<ComponentValue> {
  let mut index = 0;

  let values = values.clone();

  let mut a = values
    .iter()
    .filter_map(|child| {
      // dbg!(&child);
      let result = match child {
        ComponentValue::Delimiter(delimiter) => {
          let mut delimiter = delimiter.clone();

          delimiter.value = DelimiterValue::Comma;

          dbg!(&delimiter.value, &child.as_delimiter().unwrap().value);

          // Some(ComponentValue::Delimiter(delimiter))
          // None

          Some(child.clone())
        }
        ComponentValue::Dimension(_) => Some(child.clone()),
        ComponentValue::PreservedToken(preserved_token) => match &preserved_token.token {
          Token::WhiteSpace { value: _ } => {
            let prev_item = values.get(index - 1);

            if let Some(ComponentValue::PreservedToken(prev_token)) = prev_item {
              return match &prev_token.token {
                Token::Comma => None,
                _ => Some(child.clone()),
              };
            }

            Some(child.clone())
          }
          _ => Some(child.clone()),
        },
        _ => Some(child.clone()),
      };

      index += 1;
      result
    })
    .collect::<Vec<ComponentValue>>();

  a.reverse();

  a
}

fn timing_normalizer(dimension: &mut Dimension) -> &mut Dimension {
  match dimension {
    Dimension::Time(time) => {
      if !time.unit.eq("ms") || time.value.value < 10.0 {
        return dimension;
      }

      time.value = Number {
        value: time.value.value / 1000.0,
        raw: Option::None,
        span: DUMMY_SP,
      };

      time.unit = Ident {
        span: DUMMY_SP,
        value: "s".into(),
        raw: Option::None,
      };

      dimension
    }
    _ => dimension,
  }
}

fn zero_demention_normalizer(dimension: &mut Dimension) -> &mut Dimension {
  match dimension {
    Dimension::Length(length) => {
      if length.value.value != 0.0 {
        return dimension;
      }

      length.value = get_zero_demansion_value();
      length.unit = get_zero_demansion_unit();

      dbg!(&dimension);

      dimension
    }
    Dimension::Angle(angle) => {
      if angle.value.value != 0.0 {
        return dimension;
      }

      angle.value = get_zero_demansion_value();

      angle.unit = Ident {
        span: DUMMY_SP,
        value: "deg".into(),
        raw: Option::None,
      };

      dimension
    }
    Dimension::Time(time) => {
      if time.value.value != 0.0 {
        return dimension;
      }

      time.value = get_zero_demansion_value();

      time.unit = Ident {
        span: DUMMY_SP,
        value: "s".into(),
        raw: Option::None,
      };

      dimension
    }
    Dimension::Frequency(frequency) => {
      if frequency.value.value != 0.0 {
        return dimension;
      }

      frequency.value = get_zero_demansion_value();
      frequency.unit = get_zero_demansion_unit();

      dimension
    }
    Dimension::Resolution(resolution) => {
      if resolution.value.value != 0.0 {
        return dimension;
      }

      resolution.value = get_zero_demansion_value();
      resolution.unit = get_zero_demansion_unit();

      dimension
    }
    Dimension::Flex(flex) => {
      if flex.value.value != 0.0 {
        return dimension;
      }

      flex.value = get_zero_demansion_value();
      flex.unit = get_zero_demansion_unit();

      dimension
    }
    Dimension::UnknownDimension(unknown) => {
      if unknown.value.value != 0.0 {
        return dimension;
      }

      unknown.value = get_zero_demansion_value();
      unknown.unit = get_zero_demansion_unit();

      dimension
    }
  }
}

fn get_zero_demansion_value() -> Number {
  Number {
    value: 0.0,
    raw: Option::None,
    span: DUMMY_SP,
  }
}

fn get_zero_demansion_unit() -> Ident {
  Ident {
    value: "".into(),
    raw: Option::None,
    span: DUMMY_SP,
  }
}

fn _leading_zero_normalizer(number: &mut Number) -> &mut Number {
  if number.value < 1.0 && number.value >= 0.0 {
    if let Some(raw) = &number.raw {
      number.raw = Option::Some(raw.replace("0.", ".").into());
      dbg!(&number);
    }
  }

  number
}

fn kebab_case_normalizer(declaration: &mut Declaration) -> &mut Declaration {
  // dbg!(&declaration);
  match &declaration.name {
    DeclarationName::Ident(ident) => {
      if !ident.value.eq("transitionProperty") && !ident.value.eq("willChange") {
        return declaration;
      }
    }
    DeclarationName::DashedIdent(_) => return declaration,
  }

  declaration.value = declaration
    .value
    .clone()
    .into_iter()
    .map(|value| match value {
      ComponentValue::Ident(ident) => {
        let ident = Ident {
          value: dashify(ident.value.as_str()).into(),
          raw: Option::None,
          span: ident.span,
        };

        ComponentValue::Ident(Box::new(ident))
      }
      _ => value,
    })
    .collect();

  // if !declaration.value.starts_with("--") {
  //    declaration.raw_value = dashify(declaration.raw_value.as_str()).into();
  // };

  // dbg!(&declaration);

  declaration
}

pub(crate) fn base_normalizer(ast: Stylesheet) -> Stylesheet {
  let mut folder = CssFolder;
  ast.fold_with(&mut folder)
}

#[test]
fn should_normalize() {
  assert_eq!(
    stringify(&base_normalizer(
      swc_parse_css("* {{ transitionProperty: opacity, margin-top; }}")
        .0
        .unwrap(),
    )),
    "*{{transitionproperty:opacity,margin-top}}"
  );

  assert_eq!(
    stringify(&base_normalizer(
      swc_parse_css("* {{ boxShadow: 0px 2px 4px var(--shadow-1); }}")
        .0
        .unwrap(),
    )),
    "*{{boxshadow:0 2px 4px var(--shadow-1)}}"
  );

  assert_eq!(
    stringify(&base_normalizer(
      swc_parse_css("* {{ opacity: 0.5; }}").0.unwrap(),
    )),
    "*{{opacity:.5}}"
  );

  assert_eq!(
    stringify(&base_normalizer(
      swc_parse_css("* {{ transitionDuration: 500ms; }}")
        .0
        .unwrap(),
    )),
    "*{{transitionduration:.5s}}"
  );

  assert_eq!(
    stringify(&base_normalizer(
      swc_parse_css("* {{ boxShadow: 1px 1px #000; }}").0.unwrap(),
    )),
    "*{{boxshadow:1px 1px#000}}"
  );
}

// /// Stringifies the [`Stylesheet`]
// #[cfg(test)]
// pub fn stringify(node: &Stylesheet) -> String {
//     let mut buf = String::new();
//     let writer = BasicCssWriter::new(&mut buf, None, BasicCssWriterConfig::default());
//     let mut codegen = CodeGenerator::new(writer, CodegenConfig { minify: true });

//     let _ = codegen.emit(&node);

//     buf
// }
